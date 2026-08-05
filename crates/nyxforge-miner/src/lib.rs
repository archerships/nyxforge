//! NyxForge miner: RandomX CPU miner with P2Pool mini stratum client.
//!
//! This crate is native-only — it links against the RandomX C library and
//! cannot target wasm32.

// darkfi merge-mining module removed 2026-04-25; archived at src/archive/darkfi-era/
pub mod hasher;
pub mod p2pool;
pub mod stats;
pub mod worker;

pub use p2pool::MinerConfig;
pub use stats::MinerStats;
pub use worker::{MinerCmd, MinerHandle};
