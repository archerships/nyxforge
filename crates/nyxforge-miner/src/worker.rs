//! Mining loop: get job → hash → submit share when hash < target.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, watch};
use tracing::{error, info, warn};

use crate::hasher::{meets_target, RandomXHasher};
use crate::p2pool::{MinerConfig, Share, StratumClient};
use crate::stats::StatsTracker;

/// Commands sent to the miner control task.
#[derive(Debug)]
pub enum MinerCmd {
    Start,
    Stop,
    SetThreads(usize),
    /// Set the XMR payout address.  If a `Start` was received before the
    /// address arrived, mining begins automatically.  If already running,
    /// the current session is restarted with the new address.
    UpdateAddress(String),
}

/// Handle to the running miner, returned from [`spawn`].
pub struct MinerHandle {
    pub cmd_tx: mpsc::Sender<MinerCmd>,
    pub stats: Arc<StatsTracker>,
}

/// Spawn the miner control task.  Returns immediately; mining starts when
/// `MinerCmd::Start` is sent on the returned handle.
pub fn spawn(config: MinerConfig) -> MinerHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel::<MinerCmd>(8);
    let stats = Arc::new(StatsTracker::new());
    let stats_clone = stats.clone();

    tokio::spawn(control_loop(config, cmd_rx, stats_clone));

    MinerHandle { cmd_tx, stats }
}

// ---------------------------------------------------------------------------
// Control loop — manages start/stop and thread count changes
// ---------------------------------------------------------------------------

async fn control_loop(
    mut config: MinerConfig,
    mut cmd_rx: mpsc::Receiver<MinerCmd>,
    stats: Arc<StatsTracker>,
) {
    let mut stop_tx: Option<watch::Sender<bool>> = None;
    // True if Start was requested before an address was available.
    let mut pending_start = false;

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            MinerCmd::Start => {
                if config.xmr_address.is_empty() {
                    info!("no XMR address yet — mining will start automatically after wallet.create");
                    pending_start = true;
                    continue;
                }
                if stop_tx.is_some() {
                    info!("miner already running");
                    continue;
                }
                stop_tx = Some(do_start(&config, &stats).await);
            }
            MinerCmd::Stop => {
                pending_start = false;
                do_stop(&mut stop_tx, &stats).await;
            }
            MinerCmd::SetThreads(n) => {
                config.threads = n;
                info!(threads = n, "thread count updated (takes effect on next start)");
            }
            MinerCmd::UpdateAddress(addr) => {
                info!(address = %addr, "XMR payout address updated");
                config.xmr_address = addr;

                if stop_tx.is_some() {
                    // Restart so P2Pool picks up the new address in the login.
                    info!("restarting miner with new address");
                    do_stop(&mut stop_tx, &stats).await;
                    stop_tx = Some(do_start(&config, &stats).await);
                } else if pending_start {
                    // Start was deferred waiting for this address.
                    info!("address received — starting deferred mining session");
                    pending_start = false;
                    stop_tx = Some(do_start(&config, &stats).await);
                }
            }
        }
    }
}

async fn do_start(config: &MinerConfig, stats: &Arc<StatsTracker>) -> watch::Sender<bool> {
    info!(threads = config.threads, p2pool = %config.p2pool_url,
          address = %config.xmr_address, "starting miner");
    let (tx, rx) = watch::channel(false);
    tokio::spawn(mining_session(config.clone(), rx, stats.clone()));
    stats.set_running(true).await;
    tx
}

async fn do_stop(stop_tx: &mut Option<watch::Sender<bool>>, stats: &Arc<StatsTracker>) {
    if let Some(tx) = stop_tx.take() {
        info!("stopping miner");
        let _ = tx.send(true);
        stats.set_running(false).await;
    }
}

// ---------------------------------------------------------------------------
// Mining session — one session per Start command
// ---------------------------------------------------------------------------

async fn mining_session(
    config: MinerConfig,
    stop_rx: watch::Receiver<bool>,
    stats: Arc<StatsTracker>,
) {
    if let Err(e) = run_session(&config, stop_rx, &stats).await {
        error!("mining session error: {e:#}");
    }
    stats.set_running(false).await;
}

async fn run_session(
    config: &MinerConfig,
    mut stop_rx: watch::Receiver<bool>,
    stats: &Arc<StatsTracker>,
) -> Result<()> {
    let (mut client, mut current_job) = StratumClient::connect(config).await?;

    loop {
        if *stop_rx.borrow() {
            info!("miner stop signal received");
            return Ok(());
        }

        let job = current_job.clone();
        info!(height = job.height, job_id = %job.job_id, "new job");

        // Parse seed hash for RandomX VM.
        let seed_bytes = decode_hex_vec(&job.seed_hash)?;

        // Run hashing on a blocking thread to avoid stalling the async runtime.
        let stats_clone = stats.clone();
        let worker_id = client.worker_id().to_string();
        let config_threads = config.threads;

        let share_result = tokio::task::spawn_blocking(move || {
            hash_loop(&seed_bytes, &job, config_threads, &worker_id, &stats_clone)
        })
        .await??;

        // Submit share if found.
        if let Some(share) = share_result {
            info!(job_id = %share.job_id, nonce = %share.nonce, "share found!");
            stats.record_share().await;
            if let Err(e) = client.submit_share(&share).await {
                warn!("share submission error: {e}");
            }
        }

        // Wait for the next job from the server.
        tokio::select! {
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() { return Ok(()); }
            }
            result = client.next_job() => {
                match result {
                    Ok(j)  => current_job = j,
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CPU-bound hash loop (runs on spawn_blocking thread pool)
// ---------------------------------------------------------------------------

fn hash_loop(
    seed_bytes: &[u8],
    job: &crate::p2pool::Job,
    _threads: usize,
    worker_id: &str,
    stats: &Arc<StatsTracker>,
) -> Result<Option<Share>> {
    let mut hasher = RandomXHasher::new(seed_bytes)?;
    let mut blob = decode_hex_vec(&job.blob)?;

    // Nonce is 4 bytes at offset 39 in the blob (Monero block header format).
    const NONCE_OFFSET: usize = 39;
    if blob.len() < NONCE_OFFSET + 4 {
        return Err(anyhow::anyhow!("blob too short: {} bytes", blob.len()));
    }

    // Try up to 2^20 nonces before yielding back to the async loop.
    // This keeps each blocking slice to ~1s at typical hashrates.
    const BATCH: u32 = 1 << 20;

    // Use a tokio::sync::mpsc-friendly approach: just use a simple runtime handle.
    let rt = tokio::runtime::Handle::current();

    for nonce in 0u32..BATCH {
        let n_bytes = nonce.to_le_bytes();
        blob[NONCE_OFFSET..NONCE_OFFSET + 4].copy_from_slice(&n_bytes);

        let hash = hasher.hash(&blob)?;
        rt.block_on(stats.record_hashes(1));

        if meets_target(&hash, &job.target) {
            let share = Share {
                worker_id: worker_id.to_string(),
                job_id: job.job_id.clone(),
                nonce: encode_hex(&n_bytes),
                result: encode_hex(&hash),
            };
            return Ok(Some(share));
        }
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn decode_hex_vec(s: &str) -> Result<Vec<u8>> {
    let s = s.trim_start_matches("0x");
    if s.len() % 2 != 0 {
        return Err(anyhow::anyhow!("odd-length hex string"));
    }
    s.as_bytes()
        .chunks(2)
        .map(|c| {
            u8::from_str_radix(std::str::from_utf8(c)?, 16)
                .map_err(|e| anyhow::anyhow!("hex decode: {e}"))
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
