//! JSON-RPC server exposed on localhost for the browser WASM frontend.
//!
//! The browser cannot reach libp2p directly, so the local node acts as a bridge:
//! WASM → HTTP JSON-RPC → node → P2P network.
//!
//! Endpoints (POST /rpc, JSON body `{"method": "...", "params": {...}}`):
//!
//!   bounties.propose              — publish a bounty proposal for community review
//!   bounties.submit_for_approval  — send bounty to listed judges for acceptance
//!   bounties.judge_accept        — judge accepts responsibility for judging
//!   bounties.judge_reject        — judge declines (with reason)
//!   bounties.judge_status        — show acceptance status for each judge
//!   bounties.revise_judges       — replace judge list (resets responses)
//!   bounties.list                 — list all known bounty series
//!   bounties.get                  — fetch a single bounty by ID
//!   bounties.issue                — lock collateral and activate a Draft bounty
//!   bounties.auction_price        — current Dutch auction ask price for a bounty
//!   bounties.buy                  — purchase N bounties at the current auction price
//!   bounties.comment              — post a question or suggestion on a proposal
//!   bounties.comments             — list comments on a bounty
//!   orders.place            — post a bid/ask
//!   orders.cancel           — cancel a resting order
//!   status                  — node version + bounty count
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
//!   judge.announce         — judge registers its supported data IDs

use anyhow::Result;
use axum::{extract::State, response::Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use tracing::info;

use nyxforge_contract::bounty_market::{process_issue_bounty, IssueBountyParams};
use nyxforge_core::bounty::{Bounty, BountyComment, BountyState, JudgeResponse};
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
            "bounties":   state.bounty_count().await,
            "version": env!("CARGO_PKG_VERSION"),
        })),

        // -- Bounties ----------------------------------------------------------

        "bounties.list" => {
            let bounties = state.list_bounties().await;
            RpcResponse::ok(serde_json::json!({ "bounties": bounties }))
        }

        "bounties.get" => {
            let id_hex = req.params["id"].as_str().unwrap_or("");
            let id_bytes = match hex::decode(id_hex) {
                Ok(b) if b.len() == 32 => {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&b);
                    arr
                }
                _ => return RpcResponse::err("invalid bounty ID: expected 32-byte hex"),
            };
            match state.get_bounty(&Digest::from_bytes(id_bytes)).await {
                Some(bounty) => RpcResponse::ok(bounty),
                None => RpcResponse::err("bounty not found"),
            }
        }

        "bounties.issue" => {
            // bounties.issue activates an existing Draft bounty by locking collateral.
            // The bounty must have already passed judge approval (state = Draft),
            // unless the node was started with --allow-unverifiable.
            let bond_id_hex = req.params["bounty_id"].as_str().unwrap_or("");
            let id_bytes = match parse_hex32(bond_id_hex) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bounty_id: expected 32-byte hex"),
            };
            let bounty_id = Digest::from_bytes(id_bytes);
            let mut bounty = match state.get_bounty(&bounty_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bounty not found"),
            };

            if !state.is_unverifiable_allowed() {
                if bounty.state != BountyState::Draft {
                    return RpcResponse::err(format!(
                        "bounty is in '{}' state — judge approval required before issuance. \
                         Use bounties.submit_for_approval first.",
                        serde_json::to_value(&bounty.state)
                            .ok().and_then(|v| v.as_str().map(str::to_owned))
                            .unwrap_or_default()
                    ));
                }
                for goal in &bounty.goals {
                    let data_id = &goal.metric.data_id;
                    if !state.is_data_id_supported(data_id).await {
                        return RpcResponse::err(format!(
                            "no judge supports data_id '{data_id}' — \
                             start an judge node that covers this data source"
                        ));
                    }
                }
            }

            let params = IssueBountyParams {
                bounty: bounty.clone(),
                collateral_proof: vec![0xde, 0xad], // stub: real ZK proof in v2
            };
            match process_issue_bounty(&params) {
                Err(e) => RpcResponse::err(e.to_string()),
                Ok(_) => {
                    bounty.state = BountyState::Active;
                    bounty.activated_at_secs = Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    );
                    bounty.bounties_remaining = bounty.total_supply;
                    let id_hex = hex::encode(bounty.id.as_bytes());
                    state.insert_bounty(bounty).await;
                    info!(bounty_id = %id_hex, "bounty issued and activated");
                    RpcResponse::ok(serde_json::json!({ "bounty_id": id_hex }))
                }
            }
        }

        "bounties.propose" => {
            let mut bounty: Bounty = match serde_json::from_value(req.params["bounty"].clone()) {
                Ok(b) => b,
                Err(e) => return RpcResponse::err(format!("invalid bounty params: {e}")),
            };
            if bounty.judge.judge_keys.is_empty() {
                return RpcResponse::err("bounty must have at least one judge key");
            }
            // Always recompute the canonical ID server-side so clients don't
            // need to replicate the blake3 derivation.
            bounty.id = Bounty::compute_id(
                &bounty.goals,
                &bounty.issuer,
                bounty.created_at_block,
                &bounty.return_address,
            );
            bounty.state = BountyState::Proposed;
            let bounty_id = hex::encode(bounty.id.as_bytes());
            state.insert_bounty(bounty).await;
            info!(bounty_id = %bounty_id, "bounty proposal published");
            RpcResponse::ok(serde_json::json!({ "bounty_id": bounty_id }))
        }

        "bounties.comment" => {
            let bond_id_hex = req.params["bounty_id"].as_str().unwrap_or("");
            let id_bytes = match hex::decode(bond_id_hex) {
                Ok(b) if b.len() == 32 => { let mut arr = [0u8; 32]; arr.copy_from_slice(&b); arr }
                _ => return RpcResponse::err("invalid bounty_id: expected 32-byte hex"),
            };
            let bounty_id = Digest::from_bytes(id_bytes);

            // Bounty must exist and be in Proposed state.
            match state.get_bounty(&bounty_id).await {
                None => return RpcResponse::err("bounty not found"),
                Some(b) if b.state != BountyState::Proposed =>
                    return RpcResponse::err("comments are only accepted on Proposed bounties"),
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

            let comment = BountyComment::new(bounty_id, author, body);
            let comment_id = hex::encode(comment.id.as_bytes());
            state.insert_comment(comment).await;
            info!(bounty_id = %bond_id_hex, comment_id = %comment_id, "comment posted");
            RpcResponse::ok(serde_json::json!({ "comment_id": comment_id }))
        }

        "bounties.comments" => {
            let bond_id_hex = req.params["bounty_id"].as_str().unwrap_or("");
            let id_bytes = match hex::decode(bond_id_hex) {
                Ok(b) if b.len() == 32 => { let mut arr = [0u8; 32]; arr.copy_from_slice(&b); arr }
                _ => return RpcResponse::err("invalid bounty_id: expected 32-byte hex"),
            };
            let bounty_id = Digest::from_bytes(id_bytes);
            let comments = state.get_comments(&bounty_id).await;
            RpcResponse::ok(serde_json::json!({ "comments": comments }))
        }

        // ------------------------------------------------------------------ //
        // Judge approval flow
        // ------------------------------------------------------------------ //

        "bounties.submit_for_approval" => {
            // Accept either a fresh bounty (params["bounty"]) or an existing
            // Proposed bounty (params["bounty_id"]).
            let mut bounty: Bounty = if req.params["bounty_id"].is_string() {
                let id_bytes = match parse_hex32(req.params["bounty_id"].as_str().unwrap_or("")) {
                    Some(b) => b,
                    None => return RpcResponse::err("invalid bounty_id"),
                };
                match state.get_bounty(&Digest::from_bytes(id_bytes)).await {
                    Some(b) => b,
                    None => return RpcResponse::err("bounty not found"),
                }
            } else {
                match serde_json::from_value(req.params["bounty"].clone()) {
                    Ok(b) => b,
                    Err(e) => return RpcResponse::err(format!("invalid bounty params: {e}")),
                }
            };

            if bounty.judge.judge_keys.is_empty() {
                return RpcResponse::err("bounty must have at least one judge key");
            }
            if matches!(bounty.state, BountyState::Active | BountyState::Redeemable | BountyState::Settled | BountyState::Expired) {
                return RpcResponse::err("bounty is already past the approval stage");
            }

            bounty.state = BountyState::PendingJudgeApproval;
            let bond_id_hex = hex::encode(bounty.id.as_bytes());
            // Clear any previous responses if re-submitted after a rejection.
            state.clear_judge_responses(&bounty.id).await;
            state.insert_bounty(bounty.clone()).await;
            info!(bounty_id = %bond_id_hex, judges = bounty.judge.judge_keys.len(),
                  "bounty submitted for judge approval");
            RpcResponse::ok(serde_json::json!({
                "bounty_id":      bond_id_hex,
                "awaiting":     bounty.judge.judge_keys.len(),
            }))
        }

        "bounties.judge_accept" => {
            let bond_id_bytes = match parse_hex32(req.params["bounty_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bounty_id"),
            };
            let bounty_id = Digest::from_bytes(bond_id_bytes);

            let bounty = match state.get_bounty(&bounty_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bounty not found"),
            };
            if bounty.state != BountyState::PendingJudgeApproval {
                return RpcResponse::err("bounty is not awaiting judge approval");
            }

            let oracle_key_bytes = match parse_hex32(req.params["judge_key"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid judge_key"),
            };
            let judge_key = PublicKey(oracle_key_bytes);

            // Verify this key is actually listed in the bounty.
            if !bounty.judge.judge_keys.contains(&judge_key) {
                return RpcResponse::err("judge_key is not listed in this bounty's JudgeSpec");
            }

            let response = JudgeResponse {
                bounty_id,
                judge_key,
                accepted: true,
                reason: None,
                responded_at: Utc::now(),
                signature: vec![], // stub
            };
            let all_accepted = state.record_judge_response(response).await;
            let bond_id_hex = hex::encode(bounty_id.as_bytes());

            if all_accepted {
                // Advance bounty to Draft.
                let mut draft_bounty = bounty;
                draft_bounty.state = BountyState::Draft;
                state.insert_bounty(draft_bounty).await;
                info!(bounty_id = %bond_id_hex, "all judges accepted — bounty advanced to Draft");
                RpcResponse::ok(serde_json::json!({
                    "bounty_id": bond_id_hex,
                    "bond_state": "Draft",
                    "message": "All judges have accepted. Bounty is now in Draft state and ready for issuance.",
                }))
            } else {
                let responses = state.get_judge_responses(&bounty_id).await;
                let pending: Vec<String> = bounty.judge.judge_keys.iter()
                    .filter(|k| !responses.iter().any(|r| r.judge_key == **k && r.accepted))
                    .map(|k| hex::encode(&k.0))
                    .collect();
                info!(bounty_id = %bond_id_hex, still_pending = pending.len(), "judge accepted");
                RpcResponse::ok(serde_json::json!({
                    "bounty_id":        bond_id_hex,
                    "bond_state":     "PendingJudgeApproval",
                    "still_pending":  pending,
                }))
            }
        }

        "bounties.judge_reject" => {
            let bond_id_bytes = match parse_hex32(req.params["bounty_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bounty_id"),
            };
            let bounty_id = Digest::from_bytes(bond_id_bytes);

            let bounty = match state.get_bounty(&bounty_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bounty not found"),
            };
            if bounty.state != BountyState::PendingJudgeApproval {
                return RpcResponse::err("bounty is not awaiting judge approval");
            }

            let oracle_key_bytes = match parse_hex32(req.params["judge_key"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid judge_key"),
            };
            let judge_key = PublicKey(oracle_key_bytes);
            if !bounty.judge.judge_keys.contains(&judge_key) {
                return RpcResponse::err("judge_key is not listed in this bounty's JudgeSpec");
            }

            let reason = req.params["reason"].as_str()
                .filter(|s| !s.trim().is_empty())
                .map(str::to_owned);

            let response = JudgeResponse {
                bounty_id,
                judge_key,
                accepted: false,
                reason: reason.clone(),
                responded_at: Utc::now(),
                signature: vec![],
            };
            state.record_judge_response(response).await;
            let bond_id_hex = hex::encode(bounty_id.as_bytes());
            info!(bounty_id = %bond_id_hex, ?reason, "judge rejected bounty");
            RpcResponse::ok(serde_json::json!({
                "bounty_id": bond_id_hex,
                "message": "Rejection recorded. Issuer must revise judge list or threshold and re-submit.",
            }))
        }

        "bounties.judge_status" => {
            let bond_id_bytes = match parse_hex32(req.params["bounty_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bounty_id"),
            };
            let bounty_id = Digest::from_bytes(bond_id_bytes);
            let bounty = match state.get_bounty(&bounty_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bounty not found"),
            };
            let responses = state.get_judge_responses(&bounty_id).await;

            let status: Vec<serde_json::Value> = bounty.judge.judge_keys.iter().map(|key| {
                let key_hex = hex::encode(&key.0);
                match responses.iter().find(|r| r.judge_key == *key) {
                    None => serde_json::json!({
                        "judge": key_hex, "status": "pending"
                    }),
                    Some(r) if r.accepted => serde_json::json!({
                        "judge": key_hex, "status": "accepted",
                        "responded_at": r.responded_at,
                    }),
                    Some(r) => serde_json::json!({
                        "judge": key_hex, "status": "rejected",
                        "reason": r.reason,
                        "responded_at": r.responded_at,
                    }),
                }
            }).collect();

            RpcResponse::ok(serde_json::json!({
                "bounty_id":    hex::encode(bounty_id.as_bytes()),
                "bond_state": bounty.state,
                "judges":    status,
            }))
        }

        "bounties.revise_judges" => {
            // Allows the issuer to replace the judge list on a
            // PendingJudgeApproval bounty after one or more rejections.
            // Clears all existing responses so judges must re-accept.
            let bond_id_bytes = match parse_hex32(req.params["bounty_id"].as_str().unwrap_or("")) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bounty_id"),
            };
            let bounty_id = Digest::from_bytes(bond_id_bytes);
            let mut bounty = match state.get_bounty(&bounty_id).await {
                Some(b) => b,
                None => return RpcResponse::err("bounty not found"),
            };
            if bounty.state != BountyState::PendingJudgeApproval {
                return RpcResponse::err("bounty must be in PendingJudgeApproval state to revise judges");
            }

            let keys_raw = match req.params["judge_keys"].as_array() {
                Some(a) => a.clone(),
                None => return RpcResponse::err("missing 'judge_keys' array"),
            };
            let mut new_keys = Vec::new();
            for v in &keys_raw {
                let hex = v.as_str().unwrap_or("");
                match parse_hex32(hex) {
                    Some(b) => new_keys.push(PublicKey(b)),
                    None => return RpcResponse::err(format!("invalid judge key: '{hex}'")),
                }
            }
            if new_keys.is_empty() {
                return RpcResponse::err("bounty must have at least one judge key");
            }

            bounty.judge.judge_keys = new_keys;
            state.clear_judge_responses(&bounty_id).await;
            state.insert_bounty(bounty).await;
            info!(bounty_id = %hex::encode(bounty_id.as_bytes()), "judge list revised; responses cleared");
            RpcResponse::ok(serde_json::json!({
                "bounty_id": hex::encode(bounty_id.as_bytes()),
                "message": "Judge list updated. All previous responses cleared. Judges must re-accept.",
            }))
        }

        "bounties.auction_price" => {
            let bond_id_hex = req.params["bounty_id"].as_str().unwrap_or("");
            let id_bytes = match parse_hex32(bond_id_hex) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bounty_id: expected 32-byte hex"),
            };
            let bounty = match state.get_bounty(&Digest::from_bytes(id_bytes)).await {
                Some(b) => b,
                None => return RpcResponse::err("bounty not found"),
            };
            let elapsed = bounty.activated_at_secs.map(|t| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now.saturating_sub(t)
            }).unwrap_or(0);
            let price = bounty.auction.current_price(elapsed);
            RpcResponse::ok(serde_json::json!({
                "bounty_id":       bond_id_hex,
                "price_micro_drk": price.0,
            }))
        }

        "bounties.buy" => {
            let bond_id_hex = req.params["bounty_id"].as_str().unwrap_or("");
            let quantity = req.params["quantity"].as_u64().unwrap_or(0);
            if quantity == 0 {
                return RpcResponse::err("quantity must be > 0");
            }
            let id_bytes = match parse_hex32(bond_id_hex) {
                Some(b) => b,
                None => return RpcResponse::err("invalid bounty_id: expected 32-byte hex"),
            };
            let mut bounty = match state.get_bounty(&Digest::from_bytes(id_bytes)).await {
                Some(b) => b,
                None => return RpcResponse::err("bounty not found"),
            };
            if bounty.state != BountyState::Active {
                return RpcResponse::err("bounty is not active");
            }
            if bounty.bounties_remaining < quantity {
                return RpcResponse::err(format!(
                    "only {} bounty(s) remaining", bounty.bounties_remaining
                ));
            }
            let elapsed = bounty.activated_at_secs.map(|t| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now.saturating_sub(t)
            }).unwrap_or(0);
            let price = bounty.auction.current_price(elapsed);
            bounty.bounties_remaining = bounty.bounties_remaining.saturating_sub(quantity);
            state.insert_bounty(bounty).await;
            RpcResponse::ok(serde_json::json!({
                "purchased":       quantity,
                "price_micro_drk": price.0,
            }))
        }

        "judge.announce" => {
            let ids: Vec<String> = match serde_json::from_value(req.params["data_ids"].clone()) {
                Ok(v) => v,
                Err(e) => return RpcResponse::err(format!("invalid data_ids: {e}")),
            };
            let count = ids.len();
            state.register_data_ids(ids).await;
            info!(count, "judge announced data IDs");
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
    use nyxforge_test_fixtures::bounties::draft_bounty;

    async fn test_state() -> (NodeState, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let state = NodeState::new(dir.path(), true).await.unwrap();
        (state, dir) // return dir so it isn't dropped until end of test
    }

    fn req(method: &str, params: serde_json::Value) -> RpcRequest {
        RpcRequest { method: method.into(), params }
    }

    // --- bounties.list ---

    #[tokio::test]
    async fn bonds_list_empty_on_fresh_state() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bounties.list", serde_json::json!({}))).await;
        assert!(resp.error.is_none());
        let bounties = resp.result.unwrap()["bounties"].as_array().unwrap().clone();
        assert!(bounties.is_empty());
    }

    // --- bounties.get ---

    #[tokio::test]
    async fn bonds_get_unknown_id_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bounties.get", serde_json::json!({
            "id": "a".repeat(64)
        }))).await;
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn bonds_get_invalid_id_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bounties.get", serde_json::json!({
            "id": "not-hex"
        }))).await;
        assert!(resp.error.is_some());
    }

    // --- bounties.propose then bounties.list and bounties.get ---

    #[tokio::test]
    async fn propose_bond_appears_in_list_and_get() {
        let (state, _dir) = test_state().await;
        let bounty = draft_bounty();
        let bond_id_hex = hex::encode(bounty.id.as_bytes());

        // Propose
        let propose_resp = dispatch(&state, req("bounties.propose", serde_json::json!({
            "bounty": serde_json::to_value(&bounty).unwrap()
        }))).await;
        assert!(propose_resp.error.is_none(), "{:?}", propose_resp.error);

        // List
        let list_resp = dispatch(&state, req("bounties.list", serde_json::json!({}))).await;
        let bounties = list_resp.result.unwrap()["bounties"].as_array().unwrap().clone();
        assert_eq!(bounties.len(), 1);

        // Get — bounty.id serialises as a byte array (Digest is [u8;32]), so just
        // verify the RPC succeeded and returned the expected goal title.
        let get_resp = dispatch(&state, req("bounties.get", serde_json::json!({
            "id": bond_id_hex
        }))).await;
        assert!(get_resp.error.is_none(), "{:?}", get_resp.error);
        let result = get_resp.result.unwrap();
        assert_eq!(result["goals"][0]["title"].as_str().unwrap_or(""), "Test Goal");
    }

    // --- bounties.issue (allow_unverifiable = true) ---

    #[tokio::test]
    async fn issue_draft_bond_sets_active_state() {
        let (state, _dir) = test_state().await;
        let bounty = draft_bounty();
        let bond_id_hex = hex::encode(bounty.id.as_bytes());

        // Store bounty as Draft first
        state.insert_bounty(bounty).await;

        let resp = dispatch(&state, req("bounties.issue", serde_json::json!({
            "bounty_id": bond_id_hex
        }))).await;
        assert!(resp.error.is_none(), "{:?}", resp.error);

        // Confirm state is now Active
        let get_resp = dispatch(&state, req("bounties.get", serde_json::json!({
            "id": bond_id_hex
        }))).await;
        let state_val = get_resp.result.unwrap()["state"].clone();
        assert_eq!(state_val.as_str().unwrap_or(""), "Active");
    }

    #[tokio::test]
    async fn issue_nonexistent_bond_returns_error() {
        let (state, _dir) = test_state().await;
        let resp = dispatch(&state, req("bounties.issue", serde_json::json!({
            "bounty_id": "b".repeat(64)
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
        assert_eq!(result["bounties"].as_u64().unwrap(), 0);
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
