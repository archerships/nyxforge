//! Shared in-memory + persistent node state.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, RwLock};

use nyxforge_core::bounty::{Bounty, BountyComment, BountyId, JudgeResponse};
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
    /// Known bounty series, keyed by ID.
    bounties: RwLock<HashMap<BountyId, Bounty>>,

    /// One order book per active bounty series.
    order_books: RwLock<HashMap<BountyId, OrderBook>>,

    /// Nullifier set — spent notes cannot be respent.
    spent_nullifiers: RwLock<HashMap<Nullifier, u64>>, // nullifier → block height

    /// Wallet sub-state.
    pub wallet: WalletState,

    /// Miner sub-state.
    pub miner: MinerState,

    /// Comments on proposed bounties, keyed by bounty ID.
    comments: RwLock<HashMap<BountyId, Vec<BountyComment>>>,

    /// Judge accept/reject responses, keyed by bounty ID.
    judge_responses: RwLock<HashMap<BountyId, Vec<JudgeResponse>>>,

    /// Data IDs announced by connected judge nodes.
    known_data_ids: RwLock<HashSet<String>>,

    /// When true, bounties.issue skips the judge data_id check (test/dev mode).
    allow_unverifiable: bool,

    /// Local data directory.
    pub data_dir: std::path::PathBuf,
}

impl NodeState {
    pub async fn new(data_dir: &Path, allow_unverifiable: bool) -> Result<Self> {
        tokio::fs::create_dir_all(data_dir).await?;
        Ok(Self(Arc::new(Inner {
            bounties: RwLock::new(HashMap::new()),
            order_books: RwLock::new(HashMap::new()),
            spent_nullifiers: RwLock::new(HashMap::new()),
            comments: RwLock::new(HashMap::new()),
            judge_responses: RwLock::new(HashMap::new()),
            known_data_ids: RwLock::new(HashSet::new()),
            allow_unverifiable,
            wallet: WalletState::new(),
            miner: MinerState::new(),
            data_dir: data_dir.to_owned(),
        })))
    }

    // -- Bounty helpers -------------------------------------------------------

    pub async fn insert_bounty(&self, bounty: Bounty) {
        let mut bounties = self.0.bounties.write().await;
        let mut books = self.0.order_books.write().await;
        let id = bounty.id;
        bounties.insert(id, bounty);
        books.entry(id).or_insert_with(OrderBook::new);
    }

    pub async fn get_bounty(&self, id: &BountyId) -> Option<Bounty> {
        self.0.bounties.read().await.get(id).cloned()
    }

    pub async fn is_nullifier_spent(&self, n: &Nullifier) -> bool {
        self.0.spent_nullifiers.read().await.contains_key(n)
    }

    pub async fn mark_nullifier_spent(&self, n: Nullifier, block: u64) {
        self.0.spent_nullifiers.write().await.insert(n, block);
    }

    pub async fn bounty_count(&self) -> usize {
        self.0.bounties.read().await.len()
    }

    pub async fn list_bounties(&self) -> Vec<Bounty> {
        self.0.bounties.read().await.values().cloned().collect()
    }

    // -- Proposal comments --------------------------------------------------

    /// Append a comment to a bounty. The bounty must already be stored.
    pub async fn insert_comment(&self, comment: BountyComment) {
        self.0.comments.write().await
            .entry(comment.bounty_id)
            .or_default()
            .push(comment);
    }

    /// Return all comments on a bounty, oldest first.
    pub async fn get_comments(&self, bounty_id: &BountyId) -> Vec<BountyComment> {
        self.0.comments.read().await
            .get(bounty_id)
            .cloned()
            .unwrap_or_default()
    }

    // -- Judge approval responses ------------------------------------------

    /// Record an judge's accept/reject response.
    /// Returns `true` if this response completes the set and the bounty should
    /// advance to `Draft` (all listed judges have now accepted).
    pub async fn record_judge_response(&self, response: JudgeResponse) -> bool {
        let bounty_id = response.bounty_id;
        self.0.judge_responses.write().await
            .entry(bounty_id)
            .or_default()
            .push(response);

        // Check if all judges have accepted.
        self.all_judges_accepted(&bounty_id).await
    }

    /// Returns true if every key in the bounty's JudgeSpec has an accepted
    /// response and none have rejected.
    pub async fn all_judges_accepted(&self, bounty_id: &BountyId) -> bool {
        let bounty = match self.0.bounties.read().await.get(bounty_id).cloned() {
            Some(b) => b,
            None => return false,
        };
        let responses = self.0.judge_responses.read().await;
        let recorded = responses.get(bounty_id).map(Vec::as_slice).unwrap_or(&[]);

        bounty.judge.judge_keys.iter().all(|key| {
            recorded.iter().any(|r| r.judge_key == *key && r.accepted)
        })
    }

    /// Return all judge responses for a bounty.
    pub async fn get_judge_responses(&self, bounty_id: &BountyId) -> Vec<JudgeResponse> {
        self.0.judge_responses.read().await
            .get(bounty_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear judge responses for a bounty (called after the issuer revises
    /// the judge list so judges must re-accept from scratch).
    pub async fn clear_judge_responses(&self, bounty_id: &BountyId) {
        self.0.judge_responses.write().await.remove(bounty_id);
    }

    // -- Judge registry ----------------------------------------------------

    /// Record data IDs announced by an judge node.
    pub async fn register_data_ids(&self, ids: Vec<String>) {
        let mut set = self.0.known_data_ids.write().await;
        for id in ids {
            set.insert(id);
        }
    }

    /// Returns true if at least one judge has announced support for this data_id,
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
