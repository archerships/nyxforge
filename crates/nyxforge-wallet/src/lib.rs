//! NyxForge unified wallet: XMR light wallet + DRK note wallet.
//!
//! Design: the [`MoneroSource`] trait isolates blockchain access so a
//! full-node implementation can be swapped in later without touching callers.

pub mod balance;
pub mod drk;
pub mod keys;
pub mod storage;
pub mod xmr;

pub use balance::Balance;
pub use keys::WalletKeys;
pub use storage::WalletStorage;
pub use xmr::source::MoneroSource;
pub use xmr::remote::RemoteMonerod;
