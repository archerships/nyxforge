//! NyxForge DarkFi WASM contracts.
//!
//! Each contract exposes a single `process_instruction` entry-point following
//! DarkFi's contract ABI.  The runtime verifies ZK proofs and applies state
//! transitions atomically.
//!
//! Contracts:
//!   - `bond_market`  — issue and list bond series
//!   - `order_book`   — anonymous DEX for bond trading
//!   - `settlement`   — oracle-triggered redemption and payout

pub mod bond_market;
pub mod order_book;
pub mod settlement;

use nyxforge_core::error::NyxError;

pub type ContractResult<T> = Result<T, NyxError>;
