//! Oracle node: fetches public data, evaluates goal metrics, and posts attestations.
//!
//! An oracle node:
//!   1. Monitors registered bond series for unverified goals near their deadline.
//!   2. Fetches data from registered data adapters (HTTP APIs, IPFS, etc.).
//!   3. Evaluates the GoalMetric predicate.
//!   4. Signs an OracleAttestation with its Ed25519 key.
//!   5. Posts the attestation to the P2P network.
//!   6. Stakes collateral to back its claim; gets slashed if fraudulent.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use nyxforge_core::bond::{Bond, BondId, BondState};
use nyxforge_core::oracle_spec::OracleAttestation;
use nyxforge_core::types::{Digest, PublicKey};

use crate::verifier::{DataSource, VerificationResult};

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

    /// Evaluate a bond's goal and produce a signed attestation.
    pub async fn evaluate(&self, bond: &Bond) -> Result<OracleAttestation> {
        if bond.state != BondState::Active {
            anyhow::bail!("bond is not ACTIVE");
        }

        // Try each registered data source.
        let mut last_err = None;
        for source in &self.sources {
            if !source.supports(&bond.goal.metric.data_id) {
                continue;
            }
            match source.fetch(&bond.goal.metric.data_id).await {
                Ok(value) => {
                    let goal_met = bond.goal.metric.operator.evaluate(value, bond.goal.metric.threshold);
                    let evidence_hash = self.hash_evidence(&bond.goal.metric.data_id, value);
                    let attestation = self.sign_attestation(bond.id, goal_met, evidence_hash);
                    info!(
                        bond_id = ?bond.id,
                        goal_met,
                        value = %value,
                        "attestation produced",
                    );
                    return Ok(attestation);
                }
                Err(e) => {
                    warn!("data source error: {e}");
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no data source for {}", bond.goal.metric.data_id)))
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
}

