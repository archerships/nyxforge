//! JSON-RPC client: browser → local nyxforge-node.

use anyhow::{anyhow, Result};
use serde_json::Value;

const DEFAULT_NODE: &str = "http://127.0.0.1:8888/rpc";

/// Post a JSON-RPC call to the local node and return the `result` field.
pub async fn rpc_call(method: &str, params: Value) -> Result<Value> {
    let body = serde_json::json!({ "method": method, "params": params });

    let resp = gloo_net::http::Request::post(DEFAULT_NODE)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .map_err(|e| anyhow!("request build: {e}"))?
        .send()
        .await
        .map_err(|e| anyhow!("request send: {e}"))?;

    if !resp.ok() {
        return Err(anyhow!("HTTP {}", resp.status()));
    }

    let json: Value = resp.json()
        .await
        .map_err(|e| anyhow!("parse response: {e}"))?;

    if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
        return Err(anyhow!("RPC error: {err}"));
    }

    Ok(json["result"].clone())
}
