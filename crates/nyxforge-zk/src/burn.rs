//! BURN circuit — proves redemption of a bond note after goal achievement.
//!
//! Public inputs:
//!   - `nullifier`          : spent bond note (prevents double redemption)
//!   - `bond_id`            : bond series being redeemed
//!   - `quorum_result_hash` : blake3 of the QuorumResult (oracle attestation)
//!   - `payout_commitment`  : commitment to the DRK payout note (anonymous)
//!
//! Private witness:
//!   - bond note plaintext + owner's secret key
//!   - payout note plaintext (same value, new randomness)
//!
//! Constraints:
//!   - nullifier = PRF(owner_secret, bond_note.serial)
//!   - payout_commitment contains quantity * redemption_value
//!   - bond_note is a valid pre-image of a known commitment (Merkle path)

use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Amount, Digest, Nullifier};
use serde::{Deserialize, Serialize};

use crate::note::BondNote;
use crate::ZkError;

pub struct BurnWitness {
    pub bond_note:         BondNote,
    pub owner_secret:      [u8; 32],

    /// hash of the QuorumResult that declared the goal met.
    pub quorum_result_hash: Digest,

    /// Randomness for the anonymous payout note.
    pub payout_randomness: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnProof {
    pub bond_id:            BondId,
    pub nullifier:          Nullifier,
    pub quorum_result_hash: Digest,
    pub payout_commitment:  Digest,
    pub payout_amount:      Amount,
    pub proof_bytes:        Vec<u8>,
}

impl BurnProof {
    pub fn prove(w: &BurnWitness) -> Result<Self, ZkError> {
        let nullifier = w.bond_note.nullifier(&w.owner_secret);

        // Payout amount = quantity * redemption_value_per_unit
        let payout_amount = Amount(
            w.bond_note.quantity
                .checked_mul(w.bond_note.redemption_value.0)
                .ok_or_else(|| ZkError::InvalidWitness("payout overflow".into()))?,
        );

        // Commitment to the payout note (anonymous DRK note for the holder).
        let payout_commitment = {
            let mut h = blake3::Hasher::new();
            h.update(b"nyxforge::payout_commit");
            h.update(&payout_amount.0.to_le_bytes());
            h.update(&w.bond_note.owner.0);
            h.update(&w.payout_randomness);
            Digest::from(h.finalize())
        };

        let proof_bytes = {
            let mut h = blake3::Hasher::new();
            h.update(b"MOCK_BURN_PROOF");
            h.update(nullifier.as_bytes());
            h.update(payout_commitment.as_bytes());
            h.update(w.quorum_result_hash.as_bytes());
            h.finalize().as_bytes().to_vec()
        };

        Ok(Self {
            bond_id: w.bond_note.bond_id,
            nullifier,
            quorum_result_hash: w.quorum_result_hash,
            payout_commitment,
            payout_amount,
            proof_bytes,
        })
    }

    pub fn verify(&self) -> Result<(), ZkError> {
        if self.proof_bytes.is_empty() {
            return Err(ZkError::VerificationFailed);
        }
        Ok(())
    }
}
