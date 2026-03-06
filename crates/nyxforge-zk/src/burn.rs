//! BURN proof — generates and verifies a real Halo2 BURN proof.
//!
//! Public inputs:
//!   - `nullifier`          : spent note's nullifier (prevents double-redemption)
//!   - `bond_id`            : bond series being redeemed
//!   - `quorum_result_hash` : pass-through hash (not circuit-constrained; verified externally)
//!   - `payout_commitment`  : commitment to the anonymous payout note
//!   - `payout_amount`      : quantity * redemption_value (native arithmetic, enforced off-circuit)

use halo2_proofs::{
    circuit::Value,
    pasta::Fp,
    plonk::{self, SingleVerifier},
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Amount, Digest, Nullifier};
use pasta_curves::EqAffine;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::circuit::burn::BurnCircuit;
use crate::note::BondNote;
use crate::params::BURN_KEYS;
use crate::primitives::{fp_from_bytes, fp_to_bytes, poseidon2};
use crate::ZkError;

pub struct BurnWitness {
    /// The note being redeemed.
    pub bond_note: BondNote,

    /// Owner's secret key (used to derive the nullifier).
    pub owner_secret: [u8; 32],

    /// Hash of the QuorumResult that declared the goal met.
    /// Treated as a pass-through public input; not circuit-constrained.
    pub quorum_result_hash: Digest,

    /// Address of the party receiving the payout (set by the current holder
    /// at redemption time — not stored on the bond).
    pub payout_address: [u8; 32],

    /// Fresh randomness for the anonymous payout note.
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
        if w.bond_note.quantity == 0 {
            return Err(ZkError::InvalidWitness("quantity must be > 0".into()));
        }

        let payout_amount = Amount(
            w.bond_note.quantity
                .checked_mul(w.bond_note.redemption_value.0)
                .ok_or_else(|| ZkError::InvalidWitness("payout overflow".into()))?,
        );

        let nullifier = w.bond_note.nullifier(&w.owner_secret);

        // payout_commitment = Poseidon2(Poseidon2(payout_amount_fp, payout_address), payout_randomness)
        let payout_amount_fp   = Fp::from(payout_amount.0);
        let payout_address_fp  = fp_from_bytes(&w.payout_address);
        let h_pay              = poseidon2(payout_amount_fp, payout_address_fp);
        let payout_cm_fp       = poseidon2(h_pay, fp_from_bytes(&w.payout_randomness));
        let payout_commitment = Digest::from_bytes(fp_to_bytes(payout_cm_fp));

        let nullifier_fp   = fp_from_bytes(nullifier.as_bytes());
        let bond_id_fp     = fp_from_bytes(w.bond_note.bond_id.as_bytes());
        let quorum_hash_fp = fp_from_bytes(w.quorum_result_hash.as_bytes());

        let circuit = BurnCircuit {
            bond_id:           Value::known(bond_id_fp),
            payout_address:    Value::known(payout_address_fp),
            serial:            Value::known(fp_from_bytes(&w.bond_note.serial)),
            owner_secret:      Value::known(fp_from_bytes(&w.owner_secret)),
            payout_amount:     Value::known(payout_amount_fp),
            payout_randomness: Value::known(fp_from_bytes(&w.payout_randomness)),
        };

        // Instance: [nullifier, bond_id, quorum_result_hash, payout_commitment, payout_amount]
        let instances: &[&[Fp]] = &[&[
            nullifier_fp,
            bond_id_fp,
            quorum_hash_fp,
            payout_cm_fp,
            payout_amount_fp,
        ]];
        let keys = &*BURN_KEYS;

        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        plonk::create_proof(&keys.params, &keys.pk, &[circuit], &[instances], OsRng, &mut transcript)
            .map_err(|e| ZkError::ProvingFailed(e.to_string()))?;

        let proof_bytes = transcript.finalize();
        tracing::debug!(
            bond_id = ?w.bond_note.bond_id,
            proof_len = proof_bytes.len(),
            "BURN proof generated"
        );
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
        let nullifier_fp   = fp_from_bytes(self.nullifier.as_bytes());
        let bond_id_fp     = fp_from_bytes(self.bond_id.as_bytes());
        let quorum_hash_fp = fp_from_bytes(self.quorum_result_hash.as_bytes());
        let payout_cm_fp   = fp_from_bytes(self.payout_commitment.as_bytes());
        let payout_amt_fp  = Fp::from(self.payout_amount.0);

        let instances: &[&[Fp]] = &[&[
            nullifier_fp,
            bond_id_fp,
            quorum_hash_fp,
            payout_cm_fp,
            payout_amt_fp,
        ]];
        let keys = &*BURN_KEYS;

        let strategy       = SingleVerifier::new(&keys.params);
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(self.proof_bytes.as_slice());

        plonk::verify_proof(&keys.params, &keys.vk, strategy, &[instances], &mut transcript)
            .map_err(|_| ZkError::VerificationFailed)?;

        tracing::debug!("BURN proof verified");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::{Amount, PublicKey};

    fn test_witness() -> BurnWitness {
        BurnWitness {
            bond_note: BondNote {
                bond_id:          Digest::from_bytes([0x01u8; 32]),
                quantity:         10,
                redemption_value: Amount(1_000_000),
                owner:            PublicKey([0xBBu8; 32]),
                randomness:       [0x42u8; 32],
                serial:           [0x55u8; 32],
            },
            owner_secret:       [0xAAu8; 32],
            quorum_result_hash: Digest::from_bytes([0xDDu8; 32]),
            payout_address:     [0xCCu8; 32],
            payout_randomness:  [0x33u8; 32],
        }
    }

    #[test]
    #[ignore = "slow: generates real Halo2 proof (keygen + prove ~5-20 s)"]
    fn burn_prove_and_verify_roundtrip() {
        let proof = BurnProof::prove(&test_witness()).expect("prove failed");
        proof.verify().expect("verify failed");
    }

    #[test]
    #[ignore = "slow: generates real Halo2 proof"]
    fn burn_verify_rejects_tampered_nullifier() {
        let mut proof = BurnProof::prove(&test_witness()).expect("prove failed");
        proof.nullifier = Digest::from_bytes([0xFFu8; 32]);
        assert!(proof.verify().is_err(), "should reject tampered nullifier");
    }

    #[test]
    fn burn_rejects_zero_quantity() {
        let mut w = test_witness();
        w.bond_note.quantity = 0;
        assert!(BurnProof::prove(&w).is_err());
    }
}
