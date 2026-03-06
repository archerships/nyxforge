//! JSON-RPC server exposed on localhost for the browser WASM frontend.
//!
//! The browser cannot reach libp2p directly, so the local node acts as a bridge:
//! WASM → HTTP JSON-RPC → node → P2P network.
//!
//! Endpoints (POST /rpc, JSON body `{"method": "...", "params": {...}}`):
//!
//!   bonds.propose              — publish a bond proposal for community review
//!   bonds.submit_for_approval  — send bond to listed oracles for acceptance
//!   bonds.oracle_accept        — oracle accepts responsibility for judging
//!   bonds.oracle_reject        — oracle declines (with reason)
//!   bonds.oracle_status        — show acceptance status for each oracle
//!   bonds.revise_oracles       — replace oracle list (resets responses)
//!   bonds.list                 — list all known bond series
//!   bonds.get                  — fetch a single bond by ID
//!   bonds.issue                — lock collateral and activate a Draft bond
//!   bonds.auction_price        — current Dutch auction ask price for a bond
//!   bonds.buy                  — purchase N bonds at the current auction price
//!   bonds.comment              — post a question or suggestion on a proposal
//!   bonds.comments             — list comments on a bond
//!   orders.place            — post a bid/ask
//!   orders.cancel           — cancel a resting order
//!   status                  — node version + bond count
//!
//!   wallet.create           — generate a new wallet (xmr + drk)
//!   wallet.import           — import wallet from existing XMR spend key (hex)
//!   wallet.addresses        — return addresses
//!   wallet.balances         — return balances
//!   wallet.send_xmr         — build and submit XMR transfer
//!
//!   miner.status            — hashrate, shares, running flag
//!   miner.start             — start mining (optional threads override)
//!   miner.stop              — stop mining
//!   miner.set_threads       — change thread count
//!
//!   oracle.announce         — oracle registers its supported data IDs

use anyhow::Result;
use axum::{extract::State, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use tracing::info;

use nyxforge_contract::bond_market::{process_issue_bond, IssueBondParams};
use nyxforge_core::bond::{Bond, BondComment, BondState, OracleResponse};
use nyxforge_core::types::{Digest, PublicKey};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Utc;
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

/// Parse a 32-byte hex string into a fixed-size array.
fn parse_hex32(hex: &str) -> Option<[u8; 32]> {
    let b = hex::decode(hex.trim()).ok()?;
    if b.len() != 32 { return None; }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&b);
    Some(arr)
}

async fn dispatch(state: &NodeState, req: RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        // -- Node -----------------------------------------------------------
        "status" => RpcResponse::ok(serde_json::json!({
            "bonds":   state.bond_count().await,
            "version": env!("CARGO_PKG_VERSION"),
        })),

        // -- Bonds ----------------------------------------------------------

        "bonds.list" => {
            let bonds = state.list_bonds().await;
            RpcResponse::ok(serde_json::json!({ "bonds": bonds }))
        }

        "bonds.get" => {
            let id_hex = req.params["id"].as_str().unwrap_or("");
            let id_bytes = match hex::decode(id_hex) {
                Ok(b) if b.len() == 32 => {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&b);
                    arr
                }
                _ => return RpcResponse::err("invalid bond ID: expected 32-byte hex"),
            };
            match state.get_bond(&Digest::from_bytes(id_bytes)).await {
                Some(bond) => RpcResponse::ok(bond),
                None => RpcResponse::err("bond not found"),
            }
        }

        "bonds.issue" => {
            // bonds.issue activates an existing Draft bond by locking collateral.
            // The bond must have already passed oracle approval (state = Draft),
            // unless the node was started with --allow-unverifiable.
            let bond_id_hex = req.params["bond_id"].as_str().unwrap_or("");
            let id_bytes = match parse_hex32(bond_id_hex) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bond_id: expected 32-byte hex"),
            };
            let bond_id = Digest::from_bytes(id_bytes);
            let mut bond = match state.get_bond(&bond_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bond not found"),
            };

            if !state.is_unverifiable_allowed() {
                if bond.state != BondState::Draft {
                    return RpcResponse::err(format!(
                        "bond is in '{}' state — oracle approval required before issuance. \
                         Use bonds.submit_for_approval first.",
                        serde_json::to_value(&bond.state)
                            .ok().and_then(|v| v.as_str().map(str::to_owned))
                            .unwrap_or_default()
                    ));
                }
                for goal in &bond.goals {
                    let data_id = &goal.metric.data_id;
                    if !state.is_data_id_supported(data_id).await {
                        return RpcResponse::err(format!(
                            "no oracle supports data_id '{data_id}' — \
                             start an oracle node that covers this data source"
                        ));
                    }
                }
            }

            let params = IssueBondParams {
                bond: bond.clone(),
                collateral_proof: vec![0xde, 0xad], // stub: real ZK proof in v2
            };
            match process_issue_bond(&params) {
                Err(e) => RpcResponse::err(e.to_string()),
                Ok(_) => {
                    bond.state = BondState::Active;
                    bond.activated_at_secs = Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    );
                    bond.bonds_remaining = bond.total_supply;
                    let id_hex = hex::encode(bond.id.as_bytes());
                    state.insert_bond(bond).await;
                    info!(bond_id = %id_hex, "bond issued and activated");
                    RpcResponse::ok(serde_json::json!({ "bond_id": id_hex }))
                }
            }
        }

        "bonds.propose" => {
            let mut bond: Bond = match serde_json::from_value(req.params["bond"].clone()) {
                Ok(b) => b,
                Err(e) => return RpcResponse::err(format!("invalid bond params: {e}")),
            };
            if bond.oracle.oracle_keys.is_empty() {
                return RpcResponse::err("bond must have at least one oracle key");
            }
            // Always recompute the canonical ID server-side so clients don't
            // need to replicate the blake3 derivation.
            bond.id = Bond::compute_id(
                &bond.goals,
                &bond.issuer,
                bond.created_at_block,
                &bond.return_address,
            );
            bond.state = BondState::Proposed;
            let bond_id = hex::encode(bond.id.as_bytes());
            state.insert_bond(bond).await;
            info!(bond_id = %bond_id, "bond proposal published");
            RpcResponse::ok(serde_json::json!({ "bond_id": bond_id }))
        }

        "bonds.comment" => {
            let bond_id_hex = req.params["bond_id"].as_str().unwrap_or("");
            let id_bytes = match hex::decode(bond_id_hex) {
                Ok(b) if b.len() == 32 => { let mut arr = [0u8; 32]; arr.copy_from_slice(&b); arr }
                _ => return RpcResponse::err("invalid bond_id: expected 32-byte hex"),
            };
            let bond_id = Digest::from_bytes(id_bytes);

            // Bond must exist and be in Proposed state.
            match state.get_bond(&bond_id).await {
                None => return RpcResponse::err("bond not found"),
                Some(b) if b.state != BondState::Proposed =>
                    return RpcResponse::err("comments are only accepted on Proposed bonds"),
                _ => {}
            }

            let author_hex = match req.params["author"].as_str() {
                Some(s) => s,
                None => return RpcResponse::err("missing 'author' param"),
            };
            let author_bytes = match hex::decode(author_hex) {
                Ok(b) if b.len() == 32 => { let mut arr = [0u8; 32]; arr.copy_from_slice(&b); arr }
                _ => return RpcResponse::err("invalid author: expected 32-byte hex"),
            };
            let author = PublicKey(author_bytes);

            let body = match req.params["body"].as_str() {
                Some(s) if !s.trim().is_empty() => s.to_owned(),
                _ => return RpcResponse::err("missing or empty 'body' param"),
            };

            let comment = BondComment::new(bond_id, author, body);
            let comment_id = hex::encode(comment.id.as_bytes());
            state.insert_comment(comment).await;
            info!(bond_id = %bond_id_hex, comment_id = %comment_id, "comment posted");
            RpcResponse::ok(serde_json::json!({ "comment_id": comment_id }))
        }

        "bonds.comments" => {
            let bond_id_hex = req.params["bond_id"].as_str().unwrap_or("");
            let id_bytes = match hex::decode(bond_id_hex) {
                Ok(b) if b.len() == 32 => { let mut arr = [0u8; 32]; arr.copy_from_slice(&b); arr }
                _ => return RpcResponse::err("invalid bond_id: expected 32-byte hex"),
            };
            let bond_id = Digest::from_bytes(id_bytes);
            let comments = state.get_comments(&bond_id).await;
            RpcResponse::ok(serde_json::json!({ "comments": comments }))
        }

        // ------------------------------------------------------------------ //
        // Oracle approval flow
        // ------------------------------------------------------------------ //

        "bonds.submit_for_approval" => {
            // Accept either a fresh bond (params["bond"]) or an existing
            // Proposed bond (params["bond_id"]).
            let mut bond: Bond = if req.params["bond_id"].is_string() {
                let id_bytes = match parse_hex32(req.params["bond_id"].as_str().unwrap_or("")) {
                    Some(b) => b,
                    None => return RpcResponse::err("invalid bond_id"),
                };
                match state.get_bond(&Digest::from_bytes(id_bytes)).await {
                    Some(b) => b,
                    None => return RpcResponse::err("bond not found"),
                }
            } else {
                match serde_json::from_value(req.params["bond"].clone()) {
                    Ok(b) => b,
                    Err(e) => return RpcResponse::err(format!("invalid bond params: {e}")),
                }
            };

            if bond.oracle.oracle_keys.is_empty() {
                return RpcResponse::err("bond must have at least one oracle key");
            }
            if matches!(bond.state, BondState::Active | BondState::Redeemable | BondState::Settled | BondState::Expired) {
                return RpcResponse::err("bond is already past the approval stage");
            }

            bond.state = BondState::PendingOracleApproval;
            let bond_id_hex = hex::encode(bond.id.as_bytes());
            // Clear any previous responses if re-submitted after a rejection.
            state.clear_oracle_responses(&bond.id).await;
            state.insert_bond(bond.clone()).await;
            info!(bond_id = %bond_id_hex, oracles = bond.oracle.oracle_keys.len(),
                  "bond submitted for oracle approval");
            RpcResponse::ok(serde_json::json!({
                "bond_id":      bond_id_hex,
                "awaiting":     bond.oracle.oracle_keys.len(),
            }))
        }

        "bonds.oracle_accept" => {
            let bond_id_bytes = match parse_hex32(req.params["bond_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bond_id"),
            };
            let bond_id = Digest::from_bytes(bond_id_bytes);

            let bond = match state.get_bond(&bond_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bond not found"),
            };
            if bond.state != BondState::PendingOracleApproval {
                return RpcResponse::err("bond is not awaiting oracle approval");
            }

            let oracle_key_bytes = match parse_hex32(req.params["oracle_key"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid oracle_key"),
            };
            let oracle_key = PublicKey(oracle_key_bytes);

            // Verify this key is actually listed in the bond.
            if !bond.oracle.oracle_keys.contains(&oracle_key) {
                return RpcResponse::err("oracle_key is not listed in this bond's OracleSpec");
            }

            let response = OracleResponse {
                bond_id,
                oracle_key,
                accepted: true,
                reason: None,
                responded_at: Utc::now(),
                signature: vec![], // stub
            };
            let all_accepted = state.record_oracle_response(response).await;
            let bond_id_hex = hex::encode(bond_id.as_bytes());

            if all_accepted {
                // Advance bond to Draft.
                let mut draft_bond = bond;
                draft_bond.state = BondState::Draft;
                state.insert_bond(draft_bond).await;
                info!(bond_id = %bond_id_hex, "all oracles accepted — bond advanced to Draft");
                RpcResponse::ok(serde_json::json!({
                    "bond_id": bond_id_hex,
                    "bond_state": "Draft",
                    "message": "All oracles have accepted. Bond is now in Draft state and ready for issuance.",
                }))
            } else {
                let responses = state.get_oracle_responses(&bond_id).await;
                let pending: Vec<String> = bond.oracle.oracle_keys.iter()
                    .filter(|k| !responses.iter().any(|r| r.oracle_key == **k && r.accepted))
                    .map(|k| hex::encode(&k.0))
                    .collect();
                info!(bond_id = %bond_id_hex, still_pending = pending.len(), "oracle accepted");
                RpcResponse::ok(serde_json::json!({
                    "bond_id":        bond_id_hex,
                    "bond_state":     "PendingOracleApproval",
                    "still_pending":  pending,
                }))
            }
        }

        "bonds.oracle_reject" => {
            let bond_id_bytes = match parse_hex32(req.params["bond_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bond_id"),
            };
            let bond_id = Digest::from_bytes(bond_id_bytes);

            let bond = match state.get_bond(&bond_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bond not found"),
            };
            if bond.state != BondState::PendingOracleApproval {
                return RpcResponse::err("bond is not awaiting oracle approval");
            }

            let oracle_key_bytes = match parse_hex32(req.params["oracle_key"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid oracle_key"),
            };
            let oracle_key = PublicKey(oracle_key_bytes);
            if !bond.oracle.oracle_keys.contains(&oracle_key) {
                return RpcResponse::err("oracle_key is not listed in this bond's OracleSpec");
            }

            let reason = req.params["reason"].as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned);

            let response = OracleResponse {
                bond_id,
                oracle_key,
                accepted: false,
                reason: reason.clone(),
                responded_at: Utc::now(),
                signature: vec![],
            };
            state.record_oracle_response(response).await;
            let bond_id_hex = hex::encode(bond_id.as_bytes());
            info!(bond_id = %bond_id_hex, ?reason, "oracle rejected bond");
            RpcResponse::ok(serde_json::json!({
                "bond_id": bond_id_hex,
                "message": "Rejection recorded. Issuer must revise oracle list or threshold and re-submit.",
            }))
        }

        "bonds.oracle_status" => {
            let bond_id_bytes = match parse_hex32(req.params["bond_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bond_id"),
            };
            let bond_id = Digest::from_bytes(bond_id_bytes);
            let bond = match state.get_bond(&bond_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bond not found"),
            };
            let responses = state.get_oracle_responses(&bond_id).await;

            let status: Vec<serde_json::Value> = bond.oracle.oracle_keys.iter().map(|key| {
                let key_hex = hex::encode(&key.0);
                match responses.iter().find(|r| r.oracle_key == *key) {
                    None => serde_json::json!({
                        "oracle": key_hex, "status": "pending"
                    }),
                    Some(r) if r.accepted => serde_json::json!({
                        "oracle": key_hex, "status": "accepted",
                        "responded_at": r.responded_at,
                    }),
                    Some(r) => serde_json::json!({
                        "oracle": key_hex, "status": "rejected",
                        "reason": r.reason,
                        "responded_at": r.responded_at,
                    }),
                }
            }).collect();

            RpcResponse::ok(serde_json::json!({
                "bond_id":    hex::encode(bond_id.as_bytes()),
                "bond_state": bond.state,
                "oracles":    status,
            }))
        }

        "bonds.revise_oracles" => {
            // Allows the issuer to replace the oracle list on a
            // PendingOracleApproval bond after one or more rejections.
            // Clears all existing responses so oracles must re-accept.
            let bond_id_bytes = match parse_hex32(req.params["bond_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bond_id"),
            };
            let bond_id = Digest::from_bytes(bond_id_bytes);
            let mut bond = match state.get_bond(&bond_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bond not found"),
            };
            if bond.state != BondState::PendingOracleApproval {
                return RpcResponse::err("bond must be in PendingOracleApproval state to revise oracles");
            }

            let keys_raw = match req.params["oracle_keys"].as_array() {
                Some(a) => a.clone(),
                None => return RpcResponse::err("missing 'oracle_keys' array"),
            };
            let mut new_keys = Vec::new();
            for v in &keys_raw {
                let hex = v.as_str().unwrap_or("");
                match parse_hex32(hex) {
                    Some(b) => new_keys.push(PublicKey(b)),
                    None => return RpcResponse::err(format!("invalid oracle key: '{hex}'")),
                }
            }
            if new_keys.is_empty() {
                return RpcResponse::err("bond must have at least one oracle key");
            }

            bond.oracle.oracle_keys = new_keys;
            state.clear_oracle_responses(&bond_id).await;
            state.insert_bond(bond).await;
            info!(bond_id = %hex::encode(bond_id.as_bytes()), "oracle list revised; responses cleared");
            RpcResponse::ok(serde_json::json!({
                "bond_id": hex::encode(bond_id.as_bytes()),
                "message": "Oracle list updated. All previous responses cleared. Oracles must re-accept.",
            }))
        }

        "bonds.auction_price" => {
            let bond_id_hex = req.params["bond_id"].as_str().unwrap_or("");
            let id_bytes = match parse_hex32(bond_id_hex) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bond_id: expected 32-byte hex"),
            };
            let bond = match state.get_bond(&Digest::from_bytes(id_bytes)).await {
                Some(b) => b,
                None => return RpcResponse::err("bond not found"),
            };
            let elapsed = bond.activated_at_secs.map(|t| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now.saturating_sub(t)
            }).unwrap_or(0);
            let price = bond.auction.current_price(elapsed);
            RpcResponse::ok(serde_json::json!({
                "bond_id":       bond_id_hex,
                "price_micro_drk": price.0,
            }))
        }

        "bonds.buy" => {
            let bond_id_hex = req.params["bond_id"].as_str().unwrap_or("");
            let quantity = req.params["quantity"].as_u64().unwrap_or(0);
            if quantity == 0 {
                return RpcResponse::err("quantity must be > 0");
            }
            let id_bytes = match parse_hex32(bond_id_hex) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bond_id: expected 32-byte hex"),
            };
            let mut bond = match state.get_bond(&Digest::from_bytes(id_bytes)).await {
                Some(b) => b,
                None => return RpcResponse::err("bond not found"),
            };
            if bond.state != BondState::Active {
                return RpcResponse::err("bond is not active");
            }
            if bond.bonds_remaining < quantity {
                return RpcResponse::err(format!(
                    "only {} bond(s) remaining", bond.bonds_remaining
                ));
            }
            let elapsed = bond.activated_at_secs.map(|t| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now.saturating_sub(t)
            }).unwrap_or(0);
            let price = bond.auction.current_price(elapsed);
            bond.bonds_remaining = bond.bonds_remaining.saturating_sub(quantity);
            state.insert_bond(bond).await;
            RpcResponse::ok(serde_json::json!({
                "purchased":       quantity,
                "price_micro_drk": price.0,
            }))
        }

        "oracle.announce" => {
            let ids: Vec<String> = match serde_json::from_value(req.params["data_ids"].clone()) {
                Ok(v) => v,
                Err(e) => return RpcResponse::err(format!("invalid data_ids: {e}")),
            };
            let count = ids.len();
            state.register_data_ids(ids).await;
            info!(count, "oracle announced data IDs");
            RpcResponse::ok(serde_json::json!({ "registered": count }))
        }

        // -- Wallet ---------------------------------------------------------

        "wallet.create" => {
            let passphrase = req.params["passphrase"].as_str().unwrap_or("").to_string();
            let _ = passphrase; // TODO: use passphrase to encrypt wallet file
            match WalletKeys::generate() {
                Err(e) => RpcResponse::err(format!("key generation failed: {e}")),
                Ok(keys) => {
                    let xmr           = keys.xmr_address_string();
                    let drk           = keys.drk_address_string();
                    let spend_key_hex = hex::encode(keys.xmr_spend_key.as_ref());

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
                        "xmr_address":   xmr,
                        "drk_address":   drk,
                        "xmr_spend_key": spend_key_hex,
                    }))
                }
            }
        }

        "wallet.import" => {
            let spend_key_hex = match req.params["spend_key"].as_str() {
                Some(s) => s.to_string(),
                None => return RpcResponse::err("missing 'spend_key' param (64-char hex XMR spend key)"),
            };
            match WalletKeys::from_spend_key(&spend_key_hex) {
                Err(e) => RpcResponse::err(format!("invalid spend key: {e}")),
                Ok(keys) => {
                    let xmr = keys.xmr_address_string();
                    let drk = keys.drk_address_string();

                    let storage = WalletStorage::new(state.data_dir());
                    if let Err(e) = storage.save(&keys, Balance::zero(), 0).await {
                        return RpcResponse::err(format!("failed to save wallet: {e}"));
                    }

                    *state.wallet().keys.write().await = Some(keys);
                    state.miner().send_cmd(MinerCmd::UpdateAddress(xmr.clone())).await;

                    RpcResponse::ok(serde_json::json!({
                        "xmr_address": xmr,
                        "drk_address": drk,
                        "imported":    true,
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_test_fixtures::bonds::draft_bond;

    async fn test_state() -> (NodeState, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let state = NodeState::new(dir.path(), true).await.unwrap();
        (state, dir) // return dir so it isn't dropped until end of test
    }

    fn req(method: &str, params: serde_json::Value) -> RpcRequest {
        RpcRequest { method: method.into(), params }
    }

    // --- bonds.list ---

    #[tokio::test]
    async fn bonds_list_empty_on_fresh_state() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bonds.list", serde_json::json!({}))).await;
        assert!(resp.error.is_none());
        let bonds = resp.result.unwrap()["bonds"].as_array().unwrap().clone();
        assert!(bonds.is_empty());
    }

    // --- bonds.get ---

    #[tokio::test]
    async fn bonds_get_unknown_id_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bonds.get", serde_json::json!({
            "id": "a".repeat(64)
        }))).await;
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn bonds_get_invalid_id_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bonds.get", serde_json::json!({
            "id": "not-hex"
        }))).await;
        assert!(resp.error.is_some());
    }

    // --- bonds.propose then bonds.list and bonds.get ---

    #[tokio::test]
    async fn propose_bond_appears_in_list_and_get() {
        let (state, _dir) = test_state().await;
        let bond = draft_bond();
        let bond_id_hex = hex::encode(bond.id.as_bytes());

        // Propose
        let propose_resp = dispatch(&state, req("bonds.propose", serde_json::json!({
            "bond": serde_json::to_value(&bond).unwrap()
        }))).await;
        assert!(propose_resp.error.is_none(), "{:?}", propose_resp.error);

        // List
        let list_resp = dispatch(&state, req("bonds.list", serde_json::json!({}))).await;
        let bonds = list_resp.result.unwrap()["bonds"].as_array().unwrap().clone();
        assert_eq!(bonds.len(), 1);

        // Get — bond.id serialises as a byte array (Digest is [u8;32]), so just
        // verify the RPC succeeded and returned the expected goal title.
        let get_resp = dispatch(&state, req("bonds.get", serde_json::json!({
            "id": bond_id_hex
        }))).await;
        assert!(get_resp.error.is_none(), "{:?}", get_resp.error);
        let result = get_resp.result.unwrap();
        assert_eq!(result["goals"][0]["title"].as_str().unwrap_or(""), "Test Goal");
    }

    // --- bonds.issue (allow_unverifiable = true) ---

    #[tokio::test]
    async fn issue_draft_bond_sets_active_state() {
        let (state, _dir) = test_state().await;
        let bond = draft_bond();
        let bond_id_hex = hex::encode(bond.id.as_bytes());

        // Store bond as Draft first
        state.insert_bond(bond).await;

        let resp = dispatch(&state, req("bonds.issue", serde_json::json!({
            "bond_id": bond_id_hex
        }))).await;
        assert!(resp.error.is_none(), "{:?}", resp.error);

        // Confirm state is now Active
        let get_resp = dispatch(&state, req("bonds.get", serde_json::json!({
            "id": bond_id_hex
        }))).await;
        let state_val = get_resp.result.unwrap()["state"].clone();
        assert_eq!(state_val.as_str().unwrap_or(""), "Active");
    }

    #[tokio::test]
    async fn issue_nonexistent_bond_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bonds.issue", serde_json::json!({
            "bond_id": "b".repeat(64)
        }))).await;
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().contains("not found"));
    }

    // --- wallet.create ---

    #[tokio::test]
    async fn wallet_create_returns_addresses_and_spend_key() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("wallet.create", serde_json::json!({}))).await;
        assert!(resp.error.is_none(), "{:?}", resp.error);
        let result = resp.result.unwrap();
        assert!(result["xmr_address"].as_str().unwrap().starts_with('5'));
        assert_eq!(result["drk_address"].as_str().unwrap().len(), 64);
        assert_eq!(result["xmr_spend_key"].as_str().unwrap().len(), 64);
    }

    #[tokio::test]
    async fn wallet_import_roundtrips() {
        let (state, _dir) = test_state().await;

        // Create
        let create_resp = dispatch(&state, req("wallet.create", serde_json::json!({}))).await;
        let result = create_resp.result.unwrap();
        let spend_key = result["xmr_spend_key"].as_str().unwrap().to_string();
        let original_drk = result["drk_address"].as_str().unwrap().to_string();

        // Import with the same spend key
        let import_resp = dispatch(&state, req("wallet.import", serde_json::json!({
            "spend_key": spend_key
        }))).await;
        assert!(import_resp.error.is_none(), "{:?}", import_resp.error);
        let imported_drk = import_resp.result.unwrap()["drk_address"].as_str().unwrap().to_string();
        assert_eq!(original_drk, imported_drk);
    }

    #[tokio::test]
    async fn wallet_import_invalid_spend_key_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("wallet.import", serde_json::json!({
            "spend_key": "not-valid-hex"
        }))).await;
        assert!(resp.error.is_some());
    }

    // --- wallet.addresses ---

    #[tokio::test]
    async fn wallet_addresses_without_wallet_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("wallet.addresses", serde_json::json!({}))).await;
        assert!(resp.error.is_some());
    }

    #[tokio::test]
    async fn wallet_addresses_after_create_returns_addresses() {
        let (state, _dir) = test_state().await;
        dispatch(&state, req("wallet.create", serde_json::json!({}))).await;
        let resp = dispatch(&state, req("wallet.addresses", serde_json::json!({}))).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert!(result["xmr"].as_str().is_some());
        assert!(result["drk"].as_str().is_some());
    }

    // --- unknown method ---

    #[tokio::test]
    async fn unknown_method_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("no_such_method", serde_json::json!({}))).await;
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().contains("unknown method"));
    }

    // --- status ---

    #[tokio::test]
    async fn status_returns_version_and_bond_count() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("status", serde_json::json!({}))).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert!(result["version"].as_str().is_some());
        assert_eq!(result["bonds"].as_u64().unwrap(), 0);
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
