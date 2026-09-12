//! DRK note wallet: scan for owned notes, track balance, create spends.
//!
//! DarkFi uses anonymous Sapling-style notes with ZK proofs.
//! Full integration requires darkfi-sdk; this scaffold defines the interface.

use anyhow::{anyhow, Result};
use nyxforge_core::types::Amount as DrkAmount;
use tracing::info;

use crate::keys::WalletKeys;

/// A received DRK note owned by this wallet.
#[derive(Debug, Clone)]
pub struct DrkNote {
    pub amount: DrkAmount,
    pub nullifier: [u8; 32],
    pub spent: bool,
}

/// Scan the DarkFi chain for notes belonging to this wallet.
///
/// Returns owned notes and the latest scanned block.
pub async fn scan_notes(
    keys: &WalletKeys,
    from_block: u64,
) -> Result<(Vec<DrkNote>, u64)> {
    info!(from = from_block, drk_address = %keys.drk_address_string(), "scanning DRK notes");
    // TODO: integrate darkfi-sdk note scanning when APIs stabilise
    Ok((Vec::new(), from_block))
}

/// Sum confirmed DRK balance from unspent notes.
pub fn sum_balance(notes: &[DrkNote]) -> DrkAmount {
    notes
        .iter()
        .filter(|n| !n.spent)
        .fold(DrkAmount::ZERO, |acc, n| {
            acc.checked_add(n.amount).unwrap_or(acc)
        })
}

/// Build and submit a DRK transfer note.
pub async fn send_drk(
    keys: &WalletKeys,
    to_address: &str,
    amount: DrkAmount,
) -> Result<String> {
    info!(to = %to_address, amount = ?amount, "building DRK transfer");
    // TODO: construct DarkFi anonymous transfer with ZK proof
    let _ = (keys, to_address, amount);
    Err(anyhow!("DRK transfers not yet implemented"))
}
