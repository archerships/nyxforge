//! Oracle node: fetches public data, evaluates goal metrics, and posts attestations.
//!
//! An oracle node:
//!   1. Monitors registered bond series for unverified goals near their deadline.
//!   2. Fetches data from registered data adapters (HTTP APIs, IPFS, etc.).
//!   3. Evaluates the GoalMetric predicate.
//!   4. Signs an OracleAttestation with its Ed25519 key.
//!   5. Posts the attestation to the P2P network.
//!   6. Stakes collateral to back its claim; gets slashed if fraudulent.
//!
//! ## Push model
//! `OracleNode::monitor_bonds` spawns a monitoring task per bond and sends
//! a signed attestation on the provided channel the moment ALL goals are met.
//! This is a push model: the oracle observes goal conditions and pushes to AO,
//! rather than AO cron-pulling the oracle on a fixed schedule.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

use nyxforge_core::bond::{Bond, BondId, BondState};
use nyxforge_core::oracle_spec::OracleAttestation;
use nyxforge_core::types::{Digest, PublicKey};

use crate::verifier::DataSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    /// This oracle's public key (Ristretto255 / Ed25519 compressed).
    pub public_key: PublicKey,

    /// Bonds this oracle is registered to verify (empty = all).
    pub bond_filter: Vec<BondId>,

    /// How far before the deadline to begin polling (seconds).
    pub poll_lead_secs: u64,

    /// Polling interval (seconds).
    pub poll_interval_secs: u64,
}

pub struct OracleNode {
    pub config: OracleConfig,
    secret_key: [u8; 32],
    sources:    Vec<Box<dyn DataSource>>,
}

impl OracleNode {
    pub fn new(
        config: OracleConfig,
        secret_key: [u8; 32],
        sources: Vec<Box<dyn DataSource>>,
    ) -> Self {
        Self { config, secret_key, sources }
    }

    /// Returns the list of data IDs this oracle can evaluate.
    pub fn supported_data_ids(&self) -> Vec<String> {
        self.sources.iter().map(|s: &Box<dyn DataSource>| s.data_id().to_owned()).collect()
    }

    /// Evaluate a bond's goal and produce a signed attestation.
    pub async fn evaluate(&self, bond: &Bond) -> Result<OracleAttestation> {
        if bond.state != BondState::Active {
            anyhow::bail!("bond is not ACTIVE");
        }

        // Evaluate each goal criterion; ALL must be met (AND semantics).
        let mut all_met = true;
        let mut combined_hash = blake3::Hasher::new();
        for goal in &bond.goals {
            let mut last_err = None;
            let mut evaluated = false;
            for source in &self.sources {
                if !source.supports(&goal.metric.data_id) {
                    continue;
                }
                match source.fetch(&goal.metric.data_id).await {
                    Ok(value) => {
                        let met = goal.metric.operator.evaluate(value, goal.metric.threshold);
                        if !met { all_met = false; }
                        let evidence_hash = self.hash_evidence(&goal.metric.data_id, value);
                        combined_hash.update(evidence_hash.as_bytes());
                        info!(
                            bond_id = ?bond.id,
                            data_id = %goal.metric.data_id,
                            goal_met = met,
                            value = %value,
                            "criterion evaluated",
                        );
                        evaluated = true;
                        break;
                    }
                    Err(e) => {
                        warn!("data source error: {e}");
                        last_err = Some(e);
                    }
                }
            }
            if !evaluated {
                return Err(last_err.unwrap_or_else(|| {
                    anyhow::anyhow!("no data source for {}", goal.metric.data_id)
                }));
            }
        }

        let evidence_hash = Digest::from(combined_hash.finalize());
        let attestation = self.sign_attestation(bond.id, all_met, evidence_hash);
        info!(bond_id = ?bond.id, all_goals_met = all_met, "attestation produced");
        Ok(attestation)
    }

    fn sign_attestation(
        &self,
        bond_id: BondId,
        goal_met: bool,
        evidence_hash: Digest,
    ) -> OracleAttestation {
        let now = Utc::now();

        // TODO: replace with real Ed25519 signing.
        let mut sig_input = blake3::Hasher::new();
        sig_input.update(bond_id.as_bytes());
        sig_input.update(&[goal_met as u8]);
        sig_input.update(evidence_hash.as_bytes());
        sig_input.update(&now.timestamp().to_le_bytes());
        sig_input.update(&self.secret_key);
        let sig_hash = sig_input.finalize();

        let mut signature = [0u8; 64];
        signature[..32].copy_from_slice(sig_hash.as_bytes());
        signature[32..].copy_from_slice(&self.secret_key[..32]); // stub

        OracleAttestation {
            bond_id,
            goal_met,
            evidence_hash,
            evidence_uri: None,
            oracle_key: self.config.public_key.clone(),
            signature: signature.to_vec(),
            attested_at: now,
        }
    }

    fn hash_evidence(&self, data_id: &str, value: rust_decimal::Decimal) -> Digest {
        let mut h = blake3::Hasher::new();
        h.update(b"nyxforge::evidence");
        h.update(data_id.as_bytes());
        h.update(value.to_string().as_bytes());
        Digest::from(h.finalize())
    }

    /// Push-model monitoring loop.
    ///
    /// Spawns one tokio task per bond in `bonds`.  Each task polls all goals at
    /// `config.poll_interval_secs` and sends exactly one `OracleAttestation` on
    /// `attestation_tx` the first time ALL goals are simultaneously met.
    ///
    /// Returns a `Vec` of `tokio::task::JoinHandle`s so callers can cancel or
    /// await them.
    ///
    /// Monitoring begins immediately.  Each task exits after sending its
    /// attestation (a single bond fires at most once per call to this method).
    pub fn monitor_bonds(
        self: Arc<Self>,
        bonds: Vec<Bond>,
        attestation_tx: mpsc::Sender<OracleAttestation>,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        bonds
            .into_iter()
            .filter(|b| b.state == BondState::Active)
            .map(|bond| {
                let node = Arc::clone(&self);
                let tx = attestation_tx.clone();
                let poll_secs = node.config.poll_interval_secs.max(1);
                tokio::spawn(async move {
                    let mut ticker = interval(Duration::from_secs(poll_secs));
                    loop {
                        ticker.tick().await;
                        match node.evaluate(&bond).await {
                            Ok(attestation) if attestation.goal_met => {
                                info!(
                                    bond_id = ?bond.id,
                                    "all goals met — pushing attestation",
                                );
                                // Channel send failure means caller dropped the receiver;
                                // stop monitoring this bond.
                                let _ = tx.send(attestation).await;
                                return;
                            }
                            Ok(_) => {
                                info!(bond_id = ?bond.id, "goals not yet met — continuing poll");
                            }
                            Err(e) => {
                                warn!(bond_id = ?bond.id, "evaluate error: {e}");
                            }
                        }
                    }
                })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_test_fixtures::bonds::{active_bond, draft_bond};
    use crate::verifier::MockDataSource;
    #[allow(unused_imports)]
    use rust_decimal::Decimal;

    fn mock_source(data_id: &str, value: f64) -> Box<dyn crate::verifier::DataSource> {
        Box::new(MockDataSource {
            data_id: data_id.into(),
            value:   Decimal::try_from(value).unwrap(),
        })
    }

    fn test_node(sources: Vec<Box<dyn crate::verifier::DataSource>>) -> Arc<OracleNode> {
        Arc::new(OracleNode::new(
            OracleConfig {
                public_key:         PublicKey([0x22; 32]),
                bond_filter:        vec![],
                poll_lead_secs:     0,
                poll_interval_secs: 1,
            },
            [0xAB; 32],
            sources,
        ))
    }

    // --- evaluate() ---

    #[tokio::test]
    async fn evaluate_active_bond_all_goals_met() {
        // minimal_goal: test.metric < 100; supply value 50 → met
        let node = test_node(vec![mock_source("test.metric", 50.0)]);
        let bond = active_bond();
        let att = node.evaluate(&bond).await.unwrap();
        assert!(att.goal_met, "all goals should be met");
        assert_eq!(att.bond_id, bond.id);
    }

    #[tokio::test]
    async fn evaluate_active_bond_goal_not_met() {
        // minimal_goal: test.metric < 100; supply value 150 → not met
        let node = test_node(vec![mock_source("test.metric", 150.0)]);
        let bond = active_bond();
        let att = node.evaluate(&bond).await.unwrap();
        assert!(!att.goal_met, "goal should not be met when value exceeds threshold");
    }

    #[tokio::test]
    async fn evaluate_rejects_non_active_bond() {
        let node = test_node(vec![mock_source("test.metric", 50.0)]);
        let bond = draft_bond();
        let err = node.evaluate(&bond).await.unwrap_err();
        assert!(err.to_string().contains("ACTIVE"), "expected ACTIVE error, got: {err}");
    }

    #[tokio::test]
    async fn evaluate_error_when_no_source_for_data_id() {
        let node = test_node(vec![]);
        let bond = active_bond();
        let err = node.evaluate(&bond).await.unwrap_err();
        assert!(err.to_string().contains("no data source"), "unexpected error: {err}");
    }

    // --- monitor_bonds() push model ---

    #[tokio::test]
    async fn monitor_bonds_sends_attestation_when_all_goals_met() {
        let node = test_node(vec![mock_source("test.metric", 50.0)]);
        let (tx, mut rx) = mpsc::channel(4);
        let bond = active_bond();
        let bond_id = bond.id;
        let _handles = node.monitor_bonds(vec![bond], tx);
        let att = tokio::time::timeout(
            Duration::from_secs(5),
            rx.recv(),
        )
        .await
        .expect("timeout waiting for attestation")
        .expect("channel closed before attestation");
        assert!(att.goal_met);
        assert_eq!(att.bond_id, bond_id);
    }

    #[tokio::test]
    async fn monitor_bonds_skips_non_active_bonds() {
        let node = test_node(vec![mock_source("test.metric", 50.0)]);
        let (tx, mut rx) = mpsc::channel(4);
        // Keep a sender clone alive so the channel doesn't close when monitor_bonds
        // drops the moved sender (no tasks are spawned for non-active bonds).
        let _tx_guard = tx.clone();
        let bond = draft_bond();
        let _handles = node.monitor_bonds(vec![bond], tx);
        // Channel stays open but empty; timeout means no message arrived.
        let result = tokio::time::timeout(
            Duration::from_millis(200),
            rx.recv(),
        )
        .await;
        assert!(result.is_err(), "expected timeout — no attestation for non-active bond");
    }

    #[tokio::test]
    async fn monitor_bonds_does_not_send_when_goal_not_met() {
        // value 150 fails test.metric < 100
        let node = test_node(vec![mock_source("test.metric", 150.0)]);
        let (tx, mut rx) = mpsc::channel(4);
        let bond = active_bond();
        let handles = node.monitor_bonds(vec![bond], tx);
        // Let it poll twice (poll_interval_secs = 1), then abort.
        tokio::time::sleep(Duration::from_millis(2_500)).await;
        for h in handles { h.abort(); }
        // After abort the task's sender clone is dropped; channel closes.
        // try_recv returns Err regardless of whether channel is empty or disconnected —
        // either way confirms no attestation was buffered.
        assert!(rx.try_recv().is_err(), "no attestation should be sent while goal is not met");
    }
}
