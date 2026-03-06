//! MINT proof — generates and verifies a real Halo2 MINT proof.
//!
//! Public inputs:
//!   - `commitment` : the new note's commitment
//!   - `bond_id`    : bond series being minted
//!
//! Private witness:
//!   - `quantity`   : number of units
//!   - `owner_pk`   : recipient's public key
//!   - `randomness` : blinding factor
//!   - `serial`     : unique serial number (stored in note; not used in MINT circuit)

use halo2_proofs::{
    circuit::Value,
    pasta::Fp,
    plonk::{self, SingleVerifier},
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Amount, Digest};
use pasta_curves::EqAffine;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::circuit::mint::MintCircuit;
use crate::note::BondNote;
use crate::params::MINT_KEYS;
use crate::primitives::fp_from_bytes;
use crate::ZkError;

/// Everything the prover knows (kept secret).
#[derive(Debug)]
pub struct MintWitness {
    pub bond_id:          BondId,
    pub quantity:         u64,
    pub redemption_value: Amount,
    pub recipient:        nyxforge_core::types::PublicKey,
    pub randomness:       [u8; 32],
    pub serial:           [u8; 32],
}

/// The generated proof and its public outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintProof {
    /// Public: commitment to the new note (Poseidon-based).
    pub commitment: Digest,

    /// Public: bond series identifier.
    pub bond_id: BondId,

    /// Halo2 IPA proof bytes (Blake2b transcript).
    pub proof_bytes: Vec<u8>,
}

impl MintProof {
    /// Generate a MINT proof from the given witness.
    pub fn prove(witness: &MintWitness) -> Result<Self, ZkError> {
        let note = BondNote {
            bond_id:          witness.bond_id,
            quantity:         witness.quantity,
            redemption_value: witness.redemption_value,
            owner:            witness.recipient.clone(),
            randomness:       witness.randomness,
            serial:           witness.serial,
        };
        // commitment() calls primitives::note_commitment internally.
        let commitment   = note.commitment();
        let bond_id_fp   = fp_from_bytes(witness.bond_id.as_bytes());
        let commitment_fp = fp_from_bytes(commitment.as_bytes());

        let circuit = MintCircuit {
            bond_id:    Value::known(bond_id_fp),
            quantity:   Value::known(Fp::from(witness.quantity)),
            owner_pk:   Value::known(fp_from_bytes(&witness.recipient.0)),
            randomness: Value::known(fp_from_bytes(&witness.randomness)),
        };

        let instances: &[&[Fp]] = &[&[commitment_fp, bond_id_fp]];
        let keys = &*MINT_KEYS;

        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        plonk::create_proof(&keys.params, &keys.pk, &[circuit], &[instances], OsRng, &mut transcript)
            .map_err(|e| ZkError::ProvingFailed(e.to_string()))?;

        let proof_bytes = transcript.finalize();
        tracing::debug!(?commitment, proof_len = proof_bytes.len(), "MINT proof generated");
        Ok(Self { commitment, bond_id: witness.bond_id, proof_bytes })
    }

    /// Verify a MINT proof against its public inputs.
    pub fn verify(&self) -> Result<(), ZkError> {
        let commitment_fp = fp_from_bytes(self.commitment.as_bytes());
        let bond_id_fp    = fp_from_bytes(self.bond_id.as_bytes());

        let instances: &[&[Fp]] = &[&[commitment_fp, bond_id_fp]];
        let keys = &*MINT_KEYS;

        let strategy    = SingleVerifier::new(&keys.params);
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(self.proof_bytes.as_slice());

        plonk::verify_proof(&keys.params, &keys.vk, strategy, &[instances], &mut transcript)
            .map_err(|_| ZkError::VerificationFailed)?;

        tracing::debug!("MINT proof verified");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::{Amount, PublicKey};

    fn test_witness() -> MintWitness {
        MintWitness {
            bond_id:          Digest::from_bytes([0x01u8; 32]),
            quantity:         10,
            redemption_value: Amount(1_000_000),
            recipient:        PublicKey([0xBBu8; 32]),
            randomness:       [0x42u8; 32],
            serial:           [0x55u8; 32],
        }
    }

    #[test]
    #[ignore = "slow: generates real Halo2 proof (keygen + prove ~5-15 s)"]
    fn mint_prove_and_verify_roundtrip() {
        let proof = MintProof::prove(&test_witness()).expect("prove failed");
        proof.verify().expect("verify failed");
    }

    #[test]
    #[ignore = "slow: generates real Halo2 proof"]
    fn mint_verify_rejects_tampered_commitment() {
        let mut proof = MintProof::prove(&test_witness()).expect("prove failed");
        // Flip a byte in the commitment: the stored commitment no longer matches
        // what the circuit proved.
        proof.commitment = Digest::from_bytes([0xFFu8; 32]);
        assert!(proof.verify().is_err(), "should reject tampered commitment");
    }
}
