//! TRANSFER proof — generates and verifies a real Halo2 TRANSFER proof.
//!
//! Public inputs:
//!   - `nullifier`       : spent note's nullifier (prevents double-spend)
//!   - `new_commitment`  : commitment to the new recipient's note
//!   - `bond_id`         : must match old and new notes

use halo2_proofs::{
    circuit::Value,
    pasta::Fp,
    plonk::{self, SingleVerifier},
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Digest, Nullifier, PublicKey};
use pasta_curves::EqAffine;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::circuit::transfer::TransferCircuit;
use crate::note::BondNote;
use crate::params::TRANSFER_KEYS;
use crate::primitives::{fp_from_bytes, note_commitment};
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

        let nullifier      = w.old_note.nullifier(&w.owner_secret);
        let new_commitment = {
            let fp = note_commitment(
                w.old_note.bond_id.as_bytes(),
                w.old_note.quantity,
                &w.recipient.0,
                &w.new_randomness,
            );
            Digest::from_bytes(crate::primitives::fp_to_bytes(fp))
        };

        let bond_id_fp     = fp_from_bytes(w.old_note.bond_id.as_bytes());
        let nullifier_fp   = fp_from_bytes(nullifier.as_bytes());
        let new_cm_fp      = fp_from_bytes(new_commitment.as_bytes());

        let circuit = TransferCircuit {
            old_bond_id:    Value::known(bond_id_fp),
            old_quantity:   Value::known(Fp::from(w.old_note.quantity)),
            old_owner_pk:   Value::known(fp_from_bytes(&w.old_note.owner.0)),
            old_randomness: Value::known(fp_from_bytes(&w.old_note.randomness)),
            old_serial:     Value::known(fp_from_bytes(&w.old_note.serial)),
            owner_secret:   Value::known(fp_from_bytes(&w.owner_secret)),
            new_owner_pk:   Value::known(fp_from_bytes(&w.recipient.0)),
            new_randomness: Value::known(fp_from_bytes(&w.new_randomness)),
        };

        let instances: &[&[Fp]] = &[&[nullifier_fp, new_cm_fp, bond_id_fp]];
        let keys = &*TRANSFER_KEYS;

        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        plonk::create_proof(&keys.params, &keys.pk, &[circuit], &[instances], OsRng, &mut transcript)
            .map_err(|e| ZkError::ProvingFailed(e.to_string()))?;

        let proof_bytes = transcript.finalize();
        tracing::debug!(bond_id = ?w.old_note.bond_id, proof_len = proof_bytes.len(), "TRANSFER proof generated");
        Ok(Self { bond_id: w.old_note.bond_id, nullifier, new_commitment, proof_bytes })
    }

    pub fn verify(&self) -> Result<(), ZkError> {
        let nullifier_fp = fp_from_bytes(self.nullifier.as_bytes());
        let new_cm_fp    = fp_from_bytes(self.new_commitment.as_bytes());
        let bond_id_fp   = fp_from_bytes(self.bond_id.as_bytes());

        let instances: &[&[Fp]] = &[&[nullifier_fp, new_cm_fp, bond_id_fp]];
        let keys = &*TRANSFER_KEYS;

        let strategy       = SingleVerifier::new(&keys.params);
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(self.proof_bytes.as_slice());

        plonk::verify_proof(&keys.params, &keys.vk, strategy, &[instances], &mut transcript)
            .map_err(|_| ZkError::VerificationFailed)?;

        tracing::debug!("TRANSFER proof verified");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::Amount;

    fn test_witness() -> TransferWitness {
        TransferWitness {
            old_note: BondNote {
                bond_id:          Digest::from_bytes([0x01u8; 32]),
                quantity:         10,
                redemption_value: Amount(1_000_000),
                owner:            PublicKey([0xBBu8; 32]),
                randomness:       [0x42u8; 32],
                serial:           [0x55u8; 32],
            },
            owner_secret:   [0xAAu8; 32],
            recipient:      PublicKey([0xCCu8; 32]),
            new_randomness: [0x77u8; 32],
            new_serial:     [0x88u8; 32],
        }
    }

    #[test]
    #[ignore = "slow: generates real Halo2 proof (keygen + prove ~5-20 s)"]
    fn transfer_prove_and_verify_roundtrip() {
        let proof = TransferProof::prove(&test_witness()).expect("prove failed");
        proof.verify().expect("verify failed");
    }

    #[test]
    #[ignore = "slow: generates real Halo2 proof"]
    fn transfer_verify_rejects_tampered_nullifier() {
        let mut proof = TransferProof::prove(&test_witness()).expect("prove failed");
        proof.nullifier = Digest::from_bytes([0xFFu8; 32]);
        assert!(proof.verify().is_err(), "should reject tampered nullifier");
    }

    #[test]
    fn transfer_rejects_zero_quantity() {
        let mut w = test_witness();
        w.old_note.quantity = 0;
        assert!(TransferProof::prove(&w).is_err());
    }
}
