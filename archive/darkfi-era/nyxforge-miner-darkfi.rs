//! DarkFi merge-mine interface — stub until DarkFi exposes a public mining API.
//!
//! When DarkFi's merge-mine protocol is released, implement this module to:
//!   1. Connect to a local darkfi-node via its RPC
//!   2. Include DarkFi block headers in the auxiliary data of Monero blocks
//!   3. Submit found solutions to darkfi-node
//!
//! For now this module is intentionally empty.

/// Placeholder — returns `None` indicating no DarkFi merge-mine data.
pub fn aux_block_data() -> Option<Vec<u8>> {
    None
}
