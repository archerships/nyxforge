//! Order book contract: anonymous DEX for bond trading.
//!
//! Trades are settled atomically via ZK transfer proofs.
//!
//! # Instructions
//!
//! | Instruction   | Description                                           |
//! |---------------|-------------------------------------------------------|
//! | `PlaceOrder`  | Post a bid or ask with a ZK ownership commitment     |
//! | `CancelOrder` | Remove a resting order (prover must know commitment)  |
//! | `FillOrder`   | Execute a matching pair, produce transfer proofs      |

use nyxforge_core::market::{Order, Trade};
use nyxforge_core::types::Digest;
use nyxforge_zk::transfer::TransferProof;
use serde::{Deserialize, Serialize};

use crate::ContractResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrderParams {
    pub order: Order,
    /// ZK proof that the maker owns the bond notes (ask) or base tokens (bid)
    /// being committed to this order.
    pub ownership_proof: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FillOrderParams {
    pub maker_order_id: Digest,
    pub taker_order_id: Digest,
    /// ZK transfer proof moving bond notes from maker to taker.
    pub bond_transfer: TransferProof,
    /// ZK transfer proof moving base tokens from taker to maker.
    pub payment_transfer: TransferProof,
}

/// Validate and record a new order.
pub fn process_place_order(params: &PlaceOrderParams) -> ContractResult<Digest> {
    if params.order.quantity == 0 {
        return Err(anyhow::anyhow!("order quantity must be > 0").into());
    }
    if params.order.price.0 == 0 {
        return Err(anyhow::anyhow!("order price must be > 0").into());
    }
    if params.ownership_proof.is_empty() {
        return Err(anyhow::anyhow!("ownership proof required").into());
    }

    // TODO: persist order to DarkFi contract state trie.
    tracing::info!(order_id = ?params.order.id, side = ?params.order.side, "order placed");
    Ok(params.order.id)
}

/// Execute two matching orders atomically.
pub fn process_fill_order(params: &FillOrderParams) -> ContractResult<Trade> {
    // Verify bond transfer: nullifier must not be spent, proof must verify.
    params.bond_transfer.verify()
        .map_err(|e| anyhow::anyhow!("bond transfer: {e}"))?;

    // Verify payment transfer.
    params.payment_transfer.verify()
        .map_err(|e| anyhow::anyhow!("payment transfer: {e}"))?;

    // TODO: check nullifiers against contract state (double-spend prevention).
    // TODO: record new commitments in the note tree.

    let trade = Trade {
        id: {
            let mut h = blake3::Hasher::new();
            h.update(params.maker_order_id.as_bytes());
            h.update(params.taker_order_id.as_bytes());
            nyxforge_core::types::Digest::from(h.finalize())
        },
        bond_id:     params.bond_transfer.bond_id,
        price:       nyxforge_core::types::Amount::ZERO, // filled from order record
        quantity:    0,                                   // filled from order record
        executed_at: chrono::Utc::now(),
        nullifiers:  vec![
            params.bond_transfer.nullifier,
            params.payment_transfer.nullifier,
        ],
    };

    tracing::info!(trade_id = ?trade.id, "order filled");
    Ok(trade)
}
