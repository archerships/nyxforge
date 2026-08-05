//! Judge node: fetches public data, evaluates goal metrics, and posts attestations.
//!
//! An judge node:
//!   1. Monitors registered bounty series for unverified goals near their deadline.
//!   2. Fetches data from registered data adapters (HTTP APIs, IPFS, etc.).
//!   3. Evaluates the GoalMetric predicate.
//!   4. Signs an JudgeAttestation with its Ed25519 key.
//!   5. Posts the attestation to the P2P network.
//!   6. Stakes collateral to back its claim; gets slashed if fraudulent.
//!
//! ## Push model
//! `JudgeNode::monitor_bonds` spawns a monitoring task per bounty and sends
//! a signed attestation on the provided channel the moment ALL goals are met.
//! This is a push model: the judge observes goal conditions and pushes to AO,
//! rather than AO cron-pulling the judge on a fixed schedule.

use anyhow::Result;
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

use nyxforge_core::bounty::{Bounty, BountyId, BountyState};
use nyxforge_core::judge_spec::{JudgeAttestKey, JudgeAttestation};
use nyxforge_core::types::{Digest, PublicKey};
use nyxforge_zk::primitives::judge_attest_pk_from_key;

use crate::verifier::DataSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeConfig {
    /// This judge's public key (Ristretto255 / Ed25519 compressed).
    pub public_key: PublicKey,

    /// Bounties this judge is registered to verify (empty = all).
    pub bond_filter: Vec<BountyId>,

    /// How far before the deadline to begin polling (seconds).
    pub poll_lead_secs: u64,

    /// Polling interval (seconds).
    pub poll_interval_secs: u64,
}

pub struct JudgeNode {
    pub config: JudgeConfig,
    secret_key: [u8; 32],
    sources:    Vec<Box<dyn DataSource>>,
}

impl JudgeNode {
    pub fn new(
        config: JudgeConfig,
        secret_key: [u8; 32],
        sources: Vec<Box<dyn DataSource>>,
    ) -> Self {
        Self { config, secret_key, sources }
    }

    /// Returns the list of data IDs this judge can evaluate.
    pub fn supported_data_ids(&self) -> Vec<String> {
        self.sources.iter().map(|s: &Box<dyn DataSource>| s.data_id().to_owned()).collect()
    }

    /// Evaluate a bounty's goal and produce a signed attestation.
    pub async fn evaluate(&self, bounty: &Bounty) -> Result<JudgeAttestation> {
        if bounty.state != BountyState::Active {
            anyhow::bail!("bounty is not ACTIVE");
        }

        // Evaluate each goal criterion; ALL must be met (AND semantics).
        let mut all_met = true;
        let mut combined_hash = blake3::Hasher::new();
        for goal in &bounty.goals {
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
                            bounty_id = ?bounty.id,
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
        let attestation = self.sign_attestation(bounty.id, all_met, evidence_hash);
        info!(bounty_id = ?bounty.id, all_goals_met = all_met, "attestation produced");
        Ok(attestation)
    }

    fn sign_attestation(
        &self,
        bounty_id: BountyId,
        goal_met: bool,
        evidence_hash: Digest,
    ) -> JudgeAttestation {
        let now = Utc::now();

        // Message: bounty_id || goal_met || evidence_hash || timestamp_le
        let mut msg = Vec::with_capacity(32 + 1 + 32 + 8);
        msg.extend_from_slice(bounty_id.as_bytes());
        msg.push(goal_met as u8);
        msg.extend_from_slice(evidence_hash.as_bytes());
        msg.extend_from_slice(&now.timestamp().to_le_bytes());

        let signing_key = SigningKey::from_bytes(&self.secret_key);
        let signature   = signing_key.sign(&msg);

        JudgeAttestation {
            bounty_id,
            goal_met,
            evidence_hash,
            evidence_uri: None,
            judge_key: self.config.public_key.clone(),
            signature: signature.to_bytes().to_vec(),
            attested_at: now,
        }
    }

    /// Derive the per-bounty judge attestation key token.
    ///
    /// ```text
    /// bond_attest_key = blake3(master_sk ‖ bounty_id ‖ "nyxforge::oracle::attest::v1")
    /// judge_attest_pk = Poseidon2(fp(bond_attest_key), judge_attest_domain())
    /// ```
    ///
    /// The judge sends the returned [`JudgeAttestKey`] to the bounty holder
    /// over an encrypted channel.  The bounty holder uses `bond_attest_key` as
    /// the private witness in the BURN circuit; `judge_attest_pk` is
    /// registered on the bounty and becomes circuit instance[2].
    pub fn generate_attest_key(&self, bounty_id: BountyId) -> JudgeAttestKey {
        let mut h = blake3::Hasher::new();
        h.update(&self.secret_key);
        h.update(bounty_id.as_bytes());
        h.update(b"nyxforge::oracle::attest::v1");
        let bond_attest_key: [u8; 32] = *h.finalize().as_bytes();

        let judge_attest_pk = judge_attest_pk_from_key(&bond_attest_key);
        JudgeAttestKey { bond_attest_key, judge_attest_pk }
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
    /// Spawns one tokio task per bounty in `bounties`.  Each task polls all goals at
    /// `config.poll_interval_secs` and sends exactly one `JudgeAttestation` on
    /// `attestation_tx` the first time ALL goals are simultaneously met.
    ///
    /// Returns a `Vec` of `tokio::task::JoinHandle`s so callers can cancel or
    /// await them.
    ///
    /// Monitoring begins immediately.  Each task exits after sending its
    /// attestation (a single bounty fires at most once per call to this method).
    pub fn monitor_bonds(
        self: Arc<Self>,
        bounties: Vec<Bounty>,
        attestation_tx: mpsc::Sender<JudgeAttestation>,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        bounties
            .into_iter()
            .filter(|b| b.state == BountyState::Active)
            .map(|bounty| {
                let node = Arc::clone(&self);
                let tx = attestation_tx.clone();
                let poll_secs = node.config.poll_interval_secs.max(1);
                tokio::spawn(async move {
                    let mut ticker = interval(Duration::from_secs(poll_secs));
                    loop {
                        ticker.tick().await;
                        match node.evaluate(&bounty).await {
                            Ok(attestation) if attestation.goal_met => {
                                info!(
                                    bounty_id = ?bounty.id,
                                    "all goals met — pushing attestation",
                                );
                                // Channel send failure means caller dropped the receiver;
                                // stop monitoring this bounty.
                                let _ = tx.send(attestation).await;
                                return;
                            }
                            Ok(_) => {
                                info!(bounty_id = ?bounty.id, "goals not yet met — continuing poll");
                            }
                            Err(e) => {
                                warn!(bounty_id = ?bounty.id, "evaluate error: {e}");
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
    use nyxforge_test_fixtures::bounties::{active_bounty, draft_bounty};
    use crate::verifier::MockDataSource;
    #[allow(unused_imports)]
    use rust_decimal::Decimal;

    fn mock_source(data_id: &str, value: f64) -> Box<dyn crate::verifier::DataSource> {
        Box::new(MockDataSource {
            data_id: data_id.into(),
            value:   Decimal::try_from(value).unwrap(),
        })
    }

    fn test_node(sources: Vec<Box<dyn crate::verifier::DataSource>>) -> Arc<JudgeNode> {
        Arc::new(JudgeNode::new(
            JudgeConfig {
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
        let bounty = active_bounty();
        let att = node.evaluate(&bounty).await.unwrap();
        assert!(att.goal_met, "all goals should be met");
        assert_eq!(att.bounty_id, bounty.id);
    }

    #[tokio::test]
    async fn evaluate_active_bond_goal_not_met() {
        // minimal_goal: test.metric < 100; supply value 150 → not met
        let node = test_node(vec![mock_source("test.metric", 150.0)]);
        let bounty = active_bounty();
        let att = node.evaluate(&bounty).await.unwrap();
        assert!(!att.goal_met, "goal should not be met when value exceeds threshold");
    }

    #[tokio::test]
    async fn evaluate_rejects_non_active_bond() {
        let node = test_node(vec![mock_source("test.metric", 50.0)]);
        let bounty = draft_bounty();
        let err = node.evaluate(&bounty).await.unwrap_err();
        assert!(err.to_string().contains("ACTIVE"), "expected ACTIVE error, got: {err}");
    }

    #[tokio::test]
    async fn evaluate_error_when_no_source_for_data_id() {
        let node = test_node(vec![]);
        let bounty = active_bounty();
        let err = node.evaluate(&bounty).await.unwrap_err();
        assert!(err.to_string().contains("no data source"), "unexpected error: {err}");
    }

    // --- monitor_bonds() push model ---

    #[tokio::test]
    async fn monitor_bonds_sends_attestation_when_all_goals_met() {
        let node = test_node(vec![mock_source("test.metric", 50.0)]);
        let (tx, mut rx) = mpsc::channel(4);
        let bounty = active_bounty();
        let bounty_id = bounty.id;
        let _handles = node.monitor_bonds(vec![bounty], tx);
        let att = tokio::time::timeout(
            Duration::from_secs(5),
            rx.recv(),
        )
        .await
        .expect("timeout waiting for attestation")
        .expect("channel closed before attestation");
        assert!(att.goal_met);
        assert_eq!(att.bounty_id, bounty_id);
    }

    #[tokio::test]
    async fn monitor_bonds_skips_non_active_bonds() {
        let node = test_node(vec![mock_source("test.metric", 50.0)]);
        let (tx, mut rx) = mpsc::channel(4);
        // Keep a sender clone alive so the channel doesn't close when monitor_bonds
        // drops the moved sender (no tasks are spawned for non-active bounties).
        let _tx_guard = tx.clone();
        let bounty = draft_bounty();
        let _handles = node.monitor_bonds(vec![bounty], tx);
        // Channel stays open but empty; timeout means no message arrived.
        let result = tokio::time::timeout(
            Duration::from_millis(200),
            rx.recv(),
        )
        .await;
        assert!(result.is_err(), "expected timeout — no attestation for non-active bounty");
    }

    #[tokio::test]
    async fn monitor_bonds_does_not_send_when_goal_not_met() {
        // value 150 fails test.metric < 100
        let node = test_node(vec![mock_source("test.metric", 150.0)]);
        let (tx, mut rx) = mpsc::channel(4);
        let bounty = active_bounty();
        let handles = node.monitor_bonds(vec![bounty], tx);
        // Let it poll twice (poll_interval_secs = 1), then abort.
        tokio::time::sleep(Duration::from_millis(2_500)).await;
        for h in handles { h.abort(); }
        // After abort the task's sender clone is dropped; channel closes.
        // try_recv returns Err regardless of whether channel is empty or disconnected —
        // either way confirms no attestation was buffered.
        assert!(rx.try_recv().is_err(), "no attestation should be sent while goal is not met");
    }
}
