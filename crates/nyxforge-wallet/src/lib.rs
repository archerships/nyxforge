//! NyxForge crypto wallet: XMR light wallet (primary collateral currency).
//!
//! Design: the [`MoneroSource`] trait isolates blockchain access so a
//! full-node implementation can be swapped in later without touching callers.
//! DRK note wallet removed 2026-04-25; archived at src/archive/darkfi-era/.

pub mod balance;
pub mod keys;
pub mod storage;
pub mod xmr;

pub use balance::Balance;
pub use keys::WalletKeys;
pub use storage::WalletStorage;
pub use xmr::source::MoneroSource;
pub use xmr::remote::RemoteMonerod;
