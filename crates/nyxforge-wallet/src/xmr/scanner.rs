//! Output scanning: detect which blockchain outputs belong to this wallet.
//!
//! Monero output detection uses the Diffie-Hellman key exchange:
//!   P = Hs(r·V || i)·G + S
//! where r is tx private key, V is recipient view key, S is recipient spend
//! public key, i is output index. The wallet checks each output by computing
//! P' and comparing to the on-chain key.

use anyhow::Result;
use tracing::info;

use super::source::MoneroSource;
use crate::keys::WalletKeys;

/// An output owned by this wallet.
#[derive(Debug, Clone)]
pub struct OwnedOutput {
    pub global_index: u64,
    pub amount: u64,
    pub key_image: [u8; 32],
    pub subaddress_index: (u32, u32),
}

/// Scans a range of blocks and returns outputs owned by the wallet.
pub async fn scan_range<S: MoneroSource>(
    source: &S,
    keys: &WalletKeys,
    from_height: u64,
    to_height: u64,
) -> Result<(Vec<OwnedOutput>, u64)> {
    info!(from = from_height, to = to_height, address = %keys.xmr_address_string(), "scanning outputs");

    // TODO: full implementation:
    //   1. Fetch block headers + transactions for each height via source
    //   2. For each tx output, compute the one-time address check
    //   3. If owned, derive the key image and record the output
    let _ = source;

    let owned: Vec<OwnedOutput> = Vec::new();
    Ok((owned, to_height))
}
