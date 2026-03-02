//! Transaction construction — builds a signed XMR transfer.
//!
//! Full RingCT transaction building requires:
//!   - Selecting real inputs + decoy outputs (ring size 16 for mainnet)
//!   - Computing MLSAG/CLSAG signature
//!   - RangeProof (Bulletproofs+) for output amounts
//!
//! This scaffold exposes the right interface; the body is stubbed with TODO.

use anyhow::{anyhow, Result};
use monero::Address;
use tracing::info;

use super::source::{MoneroSource, TxHash};
use crate::keys::WalletKeys;

/// Build and submit a XMR transfer.
///
/// `amount_pico` is the send amount in picomonero (1 XMR = 1e12 pico).
pub async fn send_xmr<S: MoneroSource>(
    source: &S,
    keys: &WalletKeys,
    to: Address,
    amount_pico: u64,
) -> Result<TxHash> {
    let xmr = amount_pico as f64 / 1_000_000_000_000.0;
    info!(to = %to, amount_xmr = xmr, "building XMR transaction");

    // TODO: full RingCT implementation:
    //   1. Select UTXOs from owned outputs covering amount + fee
    //   2. Fetch decoy outputs from `source.get_outputs()`
    //   3. Construct RingCT tx with Bulletproofs+ range proofs
    //   4. Sign with MLSAG/CLSAG using spend key
    //   5. Serialise and submit via source.submit_tx()

    let _ = (source, keys, to, amount_pico);
    Err(anyhow!("XMR transaction construction not yet implemented"))
}
