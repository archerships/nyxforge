//! Oracle attestation types — shared between the oracle crate and the contract crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::bond::BondId;
use crate::types::{Digest, PublicKey};

/// A signed statement from an oracle node about whether a goal has been met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAttestation {
    /// Which bond series this attestation covers.
    pub bond_id: BondId,

    /// Whether the oracle believes the goal has been achieved.
    pub goal_met: bool,

    /// Hash of the supporting evidence (e.g. SHA-256 of a PDF report).
    pub evidence_hash: Digest,

    /// URI where the evidence can be retrieved (not required to be public).
    pub evidence_uri: Option<String>,

    /// The oracle's public key.
    pub oracle_key: PublicKey,

    /// Ed25519 signature over `bincode(bond_id || goal_met || evidence_hash || timestamp)`.
    pub signature: Vec<u8>,

    pub attested_at: DateTime<Utc>,
}

/// Aggregated result once a quorum of attestations has been collected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumResult {
    pub bond_id:      BondId,
    pub goal_met:     bool,
    pub attestations: Vec<OracleAttestation>,
    pub finalised_at: DateTime<Utc>,
}

impl QuorumResult {
    /// Check that all attestations agree and signatures are consistent.
    pub fn is_consistent(&self) -> bool {
        self.attestations.iter().all(|a| a.goal_met == self.goal_met)
    }
}
