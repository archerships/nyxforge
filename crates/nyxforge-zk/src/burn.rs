//! BURN proof — generates and verifies a real Halo2 BURN proof.
//!
//! Public inputs:
//!   - `nullifier`         : spent note's nullifier (prevents double-redemption)
//!   - `bounty_id`           : bounty series being redeemed
//!   - `judge_attest_pk`  : Poseidon PK derived from judge's private attest key;
//!                           circuit-constrained (replaces the old unconstrained quorum_result_hash)
//!   - `payout_commitment` : commitment to the anonymous payout note
//!   - `payout_amount`     : quantity * redemption_value (native arithmetic, enforced off-circuit)

use halo2_proofs::{
    circuit::Value,
    pasta::Fp,
    plonk::{self, SingleVerifier},
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use nyxforge_core::bounty::BountyId;
use nyxforge_core::types::{Amount, Digest, Nullifier};
use pasta_curves::EqAffine;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::circuit::burn::BurnCircuit;
use crate::note::BountyNote;
use crate::params::BURN_KEYS;
use crate::primitives::{fp_from_bytes, fp_to_bytes, judge_attest_domain, poseidon2};
use crate::ZkError;

pub struct BurnWitness {
    /// The note being redeemed.
    pub bounty_note: BountyNote,

    /// Owner's secret key (used to derive the nullifier).
    pub owner_secret: [u8; 32],

    /// Judge's per-bounty attest key, received from the judge privately (off-chain).
    ///
    /// This is the **private witness** for the judge attestation constraint:
    /// the circuit proves `Poseidon2(fp(judge_attest_key), domain) == judge_attest_pk`
    /// without revealing `judge_attest_key`.  The bounty holder must receive this
    /// from the judge before they can redeem.  Zero-ize after use.
    pub judge_attest_key: [u8; 32],

    /// Address of the party receiving the payout (set by the current holder
    /// at redemption time — not stored on the bounty).
    pub payout_address: [u8; 32],

    /// Fresh randomness for the anonymous payout note.
    pub payout_randomness: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnProof {
    pub bounty_id:           BountyId,
    pub nullifier:         Nullifier,
    /// `Poseidon2(fp(judge_attest_key), judge_attest_domain())` in Pallas Fp bytes.
    /// Registered on the bounty in `JudgeSpec.judge_attest_pks`.
    /// The settlement contract verifies this is in the bounty's approved attest PK list.
    pub judge_attest_pk:  [u8; 32],
    pub payout_commitment: Digest,
    pub payout_amount:     Amount,
    pub proof_bytes:       Vec<u8>,
}

impl BurnProof {
    pub fn prove(w: &BurnWitness) -> Result<Self, ZkError> {
        if w.bounty_note.quantity == 0 {
            return Err(ZkError::InvalidWitness("quantity must be > 0".into()));
        }

        let payout_amount = Amount(
            w.bounty_note.quantity
                .checked_mul(w.bounty_note.redemption_value.0)
                .ok_or_else(|| ZkError::InvalidWitness("payout overflow".into()))?,
        );

        let nullifier = w.bounty_note.nullifier(&w.owner_secret);

        // payout_commitment = Poseidon2(Poseidon2(payout_amount_fp, payout_address), payout_randomness)
        let payout_amount_fp   = Fp::from(payout_amount.0);
        let payout_address_fp  = fp_from_bytes(&w.payout_address);
        let h_pay              = poseidon2(payout_amount_fp, payout_address_fp);
        let payout_cm_fp       = poseidon2(h_pay, fp_from_bytes(&w.payout_randomness));
        let payout_commitment = Digest::from_bytes(fp_to_bytes(payout_cm_fp));

        let nullifier_fp         = fp_from_bytes(nullifier.as_bytes());
        let bond_id_fp           = fp_from_bytes(w.bounty_note.bounty_id.as_bytes());
        let oracle_attest_key_fp = fp_from_bytes(&w.judge_attest_key);
        let domain_fp            = judge_attest_domain();
        let oracle_attest_pk_fp  = poseidon2(oracle_attest_key_fp, domain_fp);
        let judge_attest_pk     = fp_to_bytes(oracle_attest_pk_fp);

        let circuit = BurnCircuit {
            bounty_id:              Value::known(bond_id_fp),
            serial:               Value::known(fp_from_bytes(&w.bounty_note.serial)),
            owner_secret:         Value::known(fp_from_bytes(&w.owner_secret)),
            judge_attest_key:    Value::known(oracle_attest_key_fp),
            judge_attest_domain: Value::known(domain_fp),
            payout_address:       Value::known(payout_address_fp),
            payout_amount:        Value::known(payout_amount_fp),
            payout_randomness:    Value::known(fp_from_bytes(&w.payout_randomness)),
        };

        // Instance: [nullifier, bounty_id, judge_attest_pk, payout_commitment, payout_amount]
        let instances: &[&[Fp]] = &[&[
            nullifier_fp,
            bond_id_fp,
            oracle_attest_pk_fp,
            payout_cm_fp,
            payout_amount_fp,
        ]];
        let keys = &*BURN_KEYS;

        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        plonk::create_proof(&keys.params, &keys.pk, &[circuit], &[instances], OsRng, &mut transcript)
            .map_err(|e| ZkError::ProvingFailed(e.to_string()))?;

        let proof_bytes = transcript.finalize();
        tracing::debug!(
            bounty_id = ?w.bounty_note.bounty_id,
            proof_len = proof_bytes.len(),
            "BURN proof generated"
        );
        Ok(Self {
            bounty_id: w.bounty_note.bounty_id,
            nullifier,
            judge_attest_pk,
            payout_commitment,
            payout_amount,
            proof_bytes,
        })
    }

    pub fn verify(&self) -> Result<(), ZkError> {
        let nullifier_fp        = fp_from_bytes(self.nullifier.as_bytes());
        let bond_id_fp          = fp_from_bytes(self.bounty_id.as_bytes());
        let oracle_attest_pk_fp = fp_from_bytes(&self.judge_attest_pk);
        let payout_cm_fp        = fp_from_bytes(self.payout_commitment.as_bytes());
        let payout_amt_fp       = Fp::from(self.payout_amount.0);

        let instances: &[&[Fp]] = &[&[
            nullifier_fp,
            bond_id_fp,
            oracle_attest_pk_fp,
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
            bounty_note: BountyNote {
                bounty_id:          Digest::from_bytes([0x01u8; 32]),
                quantity:         10,
                redemption_value: Amount(1_000_000),
                owner:            PublicKey([0xBBu8; 32]),
                randomness:       [0x42u8; 32],
                serial:           [0x55u8; 32],
            },
            owner_secret:      [0xAAu8; 32],
            judge_attest_key: [0xEEu8; 32],
            payout_address:    [0xCCu8; 32],
            payout_randomness: [0x33u8; 32],
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
        w.bounty_note.quantity = 0;
        assert!(BurnProof::prove(&w).is_err());
    }
}
