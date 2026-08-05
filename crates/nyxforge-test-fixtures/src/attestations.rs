//! Judge attestation and quorum result fixtures.
//!
//! All constructors use fixed timestamps sourced from
//! `chrono::DateTime::from_timestamp(0, 0)` (Unix epoch) so they are
//! deterministic and do not depend on wall-clock time.

use chrono::{DateTime, Utc};

use nyxforge_core::bounty::BountyId;
use nyxforge_core::judge_spec::{JudgeAttestation, QuorumResult};
use nyxforge_core::types::{Digest, PublicKey};

use crate::bounties::{ORACLE_KEY_A, ORACLE_KEY_B, ORACLE_KEY_C};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn epoch() -> DateTime<Utc> {
    DateTime::from_timestamp(0, 0).unwrap()
}

/// Build a single attestation.
fn make_attestation(
    bounty_id: BountyId,
    judge_key: PublicKey,
    goal_met: bool,
) -> JudgeAttestation {
    JudgeAttestation {
        bounty_id,
        goal_met,
        evidence_hash: Digest::zero(),
        evidence_uri: if goal_met {
            Some("https://example.com/evidence".into())
        } else {
            None
        },
        judge_key,
        // Stub signature — real Ed25519 signing is not wired yet.
        signature: vec![0xDE, 0xAD, 0xBE, 0xEF],
        attested_at: epoch(),
    }
}

// ---------------------------------------------------------------------------
// Single attestation constructors
// ---------------------------------------------------------------------------

/// Attestation from judge A asserting the goal was met.
pub fn goal_met_attestation(bounty_id: BountyId) -> JudgeAttestation {
    make_attestation(bounty_id, ORACLE_KEY_A, true)
}

/// Attestation from judge A asserting the goal was NOT met.
pub fn goal_not_met_attestation(bounty_id: BountyId) -> JudgeAttestation {
    make_attestation(bounty_id, ORACLE_KEY_A, false)
}

/// Attestation from a specified judge and key, asserting goal met.
pub fn attestation_from(bounty_id: BountyId, judge_key: PublicKey, goal_met: bool) -> JudgeAttestation {
    make_attestation(bounty_id, judge_key, goal_met)
}

// ---------------------------------------------------------------------------
// QuorumResult constructors
// ---------------------------------------------------------------------------

/// A 3-of-3 quorum result with unanimous `goal_met = true`.
///
/// Uses judge keys A, B, C — one attestation each.
pub fn quorum_met(bounty_id: BountyId) -> QuorumResult {
    QuorumResult {
        bounty_id,
        goal_met: true,
        attestations: vec![
            make_attestation(bounty_id, ORACLE_KEY_A, true),
            make_attestation(bounty_id, ORACLE_KEY_B, true),
            make_attestation(bounty_id, ORACLE_KEY_C, true),
        ],
        finalised_at: epoch(),
    }
}

/// A 3-of-3 quorum result with unanimous `goal_met = false`.
pub fn quorum_not_met(bounty_id: BountyId) -> QuorumResult {
    QuorumResult {
        bounty_id,
        goal_met: false,
        attestations: vec![
            make_attestation(bounty_id, ORACLE_KEY_A, false),
            make_attestation(bounty_id, ORACLE_KEY_B, false),
            make_attestation(bounty_id, ORACLE_KEY_C, false),
        ],
        finalised_at: epoch(),
    }
}

/// A 1-of-1 quorum result — matches [`single_judge_spec`][crate::bounties::single_judge_spec].
pub fn single_judge_quorum_met(bounty_id: BountyId) -> QuorumResult {
    QuorumResult {
        bounty_id,
        goal_met: true,
        attestations: vec![make_attestation(bounty_id, ORACLE_KEY_A, true)],
        finalised_at: epoch(),
    }
}

/// Compute the blake3 hash of a [`QuorumResult`] as used in the BURN circuit.
///
/// Mirrors the derivation in `nyxforge_zk::burn`.
pub fn quorum_result_hash(qr: &QuorumResult) -> Digest {
    let mut h = blake3::Hasher::new();
    h.update(b"nyxforge::quorum_result");
    h.update(qr.bounty_id.as_bytes());
    h.update(&[qr.goal_met as u8]);
    h.update(&(qr.attestations.len() as u64).to_le_bytes());
    Digest::from(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::Digest;

    fn zero_id() -> BountyId { Digest::zero() }

    #[test]
    fn quorum_met_is_consistent() {
        let qr = quorum_met(zero_id());
        assert!(qr.is_consistent());
        assert!(qr.goal_met);
    }

    #[test]
    fn quorum_not_met_is_consistent() {
        let qr = quorum_not_met(zero_id());
        assert!(qr.is_consistent());
        assert!(!qr.goal_met);
    }

    #[test]
    fn mixed_attestations_are_not_consistent() {
        let mut qr = quorum_met(zero_id());
        qr.attestations.push(make_attestation(zero_id(), ORACLE_KEY_C, false));
        assert!(!qr.is_consistent());
    }

    #[test]
    fn quorum_result_hash_is_deterministic() {
        let qr = quorum_met(zero_id());
        assert_eq!(quorum_result_hash(&qr), quorum_result_hash(&qr));
    }

    #[test]
    fn met_and_not_met_hashes_differ() {
        let id = zero_id();
        assert_ne!(quorum_result_hash(&quorum_met(id)), quorum_result_hash(&quorum_not_met(id)));
    }
}
