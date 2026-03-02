//! Background wallet task: load wallet from disk, scan XMR + DRK chains.

use std::time::Duration;

use anyhow::Result;
use tracing::{debug, error, info, warn};

use nyxforge_miner::MinerCmd;
use nyxforge_wallet::storage::WalletStorage;
use nyxforge_wallet::xmr::remote::RemoteMonerod;
use nyxforge_wallet::xmr::source::MoneroSource;
use nyxforge_wallet::Balance;

/// Scan interval between balance refreshes.
const SCAN_INTERVAL: Duration = Duration::from_secs(60);

/// Run the wallet background task.
///
/// On startup:
///   1. If a wallet file exists, load it and push the XMR address to the miner.
///   2. Then poll every 60 s for new XMR outputs.
///   3. If no wallet exists yet, wait for `wallet.create` (called via RPC)
///      which will populate state directly and push the address itself.
pub async fn run(state: crate::state::NodeState, xmr_node_url: String) -> Result<()> {
    info!(xmr_node = %xmr_node_url, "wallet manager starting");
    let source = RemoteMonerod::new(&xmr_node_url);
    let storage = WalletStorage::new(state.data_dir());

    // Attempt to restore a previously created wallet.
    if storage.exists() {
        match storage.load().await {
            Ok((keys, balance, height)) => {
                let xmr_address = keys.xmr_address_string();
                info!(%xmr_address, height, "loaded wallet from disk");

                // Restore state.
                *state.wallet().keys.write().await = Some(keys);
                *state.wallet().balance.write().await = balance;
                state.wallet().set_scan_height(height);

                // Tell the miner which address to use.
                push_address_to_miner(&state, xmr_address).await;
            }
            Err(e) => warn!("failed to load wallet from disk: {e:#}"),
        }
    } else {
        info!("no wallet file found — waiting for wallet.create");
    }

    // Scan loop.
    loop {
        tokio::time::sleep(SCAN_INTERVAL).await;

        let has_keys = state.wallet().keys.read().await.is_some();
        if !has_keys {
            debug!("no wallet yet — waiting");
            continue;
        }

        if let Err(e) = scan_once(&state, &source, &storage).await {
            error!("wallet scan error: {e:#}");
        }
    }
}

/// Send the XMR address to the miner control task.
pub async fn push_address_to_miner(state: &crate::state::NodeState, address: String) {
    if !state.miner().send_cmd(MinerCmd::UpdateAddress(address)).await {
        // Miner task isn't up yet; it will pull the address from state on its
        // first scan anyway, so this is not fatal.
        debug!("miner not ready yet when pushing address");
    }
}

async fn scan_once(
    state: &crate::state::NodeState,
    source: &RemoteMonerod,
    storage: &WalletStorage,
) -> Result<()> {
    let height = source.get_height().await?;
    let last = state.wallet().scan_height();

    if height <= last {
        debug!(height, "no new blocks");
        return Ok(());
    }

    info!(from = last, to = height, "scanning XMR outputs");

    // TODO: call scanner::scan_range and accumulate OwnedOutputs into balance.
    state.wallet().set_scan_height(height);

    let balance = Balance::zero(); // placeholder until scanner is implemented
    *state.wallet().balance.write().await = balance;

    // Persist updated scan height.
    let guard = state.wallet().keys.read().await;
    if let Some(keys) = guard.as_ref() {
        if let Err(e) = storage.save(keys, balance, height).await {
            error!("failed to persist wallet: {e:#}");
        }
    }

    Ok(())
}
