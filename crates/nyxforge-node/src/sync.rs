//! State synchronisation: catch up a new node from peers via request-response.
//!
//! On startup the node:
//!   1. Connects to bootstrap peers.
//!   2. Requests the latest bond list + note tree root.
//!   3. Streams bond and order history from the most-synced peer.
//!   4. Verifies all ZK proofs before applying state.

use anyhow::Result;
use tracing::{info, warn};

use crate::state::NodeState;

/// Request types for the sync protocol.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum SyncRequest {
    /// Ask for all bond series known to the peer.
    BondList,
    /// Ask for all resting orders for a bond series.
    OrderBook { bond_id: nyxforge_core::bond::BondId },
    /// Ask for the nullifier tree root (for light client verification).
    NullifierRoot,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum SyncResponse {
    BondList(Vec<nyxforge_core::bond::Bond>),
    OrderBook(Vec<nyxforge_core::market::Order>),
    NullifierRoot(nyxforge_core::types::Digest),
    NotFound,
}

/// Perform initial state sync against bootstrap peers.
pub async fn initial_sync(state: &NodeState, _bootstrap_peers: &[String]) -> Result<()> {
    if _bootstrap_peers.is_empty() {
        warn!("no bootstrap peers; skipping sync");
        return Ok(());
    }

    info!("starting initial sync");

    // TODO:
    //   1. Open request-response stream to each bootstrap peer.
    //   2. Send SyncRequest::BondList, receive SyncResponse::BondList.
    //   3. For each bond, send SyncRequest::OrderBook.
    //   4. Verify and apply all received data.

    info!("sync complete; bonds={}", state.bond_count().await);
    Ok(())
}
