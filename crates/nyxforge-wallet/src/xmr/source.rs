//! `MoneroSource` trait — swap RemoteMonerod for LocalMonerod with zero other changes.

use anyhow::Result;
use async_trait::async_trait;

/// A raw output from the Monero blockchain.
#[derive(Debug, Clone)]
pub struct Output {
    pub global_index: u64,
    pub amount: u64,        // picomonero
    pub key: [u8; 32],      // one-time public key (P)
    pub commitment: [u8; 32],
}

/// A submitted transaction hash.
pub type TxHash = String;

/// Abstraction over a Monero blockchain data source.
/// Implement this trait to add full-node support without touching callers.
#[async_trait]
pub trait MoneroSource: Send + Sync {
    /// Current blockchain height.
    async fn get_height(&self) -> Result<u64>;

    /// Fetch global outputs by their indices.
    async fn get_outputs(&self, indices: &[u64]) -> Result<Vec<Output>>;

    /// Broadcast a serialised transaction and return its hash.
    async fn submit_tx(&self, tx_hex: &str) -> Result<TxHash>;
}
