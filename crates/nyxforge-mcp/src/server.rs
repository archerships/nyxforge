//! Axum HTTP server implementing the MCP protocol + provider management REST API.

use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::config::{McpConfig, ProviderEntry, ProviderKind};
use crate::provider::{call_provider, extract_json};

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

pub type SharedState = Arc<RwLock<McpConfig>>;

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: SharedState) -> Router {
    Router::new()
        // MCP JSON-RPC endpoint
        .route("/",                post(handle_rpc))
        // Provider management REST API
        .route("/providers",       get(list_providers))
        .route("/providers",       post(add_provider))
        .route("/providers/:name", delete(remove_provider))
        .route("/providers/default", put(set_default_provider))
        // Health check
        .route("/health",          get(health))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

async fn health() -> &'static str {
    "ok"
}

// ---------------------------------------------------------------------------
// MCP JSON-RPC handler
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RpcRequest {
    method: String,
    #[serde(default)]
    params: Value,
    #[serde(default)]
    id: Value,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
    id: Value,
}

impl RpcResponse {
    fn ok(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0", result: Some(result), error: None, id }
    }
    fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            result:  None,
            error:   Some(json!({"code": code, "message": message.into()})),
            id,
        }
    }
}

async fn handle_rpc(
    State(state): State<SharedState>,
    Json(req):    Json<RpcRequest>,
) -> impl IntoResponse {
    let id = req.id.clone();
    let resp = match req.method.as_str() {
        "initialize"   => rpc_initialize(id),
        "tools/list"   => rpc_tools_list(id),
        "tools/call"   => rpc_tools_call(state, req.params, id).await,
        other          => {
            warn!("Unknown MCP method: {other}");
            RpcResponse::err(id, -32601, format!("Method not found: {other}"))
        }
    };
    Json(resp)
}

// -- initialize --------------------------------------------------------------

fn rpc_initialize(id: Value) -> RpcResponse {
    RpcResponse::ok(id, json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name":    "nyxforge-mcp",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "capabilities": {
            "tools": {}
        }
    }))
}

// -- tools/list --------------------------------------------------------------

fn rpc_tools_list(id: Value) -> RpcResponse {
    RpcResponse::ok(id, json!({
        "tools": [{
            "name": "bond_assist",
            "description": "Analyse a social-policy-bond goal description, identify similar existing bonds on the network, and draft a new bond specification.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "description": {
                        "type":        "string",
                        "description": "Natural-language description of the social or environmental goal."
                    },
                    "existing_bonds": {
                        "type":        "array",
                        "description": "Array of bond objects from bonds.list."
                    }
                },
                "required": ["description", "existing_bonds"]
            }
        }]
    }))
}

// -- tools/call --------------------------------------------------------------

#[derive(Deserialize)]
struct ToolCallParams {
    name:      String,
    arguments: Value,
}

async fn rpc_tools_call(
    state:  SharedState,
    params: Value,
    id:     Value,
) -> RpcResponse {
    let tc: ToolCallParams = match serde_json::from_value(params) {
        Ok(v)  => v,
        Err(e) => return RpcResponse::err(id, -32602, format!("Invalid params: {e}")),
    };

    if tc.name != "bond_assist" {
        return RpcResponse::err(id, -32602, format!("Unknown tool: {}", tc.name));
    }

    let description = match tc.arguments["description"].as_str() {
        Some(s) => s.to_owned(),
        None    => return RpcResponse::err(id, -32602, "Missing 'description' argument"),
    };
    let existing_bonds = tc.arguments["existing_bonds"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // Get active provider.
    let cfg = state.read().await;
    let entry = match cfg.active_provider() {
        Ok((_, e)) => e.clone(),
        Err(e)     => return RpcResponse::err(id, -32001, e.to_string()),
    };
    drop(cfg);

    info!("Calling {} provider for bond_assist", entry.kind);

    // Call the AI provider.
    let raw_text = match call_provider(&entry, &description, &existing_bonds).await {
        Ok(t)  => t,
        Err(e) => return RpcResponse::err(id, -32002, format!("Provider error: {e}")),
    };

    // Extract JSON from model output.
    let json_str = match extract_json(&raw_text) {
        Some(s) => s,
        None    => return RpcResponse::err(id, -32003,
            format!("Could not find JSON in provider response:\n{raw_text}")),
    };

    RpcResponse::ok(id, json!({
        "content": [{"type": "text", "text": json_str}]
    }))
}

// ---------------------------------------------------------------------------
// REST: provider management
// ---------------------------------------------------------------------------

async fn list_providers(State(state): State<SharedState>) -> impl IntoResponse {
    let cfg = state.read().await;
    let list: Vec<Value> = cfg.providers.iter().map(|(name, entry)| {
        let is_default = cfg.default_provider.as_deref() == Some(name);
        json!({
            "name":        name,
            "kind":        entry.kind.to_string(),
            "model":       entry.effective_model(),
            "base_url":    entry.effective_base_url(),
            "has_api_key": entry.api_key.is_some(),
            "is_default":  is_default,
        })
    }).collect();
    Json(json!({ "providers": list }))
}

#[derive(Deserialize)]
struct AddProviderRequest {
    name:     String,
    kind:     String,
    api_key:  Option<String>,
    base_url: Option<String>,
    model:    Option<String>,
}

async fn add_provider(
    State(state): State<SharedState>,
    Json(req):    Json<AddProviderRequest>,
) -> impl IntoResponse {
    let kind: ProviderKind = match req.kind.parse() {
        Ok(k)  => k,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    };

    let entry = ProviderEntry { kind, api_key: req.api_key, base_url: req.base_url, model: req.model };

    let mut cfg = state.write().await;
    let is_first = cfg.providers.is_empty();
    cfg.providers.insert(req.name.clone(), entry);
    if is_first || cfg.default_provider.is_none() {
        cfg.default_provider = Some(req.name.clone());
    }
    if let Err(e) = cfg.save() {
        warn!("Failed to save MCP config: {e}");
    }
    (StatusCode::OK, Json(json!({ "ok": true, "name": req.name })))
}

async fn remove_provider(
    State(state): State<SharedState>,
    Path(name):   Path<String>,
) -> impl IntoResponse {
    let mut cfg = state.write().await;
    if cfg.providers.remove(&name).is_none() {
        return (StatusCode::NOT_FOUND, Json(json!({"error": format!("Provider '{name}' not found")})));
    }
    // Clear default if it was removed.
    if cfg.default_provider.as_deref() == Some(&name) {
        cfg.default_provider = cfg.providers.keys().next().cloned();
    }
    if let Err(e) = cfg.save() {
        warn!("Failed to save MCP config: {e}");
    }
    (StatusCode::OK, Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
struct SetDefaultRequest {
    name: String,
}

async fn set_default_provider(
    State(state): State<SharedState>,
    Json(req):    Json<SetDefaultRequest>,
) -> impl IntoResponse {
    let mut cfg = state.write().await;
    if !cfg.providers.contains_key(&req.name) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Provider '{}' not found", req.name)})),
        );
    }
    cfg.default_provider = Some(req.name.clone());
    if let Err(e) = cfg.save() {
        warn!("Failed to save MCP config: {e}");
    }
    (StatusCode::OK, Json(json!({ "ok": true, "default": req.name })))
}

// ---------------------------------------------------------------------------
// Server startup
// ---------------------------------------------------------------------------

pub async fn serve(addr: &str, state: SharedState) -> Result<()> {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("nyxforge-mcp listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
