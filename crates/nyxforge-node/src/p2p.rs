//! P2P swarm: libp2p gossipsub for bond/order propagation + Kademlia DHT.
//!
//! Topics:
//!   - `nyxforge/bonds/1`   — new bond series announcements
//!   - `nyxforge/orders/1`  — order book updates
//!   - `nyxforge/trades/1`  — executed trade records
//!   - `nyxforge/oracles/1` — oracle attestations
//!   - `nyxforge/quorum/1`  — quorum results and state transitions

use anyhow::Result;
use tracing::{info, warn};

use crate::state::NodeState;

/// Gossipsub topic names.
pub mod topics {
    pub const BONDS:   &str = "nyxforge/bonds/1";
    pub const ORDERS:  &str = "nyxforge/orders/1";
    pub const TRADES:  &str = "nyxforge/trades/1";
    pub const ORACLES: &str = "nyxforge/oracles/1";
    pub const QUORUM:  &str = "nyxforge/quorum/1";
}

/// Messages that can be gossiped on the network.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GossipMessage {
    NewBond(Box<nyxforge_core::bond::Bond>),
    NewOrder(Box<nyxforge_core::market::Order>),
    NewTrade(Box<nyxforge_core::market::Trade>),
    OracleAttestation(Box<nyxforge_core::oracle_spec::OracleAttestation>),
    QuorumResult(Box<nyxforge_core::oracle_spec::QuorumResult>),
}

/// Dispatch an incoming gossip message to the appropriate handler.
async fn handle_message(msg: GossipMessage, state: &NodeState) {
    match msg {
        GossipMessage::NewBond(bond) => {
            info!(id = ?bond.id, "received new bond");
            state.insert_bond(*bond).await;
        }
        GossipMessage::NewOrder(order) => {
            info!(id = ?order.id, side = ?order.side, "received order");
            // TODO: insert into order book and attempt matching.
        }
        GossipMessage::NewTrade(trade) => {
            info!(id = ?trade.id, "received trade");
            // TODO: mark nullifiers spent.
        }
        GossipMessage::OracleAttestation(att) => {
            info!(bond_id = ?att.bond_id, goal_met = att.goal_met, "oracle attestation");
            // TODO: accumulate attestations, check quorum.
        }
        GossipMessage::QuorumResult(q) => {
            info!(bond_id = ?q.bond_id, goal_met = q.goal_met, "quorum finalised");
            // TODO: update bond state in contract.
        }
    }
}

/// Run the libp2p swarm.  This is a skeleton; full libp2p wiring goes here.
pub async fn run_swarm(
    state: NodeState,
    listen_addr: &str,
    bootstrap_peers: &[String],
) -> Result<()> {
    info!(%listen_addr, "starting P2P swarm");

    if bootstrap_peers.is_empty() {
        warn!("no bootstrap peers — operating in isolated mode");
    } else {
        for peer in bootstrap_peers {
            info!(%peer, "bootstrap peer");
        }
    }

    // TODO: construct libp2p swarm with:
    //   - Noise encryption (XX handshake)
    //   - Yamux multiplexing
    //   - Gossipsub for message propagation
    //   - Kademlia DHT for peer/content discovery
    //   - Identify protocol
    //
    // Then subscribe to all topics, deserialise incoming messages,
    // and call handle_message(msg, &state).await.

    // Placeholder: keep the task alive.
    let _ = state.bond_count().await;
    futures::future::pending::<()>().await;
    Ok(())
}
