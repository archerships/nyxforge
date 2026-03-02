//! Shared in-memory + persistent node state.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, RwLock};

use nyxforge_core::bond::{Bond, BondId};
use nyxforge_core::market::OrderBook;
use nyxforge_core::types::Nullifier;
use nyxforge_miner::{MinerCmd, MinerStats};
use nyxforge_wallet::{Balance, WalletKeys};

// ---------------------------------------------------------------------------
// Wallet state
// ---------------------------------------------------------------------------

/// Wallet state for XMR + DRK.
#[derive(Debug)]
pub struct WalletState {
    /// Keys — None until `wallet.create` is called.
    pub keys: RwLock<Option<WalletKeys>>,
    /// Cached balance updated by the background scanner.
    pub balance: RwLock<Balance>,
    /// Last XMR scan height.
    pub last_scan_height: AtomicU64,
}

impl WalletState {
    fn new() -> Self {
        Self {
            keys: RwLock::new(None),
            balance: RwLock::new(Balance::zero()),
            last_scan_height: AtomicU64::new(0),
        }
    }

    pub fn scan_height(&self) -> u64 {
        self.last_scan_height.load(Ordering::Relaxed)
    }

    pub fn set_scan_height(&self, h: u64) {
        self.last_scan_height.store(h, Ordering::Relaxed);
    }
}

// ---------------------------------------------------------------------------
// Miner state
// ---------------------------------------------------------------------------

/// Miner state: config, live stats, and control channel.
#[derive(Debug)]
pub struct MinerState {
    /// Miner configuration (updated via RPC).
    pub config: RwLock<Option<nyxforge_miner::MinerConfig>>,
    /// Latest stats snapshot from the mining threads.
    pub stats: RwLock<MinerStats>,
    /// Channel to send commands to the miner control task.
    pub ctl_tx: RwLock<Option<mpsc::Sender<MinerCmd>>>,
}

impl MinerState {
    fn new() -> Self {
        Self {
            config: RwLock::new(None),
            stats: RwLock::new(MinerStats::default()),
            ctl_tx: RwLock::new(None),
        }
    }

    /// Send a command to the miner. Returns `false` if miner is not
    /// initialised yet.
    pub async fn send_cmd(&self, cmd: MinerCmd) -> bool {
        let guard = self.ctl_tx.read().await;
        if let Some(tx) = guard.as_ref() {
            tx.send(cmd).await.is_ok()
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// NodeState
// ---------------------------------------------------------------------------

/// Shared node state, cheap to clone (Arc-wrapped).
#[derive(Clone, Debug)]
pub struct NodeState(Arc<Inner>);

#[derive(Debug)]
struct Inner {
    /// Known bond series, keyed by ID.
    bonds: RwLock<HashMap<BondId, Bond>>,

    /// One order book per active bond series.
    order_books: RwLock<HashMap<BondId, OrderBook>>,

    /// Nullifier set — spent notes cannot be respent.
    spent_nullifiers: RwLock<HashMap<Nullifier, u64>>, // nullifier → block height

    /// Wallet sub-state.
    pub wallet: WalletState,

    /// Miner sub-state.
    pub miner: MinerState,

    /// Local data directory.
    pub data_dir: std::path::PathBuf,
}

impl NodeState {
    pub async fn new(data_dir: &Path) -> Result<Self> {
        tokio::fs::create_dir_all(data_dir).await?;
        Ok(Self(Arc::new(Inner {
            bonds: RwLock::new(HashMap::new()),
            order_books: RwLock::new(HashMap::new()),
            spent_nullifiers: RwLock::new(HashMap::new()),
            wallet: WalletState::new(),
            miner: MinerState::new(),
            data_dir: data_dir.to_owned(),
        })))
    }

    // -- Bond helpers -------------------------------------------------------

    pub async fn insert_bond(&self, bond: Bond) {
        let mut bonds = self.0.bonds.write().await;
        let mut books = self.0.order_books.write().await;
        let id = bond.id;
        bonds.insert(id, bond);
        books.entry(id).or_insert_with(OrderBook::new);
    }

    pub async fn get_bond(&self, id: &BondId) -> Option<Bond> {
        self.0.bonds.read().await.get(id).cloned()
    }

    pub async fn is_nullifier_spent(&self, n: &Nullifier) -> bool {
        self.0.spent_nullifiers.read().await.contains_key(n)
    }

    pub async fn mark_nullifier_spent(&self, n: Nullifier, block: u64) {
        self.0.spent_nullifiers.write().await.insert(n, block);
    }

    pub async fn bond_count(&self) -> usize {
        self.0.bonds.read().await.len()
    }

    pub async fn list_bonds(&self) -> Vec<Bond> {
        self.0.bonds.read().await.values().cloned().collect()
    }

    // -- Wallet / miner accessors -------------------------------------------

    pub fn wallet(&self) -> &WalletState {
        &self.0.wallet
    }

    pub fn miner(&self) -> &MinerState {
        &self.0.miner
    }

    pub fn data_dir(&self) -> &std::path::Path {
        &self.0.data_dir
    }
}
