//! Miner control task: bridges NodeState ↔ nyxforge-miner.

use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info};

use nyxforge_miner::{MinerCmd, MinerConfig, MinerHandle};

use crate::state::NodeState;

/// Stats polling interval.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Run the miner control task.
///
/// Initialises the miner with `config`, wires its command channel into
/// `NodeState`, and polls stats every 5 s so the RPC layer can serve them.
pub async fn run(state: NodeState, config: MinerConfig, start_immediately: bool) -> Result<()> {
    info!(threads = config.threads, p2pool = %config.p2pool_url, "miner task starting");

    // Store config in state.
    *state.miner().config.write().await = Some(config.clone());

    // Spawn the miner handle.
    let handle: MinerHandle = nyxforge_miner::worker::spawn(config);

    // Wire the control channel into NodeState so RPC can send commands.
    *state.miner().ctl_tx.write().await = Some(handle.cmd_tx.clone());

    if start_immediately {
        info!("--mine-on-start: sending Start command");
        let _ = handle.cmd_tx.send(MinerCmd::Start).await;
    }

    // Stats polling loop.
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;
        let stats = handle.stats.snapshot().await;
        *state.miner().stats.write().await = stats;
    }
}
