//! Hashrate tracker — rolling 60-second average and share counters.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

const WINDOW: Duration = Duration::from_secs(60);

/// Public miner statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MinerStats {
    /// True while the miner loop is running.
    pub running: bool,
    /// Rolling 60-second average hashrate in H/s.
    pub hashrate_h_s: f64,
    /// Total shares submitted this session.
    pub shares_found: u64,
    /// Estimated pending XMR payout from P2Pool (picomonero).
    pub xmr_pending_pico: u64,
}

/// Shared stats object updated by the mining threads.
pub struct StatsTracker {
    inner: RwLock<TrackerInner>,
}

struct TrackerInner {
    /// Timestamps of individual hashes computed (used for rolling average).
    samples: VecDeque<Instant>,
    shares_found: u64,
    xmr_pending_pico: u64,
    running: bool,
}

impl StatsTracker {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(TrackerInner {
                samples: VecDeque::new(),
                shares_found: 0,
                xmr_pending_pico: 0,
                running: false,
            }),
        }
    }

    /// Record `count` hashes having been computed.
    pub async fn record_hashes(&self, count: u64) {
        let now = Instant::now();
        let mut inner = self.inner.write().await;
        for _ in 0..count {
            inner.samples.push_back(now);
        }
        // Prune samples older than WINDOW.
        let cutoff = now - WINDOW;
        while inner.samples.front().map_or(false, |&t| t < cutoff) {
            inner.samples.pop_front();
        }
    }

    /// Record a found share.
    pub async fn record_share(&self) {
        let mut inner = self.inner.write().await;
        inner.shares_found += 1;
    }

    pub async fn set_running(&self, running: bool) {
        self.inner.write().await.running = running;
    }

    pub async fn set_xmr_pending(&self, pico: u64) {
        self.inner.write().await.xmr_pending_pico = pico;
    }

    /// Snapshot current statistics.
    pub async fn snapshot(&self) -> MinerStats {
        let inner = self.inner.read().await;
        let hashrate = inner.samples.len() as f64 / WINDOW.as_secs_f64();
        MinerStats {
            running: inner.running,
            hashrate_h_s: hashrate,
            shares_found: inner.shares_found,
            xmr_pending_pico: inner.xmr_pending_pico,
        }
    }
}

impl Default for StatsTracker {
    fn default() -> Self {
        Self::new()
    }
}
