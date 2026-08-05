//! ZK proof fixtures.
//!
//! Each constructor calls the real `prove()` implementation with fixed witness
//! inputs.  These succeed against the current stub verifier and will continue
//! to succeed once the real Halo2 circuits are wired — the witness values are
//! always valid pre-images.
//!
//! If `prove()` fails for a fixture, that is a test infrastructure bug, not a
//! test failure — the constructors panic with a descriptive message rather than
//! returning `Result`.

use nyxforge_core::bounty::BountyId;
use nyxforge_core::types::Amount;
use nyxforge_zk::{
    burn::{BurnProof, BurnWitness},
    mint::{MintProof, MintWitness},
    transfer::{TransferProof, TransferWitness},
};

use crate::notes::{note_for_bounty, OWNER_KEY, OWNER_SECRET, RECIPIENT_KEY};

// ---------------------------------------------------------------------------
// MINT
// ---------------------------------------------------------------------------

/// Witness for minting `quantity` units of `bounty_id` to [`OWNER_KEY`].
pub fn mint_witness(bounty_id: BountyId, quantity: u64) -> MintWitness {
    MintWitness {
        bounty_id,
        quantity,
        redemption_value: Amount::from_whole(10),
        recipient: OWNER_KEY,
        randomness: [0x07u8; 32],
        serial: [0x03u8; 32],
    }
}

/// A valid MINT proof for `quantity` units of `bounty_id`.
///
/// # Panics
/// If the stub prover rejects the witness (should never happen with valid inputs).
pub fn mint_proof(bounty_id: BountyId, quantity: u64) -> MintProof {
    MintProof::prove(&mint_witness(bounty_id, quantity))
        .expect("fixture MintProof::prove should succeed")
}

// ---------------------------------------------------------------------------
// TRANSFER
// ---------------------------------------------------------------------------

/// Witness transferring 5 units of `bounty_id` from [`OWNER_KEY`] to [`RECIPIENT_KEY`].
pub fn transfer_witness(bounty_id: BountyId) -> TransferWitness {
    TransferWitness {
        old_note: note_for_bounty(bounty_id, 5),
        owner_secret: OWNER_SECRET,
        recipient: RECIPIENT_KEY,
        new_randomness: [0x08u8; 32],
        new_serial: [0x04u8; 32],
    }
}

/// A valid TRANSFER proof: 5 units of `bounty_id`, owner → recipient.
///
/// # Panics
/// If the stub prover rejects the witness.
pub fn transfer_proof(bounty_id: BountyId) -> TransferProof {
    TransferProof::prove(&transfer_witness(bounty_id))
        .expect("fixture TransferProof::prove should succeed")
}

/// TRANSFER witness with zero quantity — proves the prover rejects it.
///
/// Pass to `TransferProof::prove` and assert `is_err()`.
pub fn zero_quantity_transfer_witness(bounty_id: BountyId) -> TransferWitness {
    TransferWitness {
        old_note: note_for_bounty(bounty_id, 0),
        ..transfer_witness(bounty_id)
    }
}

// ---------------------------------------------------------------------------
// BURN (Redemption)
// ---------------------------------------------------------------------------

/// A fixed judge attest key used in burn fixtures.
/// Corresponds to `judge_attest_pk_from_key(BURN_JUDGE_ATTEST_KEY)`.
pub const BURN_JUDGE_ATTEST_KEY: [u8; 32] = [0xEEu8; 32];

/// Witness burning 5 units of `bounty_id` with the given judge attest key.
pub fn burn_witness(bounty_id: BountyId, judge_attest_key: [u8; 32]) -> BurnWitness {
    BurnWitness {
        bounty_note:         note_for_bounty(bounty_id, 5),
        owner_secret:      OWNER_SECRET,
        judge_attest_key,
        payout_address:    [0xCCu8; 32],
        payout_randomness: [0x09u8; 32],
    }
}

/// A valid BURN proof for 5 units of `bounty_id`.
///
/// # Panics
/// If the prover rejects the witness.
pub fn burn_proof(bounty_id: BountyId) -> BurnProof {
    BurnProof::prove(&burn_witness(bounty_id, BURN_JUDGE_ATTEST_KEY))
        .expect("fixture BurnProof::prove should succeed")
}

/// BURN witness with a zeroed attest key.  Use to exercise the path where the
/// attest key is wrong (the resulting `judge_attest_pk` will differ from any
/// registered PK on the bounty).
pub fn burn_witness_wrong_key(bounty_id: BountyId) -> BurnWitness {
    burn_witness(bounty_id, [0u8; 32])
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::Digest;

    fn zero_id() -> BountyId { Digest::zero() }

    #[test]
    fn mint_proof_verifies() {
        let proof = mint_proof(zero_id(), 10);
        assert!(proof.verify().is_ok());
    }

    #[test]
    fn mint_proof_with_empty_bytes_fails_verify() {
        let mut proof = mint_proof(zero_id(), 10);
        proof.proof_bytes.clear();
        assert!(proof.verify().is_err());
    }

    #[test]
    fn transfer_proof_verifies() {
        let proof = transfer_proof(zero_id());
        assert!(proof.verify().is_ok());
    }

    #[test]
    fn transfer_zero_quantity_rejected() {
        let w = zero_quantity_transfer_witness(zero_id());
        assert!(TransferProof::prove(&w).is_err());
    }

    #[test]
    fn burn_proof_verifies() {
        let proof = burn_proof(zero_id());
        assert!(proof.verify().is_ok());
    }

    #[test]
    fn burn_proof_payout_amount_correct() {
        // note has quantity=5, redemption_value=10 DRK → payout = 50 DRK
        let proof = burn_proof(zero_id());
        assert_eq!(proof.payout_amount, Amount::from_whole(50));
    }

    #[test]
    fn transfer_nullifier_matches_note() {
        use crate::notes::default_note;
        let proof = transfer_proof(zero_id());
        let expected = default_note().nullifier(&OWNER_SECRET);
        // The transfer witness uses note_for_bounty(zero_id, 5) which has same
        // serial/owner as default_note — nullifiers must match.
        assert_eq!(proof.nullifier, expected);
    }

    #[test]
    fn mint_commitment_matches_note_commitment() {
        use crate::notes::note_for_bounty;
        let id = zero_id();
        let proof = mint_proof(id, 5);
        // MintProof builds the same note internally — commitment must agree.
        let note = note_for_bounty(id, 5);
        assert_eq!(proof.commitment, note.commitment());
    }
}
