//! JSON-RPC server exposed on localhost for the browser WASM frontend.
//!
//! The browser cannot reach libp2p directly, so the local node acts as a bridge:
//! WASM → HTTP JSON-RPC → node → P2P network.
//!
//! Endpoints (POST /rpc, JSON body `{"method": "...", "params": {...}}`):
//!
//!   bonds.list              — list all known bond series
//!   bonds.get               — fetch a single bond by ID
//!   bonds.issue             — submit a bond issuance
//!   orders.place            — post a bid/ask
//!   orders.cancel           — cancel a resting order
//!   status                  — node version + bond count
//!
//!   wallet.create           — generate a new wallet (xmr + drk)
//!   wallet.addresses        — return addresses
//!   wallet.balances         — return balances
//!   wallet.send_xmr         — build and submit XMR transfer
//!
//!   miner.status            — hashrate, shares, running flag
//!   miner.start             — start mining (optional threads override)
//!   miner.stop              — stop mining
//!   miner.set_threads       — change thread count

use anyhow::Result;
use axum::{extract::State, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use tracing::info;

use nyxforge_miner::MinerCmd;
use nyxforge_wallet::storage::WalletStorage;
use nyxforge_wallet::Balance;
use nyxforge_wallet::WalletKeys;

use crate::state::NodeState;

// ---------------------------------------------------------------------------
// Request / response envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RpcResponse {
    pub fn ok(v: impl Serialize) -> Self {
        Self {
            result: Some(serde_json::to_value(v).unwrap_or_default()),
            error: None,
        }
    }
    pub fn err(msg: impl ToString) -> Self {
        Self { result: None, error: Some(msg.to_string()) }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn handle_rpc(
    State(state): State<NodeState>,
    Json(req): Json<RpcRequest>,
) -> Json<RpcResponse> {
    info!(method = %req.method, "RPC call");
    let resp = dispatch(&state, req).await;
    Json(resp)
}

async fn dispatch(state: &NodeState, req: RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        // -- Node -----------------------------------------------------------
        "status" => RpcResponse::ok(serde_json::json!({
            "bonds":   state.bond_count().await,
            "version": env!("CARGO_PKG_VERSION"),
        })),

        // -- Bonds (existing stubs) -----------------------------------------
        "bonds.list" => RpcResponse::ok(serde_json::json!({ "bonds": [] })),

        "bonds.get" => {
            let id_hex = req.params["id"].as_str().unwrap_or("");
            let _ = (id_hex, state);
            RpcResponse::ok(serde_json::json!(null))
        }

        // -- Wallet ---------------------------------------------------------

        "wallet.create" => {
            let passphrase = req.params["passphrase"].as_str().unwrap_or("").to_string();
            let _ = passphrase; // TODO: use passphrase to encrypt wallet file
            match WalletKeys::generate() {
                Err(e) => RpcResponse::err(format!("key generation failed: {e}")),
                Ok(keys) => {
                    let xmr = keys.xmr_address_string();
                    let drk = keys.drk_address_string();

                    // Persist to disk.
                    let storage = WalletStorage::new(state.data_dir());
                    if let Err(e) = storage.save(&keys, Balance::zero(), 0).await {
                        return RpcResponse::err(format!("failed to save wallet: {e}"));
                    }

                    // Store in state.
                    *state.wallet().keys.write().await = Some(keys);

                    // Tell the miner which address to use — starts mining if
                    // a Start command was already received.
                    state.miner().send_cmd(MinerCmd::UpdateAddress(xmr.clone())).await;

                    RpcResponse::ok(serde_json::json!({
                        "xmr_address": xmr,
                        "drk_address": drk,
                    }))
                }
            }
        }

        "wallet.addresses" => {
            let guard = state.wallet().keys.read().await;
            match guard.as_ref() {
                None => RpcResponse::err("no wallet — call wallet.create first"),
                Some(keys) => RpcResponse::ok(serde_json::json!({
                    "xmr": keys.xmr_address_string(),
                    "drk": keys.drk_address_string(),
                })),
            }
        }

        "wallet.balances" => {
            let balance = *state.wallet().balance.read().await;
            RpcResponse::ok(serde_json::json!({
                "xmr_confirmed":   balance.xmr_confirmed,
                "xmr_unconfirmed": balance.xmr_unconfirmed,
                "drk":             balance.drk.0,
            }))
        }

        "wallet.send_xmr" => {
            let to = match req.params["to"].as_str() {
                Some(s) => s.to_string(),
                None => return RpcResponse::err("missing 'to' param"),
            };
            let amount_xmr = match req.params["amount_xmr"].as_str() {
                Some(s) => s.to_string(),
                None => return RpcResponse::err("missing 'amount_xmr' param"),
            };
            // TODO: parse amount_xmr → picomonero, call xmr::tx_builder::send_xmr
            let _ = (to, amount_xmr);
            RpcResponse::err("XMR send not yet implemented")
        }

        // -- Miner ----------------------------------------------------------

        "miner.status" => {
            let stats = state.miner().stats.read().await.clone();
            RpcResponse::ok(serde_json::json!({
                "running":           stats.running,
                "hashrate":          stats.hashrate_h_s,
                "shares":            stats.shares_found,
                "xmr_pending_pico":  stats.xmr_pending_pico,
            }))
        }

        "miner.start" => {
            let threads = req.params["threads"].as_u64().map(|t| t as usize);
            if let Some(n) = threads {
                state.miner().send_cmd(MinerCmd::SetThreads(n)).await;
            }
            if state.miner().send_cmd(MinerCmd::Start).await {
                RpcResponse::ok(serde_json::json!({ "ok": true }))
            } else {
                RpcResponse::err("miner not initialised")
            }
        }

        "miner.stop" => {
            if state.miner().send_cmd(MinerCmd::Stop).await {
                RpcResponse::ok(serde_json::json!({ "ok": true }))
            } else {
                RpcResponse::err("miner not initialised")
            }
        }

        "miner.set_threads" => {
            let count = match req.params["count"].as_u64() {
                Some(n) => n as usize,
                None => return RpcResponse::err("missing 'count' param"),
            };
            state.miner().send_cmd(MinerCmd::SetThreads(count)).await;
            RpcResponse::ok(serde_json::json!({ "ok": true }))
        }

        other => RpcResponse::err(format!("unknown method: {other}")),
    }
}

// ---------------------------------------------------------------------------
// Server startup
// ---------------------------------------------------------------------------

pub async fn run_server(state: NodeState, addr: &str) -> Result<()> {
    let app = Router::new()
        .route("/rpc", post(handle_rpc))
        .with_state(state)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::POST])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "RPC server listening");
    axum::serve(listener, app).await?;
    Ok(())
}
