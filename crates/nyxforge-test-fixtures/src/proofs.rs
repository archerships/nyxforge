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

use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Amount, Digest};
use nyxforge_zk::{
    burn::{BurnProof, BurnWitness},
    mint::{MintProof, MintWitness},
    transfer::{TransferProof, TransferWitness},
};

use crate::notes::{note_for_bond, OWNER_KEY, OWNER_SECRET, RECIPIENT_KEY};

// ---------------------------------------------------------------------------
// MINT
// ---------------------------------------------------------------------------

/// Witness for minting `quantity` units of `bond_id` to [`OWNER_KEY`].
pub fn mint_witness(bond_id: BondId, quantity: u64) -> MintWitness {
    MintWitness {
        bond_id,
        quantity,
        redemption_value: Amount::from_whole(10),
        recipient: OWNER_KEY,
        randomness: [0x07u8; 32],
        serial: [0x03u8; 32],
    }
}

/// A valid MINT proof for `quantity` units of `bond_id`.
///
/// # Panics
/// If the stub prover rejects the witness (should never happen with valid inputs).
pub fn mint_proof(bond_id: BondId, quantity: u64) -> MintProof {
    MintProof::prove(&mint_witness(bond_id, quantity))
        .expect("fixture MintProof::prove should succeed")
}

// ---------------------------------------------------------------------------
// TRANSFER
// ---------------------------------------------------------------------------

/// Witness transferring 5 units of `bond_id` from [`OWNER_KEY`] to [`RECIPIENT_KEY`].
pub fn transfer_witness(bond_id: BondId) -> TransferWitness {
    TransferWitness {
        old_note: note_for_bond(bond_id, 5),
        owner_secret: OWNER_SECRET,
        recipient: RECIPIENT_KEY,
        new_randomness: [0x08u8; 32],
        new_serial: [0x04u8; 32],
    }
}

/// A valid TRANSFER proof: 5 units of `bond_id`, owner → recipient.
///
/// # Panics
/// If the stub prover rejects the witness.
pub fn transfer_proof(bond_id: BondId) -> TransferProof {
    TransferProof::prove(&transfer_witness(bond_id))
        .expect("fixture TransferProof::prove should succeed")
}

/// TRANSFER witness with zero quantity — proves the prover rejects it.
///
/// Pass to `TransferProof::prove` and assert `is_err()`.
pub fn zero_quantity_transfer_witness(bond_id: BondId) -> TransferWitness {
    TransferWitness {
        old_note: note_for_bond(bond_id, 0),
        ..transfer_witness(bond_id)
    }
}

// ---------------------------------------------------------------------------
// BURN (Redemption)
// ---------------------------------------------------------------------------

/// Witness burning 5 units of `bond_id` with the given quorum hash.
pub fn burn_witness(bond_id: BondId, quorum_hash: Digest) -> BurnWitness {
    BurnWitness {
        bond_note:          note_for_bond(bond_id, 5),
        owner_secret:       OWNER_SECRET,
        quorum_result_hash: quorum_hash,
        payout_address:     [0xCCu8; 32],
        payout_randomness:  [0x09u8; 32],
    }
}

/// A valid BURN proof for 5 units of `bond_id`.
///
/// # Panics
/// If the stub prover rejects the witness.
pub fn burn_proof(bond_id: BondId, quorum_hash: Digest) -> BurnProof {
    BurnProof::prove(&burn_witness(bond_id, quorum_hash))
        .expect("fixture BurnProof::prove should succeed")
}

/// BURN witness with an empty quorum hash — always produces a valid witness
/// (the hash is just bytes).  Use as a "wrong hash" case by comparing the
/// resulting proof's `quorum_result_hash` against a different known hash.
pub fn burn_witness_wrong_hash(bond_id: BondId) -> BurnWitness {
    burn_witness(bond_id, Digest::zero())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::Digest;

    fn zero_id() -> BondId { Digest::zero() }
    fn zero_hash() -> Digest { Digest::zero() }

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
        let proof = burn_proof(zero_id(), zero_hash());
        assert!(proof.verify().is_ok());
    }

    #[test]
    fn burn_proof_payout_amount_correct() {
        // note has quantity=5, redemption_value=10 DRK → payout = 50 DRK
        let proof = burn_proof(zero_id(), zero_hash());
        assert_eq!(proof.payout_amount, Amount::from_whole(50));
    }

    #[test]
    fn transfer_nullifier_matches_note() {
        use crate::notes::default_note;
        let proof = transfer_proof(zero_id());
        let expected = default_note().nullifier(&OWNER_SECRET);
        // The transfer witness uses note_for_bond(zero_id, 5) which has same
        // serial/owner as default_note — nullifiers must match.
        assert_eq!(proof.nullifier, expected);
    }

    #[test]
    fn mint_commitment_matches_note_commitment() {
        use crate::notes::note_for_bond;
        let id = zero_id();
        let proof = mint_proof(id, 5);
        // MintProof builds the same note internally — commitment must agree.
        let note = note_for_bond(id, 5);
        assert_eq!(proof.commitment, note.commitment());
    }
}
