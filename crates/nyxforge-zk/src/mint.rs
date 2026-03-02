//! MINT circuit — proves creation of a valid bond note.
//!
//! Public inputs:
//!   - `commitment`  : the new note's commitment
//!   - `bond_id`     : bond series being minted
//!
//! Private witness:
//!   - `quantity`    : number of units
//!   - `owner`       : recipient's public key
//!   - `randomness`  : blinding factor
//!   - `serial`      : unique serial number

use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Amount, Digest, PublicKey};
use serde::{Deserialize, Serialize};

use crate::note::BondNote;
use crate::ZkError;

/// Everything the prover knows (kept secret).
#[derive(Debug)]
pub struct MintWitness {
    pub bond_id:          BondId,
    pub quantity:         u64,
    pub redemption_value: Amount,
    pub recipient:        PublicKey,
    pub randomness:       [u8; 32],
    pub serial:           [u8; 32],
}

/// The generated proof and its public outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintProof {
    /// Public: commitment to the new note.
    pub commitment: Digest,

    /// Public: bond series identifier.
    pub bond_id: BondId,

    /// Opaque proof bytes (Halo2 / DarkFi zkVM encoded).
    pub proof_bytes: Vec<u8>,
}

impl MintProof {
    /// Generate a MINT proof from the given witness.
    ///
    /// TODO: replace blake3-based stub with real DarkFi zkVM call.
    pub fn prove(witness: &MintWitness) -> Result<Self, ZkError> {
        let note = BondNote {
            bond_id:          witness.bond_id,
            quantity:         witness.quantity,
            redemption_value: witness.redemption_value,
            owner:            witness.recipient.clone(),
            randomness:       witness.randomness,
            serial:           witness.serial,
        };

        let commitment = note.commitment();

        // Placeholder: real proof would invoke darkfi_sdk::zk::Proof::create(...)
        let proof_bytes = {
            let mut h = blake3::Hasher::new();
            h.update(b"MOCK_MINT_PROOF");
            h.update(commitment.as_bytes());
            h.finalize().as_bytes().to_vec()
        };

        tracing::debug!(?commitment, "mint proof generated (stub)");

        Ok(Self { commitment, bond_id: witness.bond_id, proof_bytes })
    }

    /// Verify a MINT proof against its public inputs.
    pub fn verify(&self) -> Result<(), ZkError> {
        // Placeholder: real verification would call darkfi_sdk::zk::Proof::verify(...)
        if self.proof_bytes.is_empty() {
            return Err(ZkError::VerificationFailed);
        }
        tracing::debug!("mint proof verified (stub)");
        Ok(())
    }
}
