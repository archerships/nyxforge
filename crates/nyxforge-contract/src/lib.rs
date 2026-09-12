//! NyxForge DarkFi WASM contracts.
//!
//! Each contract exposes a single `process_instruction` entry-point following
//! DarkFi's contract ABI.  The runtime verifies ZK proofs and applies state
//! transitions atomically.
//!
//! Contracts:
//!   - `bounty_market`  — issue and list bounty series
//!   - `order_book`   — anonymous DEX for bounty trading
//!   - `settlement`   — judge-triggered redemption and payout

pub mod bounty_market;
pub mod order_book;
pub mod settlement;
pub mod actor;

use nyxforge_core::error::NyxError;

pub type ContractResult<T> = Result<T, NyxError>;
