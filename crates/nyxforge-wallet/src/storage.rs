//! Wallet file persistence.
//!
//! Current implementation: plaintext JSON (TODO: add AES-256-GCM encryption
//! keyed from a user passphrase via Argon2id before mainnet).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::balance::Balance;
use crate::keys::{WalletKeys, WalletKeysSerde};

/// On-disk wallet data.
#[derive(Serialize, Deserialize)]
pub struct WalletFile {
    pub version: u32,
    pub keys: WalletKeysSerde,
    pub cached_balance: Balance,
    pub last_scan_height: u64,
}

pub struct WalletStorage {
    path: PathBuf,
}

impl WalletStorage {
    pub fn new(data_dir: &Path) -> Self {
        Self { path: data_dir.join("wallet.json") }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub async fn save(&self, keys: &WalletKeys, balance: Balance, height: u64) -> Result<()> {
        let file = WalletFile {
            version: 1,
            keys: keys.to_serde(),
            cached_balance: balance,
            last_scan_height: height,
        };
        let json = serde_json::to_string_pretty(&file)?;
        fs::write(&self.path, json)
            .await
            .with_context(|| format!("writing wallet to {}", self.path.display()))?;
        Ok(())
    }

    pub async fn load(&self) -> Result<(WalletKeys, Balance, u64)> {
        let json = fs::read_to_string(&self.path)
            .await
            .with_context(|| format!("reading wallet from {}", self.path.display()))?;
        let file: WalletFile = serde_json::from_str(&json)?;
        let keys = WalletKeys::from_serde(file.keys)?;
        Ok((keys, file.cached_balance, file.last_scan_height))
    }
}
