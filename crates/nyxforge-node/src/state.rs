//! Shared in-memory + persistent node state.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, RwLock};

use nyxforge_core::bond::{Bond, BondComment, BondId, OracleResponse};
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

    /// Comments on proposed bonds, keyed by bond ID.
    comments: RwLock<HashMap<BondId, Vec<BondComment>>>,

    /// Oracle accept/reject responses, keyed by bond ID.
    oracle_responses: RwLock<HashMap<BondId, Vec<OracleResponse>>>,

    /// Data IDs announced by connected oracle nodes.
    known_data_ids: RwLock<HashSet<String>>,

    /// When true, bonds.issue skips the oracle data_id check (test/dev mode).
    allow_unverifiable: bool,

    /// Local data directory.
    pub data_dir: std::path::PathBuf,
}

impl NodeState {
    pub async fn new(data_dir: &Path, allow_unverifiable: bool) -> Result<Self> {
        tokio::fs::create_dir_all(data_dir).await?;
        Ok(Self(Arc::new(Inner {
            bonds: RwLock::new(HashMap::new()),
            order_books: RwLock::new(HashMap::new()),
            spent_nullifiers: RwLock::new(HashMap::new()),
            comments: RwLock::new(HashMap::new()),
            oracle_responses: RwLock::new(HashMap::new()),
            known_data_ids: RwLock::new(HashSet::new()),
            allow_unverifiable,
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

    // -- Proposal comments --------------------------------------------------

    /// Append a comment to a bond. The bond must already be stored.
    pub async fn insert_comment(&self, comment: BondComment) {
        self.0.comments.write().await
            .entry(comment.bond_id)
            .or_default()
            .push(comment);
    }

    /// Return all comments on a bond, oldest first.
    pub async fn get_comments(&self, bond_id: &BondId) -> Vec<BondComment> {
        self.0.comments.read().await
            .get(bond_id)
            .cloned()
            .unwrap_or_default()
    }

    // -- Oracle approval responses ------------------------------------------

    /// Record an oracle's accept/reject response.
    /// Returns `true` if this response completes the set and the bond should
    /// advance to `Draft` (all listed oracles have now accepted).
    pub async fn record_oracle_response(&self, response: OracleResponse) -> bool {
        let bond_id = response.bond_id;
        self.0.oracle_responses.write().await
            .entry(bond_id)
            .or_default()
            .push(response);

        // Check if all oracles have accepted.
        self.all_oracles_accepted(&bond_id).await
    }

    /// Returns true if every key in the bond's OracleSpec has an accepted
    /// response and none have rejected.
    pub async fn all_oracles_accepted(&self, bond_id: &BondId) -> bool {
        let bond = match self.0.bonds.read().await.get(bond_id).cloned() {
            Some(b) => b,
            None => return false,
        };
        let responses = self.0.oracle_responses.read().await;
        let recorded = responses.get(bond_id).map(Vec::as_slice).unwrap_or(&[]);

        bond.oracle.oracle_keys.iter().all(|key| {
            recorded.iter().any(|r| r.oracle_key == *key && r.accepted)
        })
    }

    /// Return all oracle responses for a bond.
    pub async fn get_oracle_responses(&self, bond_id: &BondId) -> Vec<OracleResponse> {
        self.0.oracle_responses.read().await
            .get(bond_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear oracle responses for a bond (called after the issuer revises
    /// the oracle list so oracles must re-accept from scratch).
    pub async fn clear_oracle_responses(&self, bond_id: &BondId) {
        self.0.oracle_responses.write().await.remove(bond_id);
    }

    // -- Oracle registry ----------------------------------------------------

    /// Record data IDs announced by an oracle node.
    pub async fn register_data_ids(&self, ids: Vec<String>) {
        let mut set = self.0.known_data_ids.write().await;
        for id in ids {
            set.insert(id);
        }
    }

    /// Returns true if at least one oracle has announced support for this data_id,
    /// or if the node is running in allow-unverifiable mode.
    pub async fn is_data_id_supported(&self, data_id: &str) -> bool {
        self.0.allow_unverifiable
            || self.0.known_data_ids.read().await.contains(data_id)
    }

    /// Returns true when the node was started with --allow-unverifiable.
    pub fn is_unverifiable_allowed(&self) -> bool {
        self.0.allow_unverifiable
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
