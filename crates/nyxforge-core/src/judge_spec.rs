//! Judge attestation types — shared between the judge crate and the contract crate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::bounty::BountyId;
use crate::types::{Digest, PublicKey};

/// A per-bounty capability token sent privately from the judge to the bounty holder.
///
/// # Privacy model (Option A — ZK judge attestation)
///
/// Instead of posting attestations to the public AO ledger, the judge:
/// 1. Verifies that the bounty's goal conditions are met.
/// 2. Computes `bond_attest_key = blake3(master_sk ‖ bounty_id ‖ ATTEST_DOMAIN)`.
/// 3. Sends this token to the bounty holder over an encrypted channel (DarkFi P2P).
///
/// The bounty holder uses `bond_attest_key` as a **private witness** in the BURN
/// ZK circuit, which proves knowledge of the attest key without revealing it.
/// The `judge_attest_pk` (the Poseidon-derived public key) is the public
/// instance[2] of the BURN proof — never the raw attest key.
///
/// Zero-ized on drop to protect the judge's secret from memory scraping.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct JudgeAttestKey {
    /// The private per-bounty attest key (NEVER publish this).
    /// Derived as `blake3(oracle_master_sk ‖ bounty_id ‖ "nyxforge::oracle::attest::v1")`.
    pub bond_attest_key: [u8; 32],

    /// `Poseidon2(fp(bond_attest_key), judge_attest_domain())` — little-endian Pallas bytes.
    /// Registered on the bounty in `JudgeSpec.judge_attest_pks`.
    /// Used as BURN circuit instance[2]; reveals which judge endorsed this bounty without
    /// revealing when or what evidence they observed.
    pub judge_attest_pk: [u8; 32],
}

/// A signed statement from an judge node about whether a goal has been met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeAttestation {
    /// Which bounty series this attestation covers.
    pub bounty_id: BountyId,

    /// Whether the judge believes the goal has been achieved.
    pub goal_met: bool,

    /// Hash of the supporting evidence (e.g. SHA-256 of a PDF report).
    pub evidence_hash: Digest,

    /// URI where the evidence can be retrieved (not required to be public).
    pub evidence_uri: Option<String>,

    /// The judge's public key.
    pub judge_key: PublicKey,

    /// Ed25519 signature over `bincode(bounty_id || goal_met || evidence_hash || timestamp)`.
    pub signature: Vec<u8>,

    pub attested_at: DateTime<Utc>,
}

/// Aggregated result once a quorum of attestations has been collected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumResult {
    pub bounty_id:      BountyId,
    pub goal_met:     bool,
    pub attestations: Vec<JudgeAttestation>,
    pub finalised_at: DateTime<Utc>,
}

impl QuorumResult {
    /// Check that all attestations agree and signatures are consistent.
    pub fn is_consistent(&self) -> bool {
        self.attestations.iter().all(|a| a.goal_met == self.goal_met)
    }
}
