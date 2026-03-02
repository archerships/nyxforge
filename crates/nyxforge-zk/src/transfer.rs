//! TRANSFER circuit — proves anonymous ownership transfer of a bond note.
//!
//! Public inputs:
//!   - `nullifier`       : spent note's nullifier (prevents double-spend)
//!   - `new_commitment`  : commitment to the new recipient's note
//!   - `bond_id`         : must match old and new notes
//!
//! Private witness:
//!   - old note plaintext + owner's secret key
//!   - new note plaintext (quantity, recipient pubkey, fresh randomness/serial)
//!
//! Constraints (informally):
//!   - `old_note.quantity == new_note.quantity`
//!   - `old_note.bond_id  == new_note.bond_id`
//!   - `nullifier == PRF(owner_secret, old_note.serial)`
//!   - `old_commitment` is a valid commitment (pre-image check)

use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Amount, Digest, Nullifier, PublicKey};
use serde::{Deserialize, Serialize};

use crate::note::BondNote;
use crate::ZkError;

pub struct TransferWitness {
    /// The note being consumed.
    pub old_note: BondNote,

    /// Owner's secret key (used to derive the nullifier).
    pub owner_secret: [u8; 32],

    /// Recipient's public key.
    pub recipient: PublicKey,

    /// Fresh randomness for the new note.
    pub new_randomness: [u8; 32],

    /// Fresh serial number for the new note.
    pub new_serial: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProof {
    pub bond_id:        BondId,
    pub nullifier:      Nullifier,
    pub new_commitment: Digest,
    pub proof_bytes:    Vec<u8>,
}

impl TransferProof {
    pub fn prove(w: &TransferWitness) -> Result<Self, ZkError> {
        if w.old_note.quantity == 0 {
            return Err(ZkError::InvalidWitness("quantity must be > 0".into()));
        }

        let nullifier = w.old_note.nullifier(&w.owner_secret);

        let new_note = BondNote {
            bond_id:          w.old_note.bond_id,
            quantity:         w.old_note.quantity,
            redemption_value: w.old_note.redemption_value,
            owner:            w.recipient.clone(),
            randomness:       w.new_randomness,
            serial:           w.new_serial,
        };
        let new_commitment = new_note.commitment();

        // Placeholder: real proof would invoke zkVM
        let proof_bytes = {
            let mut h = blake3::Hasher::new();
            h.update(b"MOCK_TRANSFER_PROOF");
            h.update(nullifier.as_bytes());
            h.update(new_commitment.as_bytes());
            h.finalize().as_bytes().to_vec()
        };

        Ok(Self {
            bond_id: w.old_note.bond_id,
            nullifier,
            new_commitment,
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
