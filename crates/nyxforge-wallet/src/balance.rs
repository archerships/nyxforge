//! Unified Balance across XMR and DRK.

use nyxforge_core::types::Amount as DrkAmount;
use serde::{Deserialize, Serialize};

/// Combined wallet balance.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Balance {
    /// XMR with at least 10 confirmations (picomonero; 1 XMR = 1e12).
    pub xmr_confirmed: u64,
    /// XMR in mempool / < 10 confirmations (picomonero).
    pub xmr_unconfirmed: u64,
    /// DRK balance in μDRK (1 DRK = 1_000_000 μDRK).
    pub drk: DrkAmount,
}

impl Balance {
    pub fn zero() -> Self {
        Self {
            xmr_confirmed: 0,
            xmr_unconfirmed: 0,
            drk: DrkAmount::ZERO,
        }
    }

    /// Human-readable XMR (confirmed), e.g. "1.234560".
    pub fn xmr_confirmed_display(&self) -> String {
        format!("{:.6}", self.xmr_confirmed as f64 / 1_000_000_000_000.0)
    }

    /// Human-readable DRK, e.g. "1.250000".
    pub fn drk_display(&self) -> String {
        format!("{:.6}", self.drk.0 as f64 / 1_000_000.0)
    }
}
