//! P2P swarm: libp2p gossipsub for bounty/order propagation + Kademlia DHT.
//!
//! Topics:
//!   - `nyxforge/bounties/1`   — new bounty series announcements
//!   - `nyxforge/orders/1`  — order book updates
//!   - `nyxforge/trades/1`  — executed trade records
//!   - `nyxforge/judges/1` — judge attestations
//!   - `nyxforge/quorum/1`  — quorum results and state transitions

use anyhow::Result;
use tracing::{info, warn};

use crate::state::NodeState;

/// Gossipsub topic names.
pub mod topics {
    pub const BOUNTIES: &str = "nyxforge/bounties/1";
    pub const ORDERS:   &str = "nyxforge/orders/1";
    pub const TRADES:   &str = "nyxforge/trades/1";
    pub const JUDGES:   &str = "nyxforge/judges/1";
    pub const QUORUM:   &str = "nyxforge/quorum/1";
}

/// Messages that can be gossiped on the network.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GossipMessage {
    NewBounty(Box<nyxforge_core::bounty::Bounty>),
    NewOrder(Box<nyxforge_core::market::Order>),
    NewTrade(Box<nyxforge_core::market::Trade>),
    JudgeAttestation(Box<nyxforge_core::judge_spec::JudgeAttestation>),
    QuorumResult(Box<nyxforge_core::judge_spec::QuorumResult>),
}

/// Dispatch an incoming gossip message to the appropriate handler.
async fn handle_message(msg: GossipMessage, state: &NodeState) {
    match msg {
        GossipMessage::NewBounty(bounty) => {
            info!(id = ?bounty.id, "received new bounty");
            state.insert_bounty(*bounty).await;
        }
        GossipMessage::NewOrder(order) => {
            info!(id = ?order.id, side = ?order.side, "received order");
            // TODO: insert into order book and attempt matching.
        }
        GossipMessage::NewTrade(trade) => {
            info!(id = ?trade.id, "received trade");
            // TODO: mark nullifiers spent.
        }
        GossipMessage::JudgeAttestation(att) => {
            info!(bounty_id = ?att.bounty_id, goal_met = att.goal_met, "judge attestation");
            // TODO: accumulate attestations, check quorum.
        }
        GossipMessage::QuorumResult(q) => {
            info!(bounty_id = ?q.bounty_id, goal_met = q.goal_met, "quorum finalised");
            // TODO: update bounty state in contract.
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
    let _ = state.bounty_count().await;
    std::future::pending::<()>().await;
    Ok(())
}
