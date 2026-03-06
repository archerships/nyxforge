//! BURN circuit — proves redemption of a bond note after goal achievement.
//!
//! # Public inputs (instance column, rows 0..4)
//!
//! | Row | Value                |
//! |-----|----------------------|
//! |  0  | `nullifier`          |
//! |  1  | `bond_id`            |
//! |  2  | `quorum_result_hash` |
//! |  3  | `payout_commitment`  |
//! |  4  | `payout_amount`      |
//!
//! # Statement proved
//!
//! The prover knows `(bond_note, owner_secret, payout_address, payout_randomness)` such that:
//!
//! ```text
//! nullifier        = Poseidon2(owner_secret, bond_note.serial)
//! payout_amount    = Fp::from(quantity * redemption_value)        [native check]
//! payout_commitment = Poseidon2(
//!                        Poseidon2(payout_amount_fp, payout_address),
//!                        payout_randomness
//!                    )
//! ```
//!
//! `quorum_result_hash` and `payout_amount` are included as public inputs to
//! bind the proof to a specific oracle result and payout value.  The contract
//! verifies externally that `quorum_result_hash` matches the recorded quorum.
//! The arithmetic constraint `payout_amount == quantity * redemption_value` is
//! enforced off-circuit by the caller before creating the proof.

use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};

/// Configuration for [`BurnCircuit`].
#[derive(Clone, Debug)]
pub struct BurnConfig {
    pub poseidon: Pow5Config<Fp, 3, 2>,
    pub state:    [Column<Advice>; 3],
    pub instance: Column<Instance>,
}

/// The BURN circuit.
#[derive(Debug, Default)]
pub struct BurnCircuit {
    // Bond note witnesses
    pub bond_id:           Value<Fp>,
    pub serial:            Value<Fp>,

    // Spending key
    pub owner_secret:      Value<Fp>,

    // Payout note witnesses
    pub payout_address:    Value<Fp>,
    pub payout_amount:     Value<Fp>,
    pub payout_randomness: Value<Fp>,
}

impl BurnCircuit {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Circuit<Fp> for BurnCircuit {
    type Config = BurnConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> BurnConfig {
        let state        = [meta.advice_column(), meta.advice_column(), meta.advice_column()];
        let partial_sbox = meta.advice_column();
        let rc_a         = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        let rc_b         = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        meta.enable_constant(rc_b[0]);

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        let instance = meta.instance_column();
        meta.enable_equality(instance);

        BurnConfig { poseidon, state, instance }
    }

    fn synthesize(&self, config: BurnConfig, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        // ----- Nullifier = Poseidon2(owner_secret, serial) -----

        let (secret_cell, serial_cell) = layouter.assign_region(
            || "load nullifier witnesses",
            |mut region| {
                let a = region.assign_advice(|| "owner_secret", config.state[0], 0, || self.owner_secret)?;
                let b = region.assign_advice(|| "serial",       config.state[1], 0, || self.serial)?;
                Ok((a, b))
            },
        )?;

        let nullifier = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "nul init"),
            )?;
            hasher.hash(layouter.namespace(|| "nul hash"), [secret_cell, serial_cell])?
        };

        // ----- payout_commitment = Poseidon2(Poseidon2(payout_amount, payout_address), payout_randomness) -----
        // h_pay = Poseidon2(payout_amount, payout_address)

        let (amount_cell, addr_cell) = layouter.assign_region(
            || "load payout inner witnesses",
            |mut region| {
                let a = region.assign_advice(|| "payout_amount",   config.state[0], 0, || self.payout_amount)?;
                let b = region.assign_advice(|| "payout_address",  config.state[1], 0, || self.payout_address)?;
                Ok((a, b))
            },
        )?;
        let payout_amount_cell_ref = amount_cell.cell();

        let h_pay = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "h_pay init"),
            )?;
            hasher.hash(layouter.namespace(|| "h_pay hash"), [amount_cell, addr_cell])?
        };

        // Load payout_randomness for the outer hash.
        let rand_cell = layouter.assign_region(
            || "load payout_randomness",
            |mut region| {
                region.assign_advice(
                    || "payout_randomness",
                    config.state[0],
                    0,
                    || self.payout_randomness,
                )
            },
        )?;

        // payout_commitment = Poseidon2(h_pay, payout_randomness)
        let payout_commitment = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "payout_cm init"),
            )?;
            hasher.hash(layouter.namespace(|| "payout_cm hash"), [h_pay, rand_cell])?
        };

        // Load bond_id for the public instance constraint.
        let bond_id_cell = layouter.assign_region(
            || "load bond_id for instance",
            |mut region| {
                region.assign_advice(|| "bond_id", config.state[0], 0, || self.bond_id)
            },
        )?;

        // ----- Constrain public instances -----
        layouter.constrain_instance(nullifier.cell(),          config.instance, 0)?;
        layouter.constrain_instance(bond_id_cell.cell(),       config.instance, 1)?;
        // instance[2] = quorum_result_hash — it's a pass-through public value,
        // not computed in-circuit.  We do NOT constrain it here; the contract
        // checks it against on-chain state externally.
        layouter.constrain_instance(payout_commitment.cell(),  config.instance, 3)?;
        layouter.constrain_instance(payout_amount_cell_ref,    config.instance, 4)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{fp_from_bytes, fp_to_bytes, note_nullifier, poseidon2};
    use halo2_proofs::dev::MockProver;

    const BOND_ID:           [u8; 32] = [0x01u8; 32];
    const SERIAL:            [u8; 32] = [0x55u8; 32];
    const OWNER_SECRET:      [u8; 32] = [0xAAu8; 32];
    const PAYOUT_ADDRESS:    [u8; 32] = [0xCCu8; 32];
    const PAYOUT_RANDOMNESS: [u8; 32] = [0x33u8; 32];
    const QTY:               u64      = 10;
    const REDEMPTION_VALUE:  u64      = 1_000_000;

    fn test_circuit() -> (BurnCircuit, Vec<Vec<Fp>>) {
        let nullifier_fp = note_nullifier(&OWNER_SECRET, &SERIAL);

        let payout_amount = Fp::from(QTY * REDEMPTION_VALUE);
        let h_pay         = poseidon2(payout_amount, fp_from_bytes(&PAYOUT_ADDRESS));
        let payout_cm     = poseidon2(h_pay, fp_from_bytes(&PAYOUT_RANDOMNESS));

        // quorum_result_hash — just a test value (contract checks this externally)
        let quorum_hash_fp = Fp::from(0xbeef_cafe_u64);

        let circuit = BurnCircuit {
            bond_id:           Value::known(fp_from_bytes(&BOND_ID)),
            payout_address:    Value::known(fp_from_bytes(&PAYOUT_ADDRESS)),
            serial:            Value::known(fp_from_bytes(&SERIAL)),
            owner_secret:      Value::known(fp_from_bytes(&OWNER_SECRET)),
            payout_amount:     Value::known(payout_amount),
            payout_randomness: Value::known(fp_from_bytes(&PAYOUT_RANDOMNESS)),
        };

        // Instance: [nullifier, bond_id, quorum_result_hash, payout_commitment, payout_amount]
        let instances = vec![
            nullifier_fp,
            fp_from_bytes(&BOND_ID),
            quorum_hash_fp,                 // instance[2] — not circuit-constrained
            payout_cm,
            payout_amount,
        ];
        (circuit, vec![instances])
    }

    #[test]
    fn burn_circuit_satisfies_constraints() {
        let (circuit, instances) = test_circuit();
        let k = 10;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert_eq!(prover.verify(), Ok(()), "BURN MockProver failed");
    }

    #[test]
    fn burn_circuit_fails_with_wrong_nullifier() {
        let (circuit, mut instances) = test_circuit();
        instances[0][0] = Fp::from(0xdeadbeefu64);
        let k = 10;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert!(prover.verify().is_err(), "should fail with wrong nullifier");
    }

    #[test]
    fn burn_circuit_fails_with_wrong_payout_commitment() {
        let (circuit, mut instances) = test_circuit();
        instances[0][3] = Fp::from(0xdeadbeefu64);
        let k = 10;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert!(prover.verify().is_err(), "should fail with wrong payout commitment");
    }

    // Silence unused fn warning for fp_to_bytes in test imports
    #[allow(dead_code)]
    fn _use_fp_to_bytes(fp: Fp) -> [u8; 32] { fp_to_bytes(fp) }
}
