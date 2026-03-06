//! MINT circuit — proves a new bond note commitment is well-formed.
//!
//! # Public inputs (instance column, rows 0..1)
//!
//! | Row | Value        |
//! |-----|--------------|
//! |  0  | `commitment` |
//! |  1  | `bond_id`    |
//!
//! # Statement proved
//!
//! The prover knows `(bond_id, quantity, owner_pk, randomness)` such that:
//!
//! ```text
//! h1         = Poseidon2(bond_id, quantity)
//! h2         = Poseidon2(owner_pk, randomness)
//! commitment = Poseidon2(h1, h2)
//! ```
//!
//! The public `bond_id` and computed `commitment` are constrained equal to
//! the declared instance values.

use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};

/// Configuration for [`MintCircuit`].
#[derive(Clone, Debug)]
pub struct MintConfig {
    pub poseidon: Pow5Config<Fp, 3, 2>,
    pub state:    [Column<Advice>; 3],
    pub instance: Column<Instance>,
}

/// The MINT circuit.
#[derive(Debug, Default)]
pub struct MintCircuit {
    /// `fp(bond_id)` — also exposed as a public input (row 1).
    pub bond_id:    Value<Fp>,
    /// `Fp::from(quantity)`.
    pub quantity:   Value<Fp>,
    /// `fp(owner_pk)`.
    pub owner_pk:   Value<Fp>,
    /// `fp(randomness)`.
    pub randomness: Value<Fp>,
}

impl MintCircuit {
    /// Circuit with all witnesses set to unknown (used for key generation).
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Circuit<Fp> for MintCircuit {
    type Config = MintConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> MintConfig {
        let state        = [meta.advice_column(), meta.advice_column(), meta.advice_column()];
        let partial_sbox = meta.advice_column();
        let rc_a         = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        let rc_b         = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        meta.enable_constant(rc_b[0]);

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        let instance = meta.instance_column();
        meta.enable_equality(instance);

        MintConfig { poseidon, state, instance }
    }

    fn synthesize(&self, config: MintConfig, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        // Load bond_id and quantity as the first pair.
        let (bond_id_cell, qty_cell) = layouter.assign_region(
            || "load witness pair 1",
            |mut region| {
                let a = region.assign_advice(|| "bond_id",  config.state[0], 0, || self.bond_id)?;
                let b = region.assign_advice(|| "quantity", config.state[1], 0, || self.quantity)?;
                Ok((a, b))
            },
        )?;
        let bond_id_cell_ref = bond_id_cell.cell();

        // h1 = Poseidon2(bond_id, quantity)
        let h1 = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "h1 init"),
            )?;
            hasher.hash(layouter.namespace(|| "h1 hash"), [bond_id_cell, qty_cell])?
        };

        // Load owner_pk and randomness as the second pair.
        let (owner_pk_cell, randomness_cell) = layouter.assign_region(
            || "load witness pair 2",
            |mut region| {
                let c = region.assign_advice(|| "owner_pk",   config.state[0], 0, || self.owner_pk)?;
                let d = region.assign_advice(|| "randomness", config.state[1], 0, || self.randomness)?;
                Ok((c, d))
            },
        )?;

        // h2 = Poseidon2(owner_pk, randomness)
        let h2 = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "h2 init"),
            )?;
            hasher.hash(layouter.namespace(|| "h2 hash"), [owner_pk_cell, randomness_cell])?
        };

        // commitment = Poseidon2(h1, h2)
        let commitment = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "cm init"),
            )?;
            hasher.hash(layouter.namespace(|| "cm hash"), [h1, h2])?
        };

        // Constrain commitment == instance[0] and bond_id == instance[1].
        layouter.constrain_instance(commitment.cell(), config.instance, 0)?;
        layouter.constrain_instance(bond_id_cell_ref, config.instance, 1)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{fp_from_bytes, note_commitment};
    use halo2_proofs::dev::MockProver;

    const BOND_ID:    [u8; 32] = [0x01u8; 32];
    const QTY:        u64      = 10;
    const OWNER_PK:   [u8; 32] = [0xBBu8; 32];
    const RANDOMNESS: [u8; 32] = [0x42u8; 32];

    fn test_circuit() -> (MintCircuit, Vec<Vec<Fp>>) {
        let commitment = note_commitment(&BOND_ID, QTY, &OWNER_PK, &RANDOMNESS);
        let bond_id_fp = fp_from_bytes(&BOND_ID);

        let circuit = MintCircuit {
            bond_id:    Value::known(bond_id_fp),
            quantity:   Value::known(Fp::from(QTY)),
            owner_pk:   Value::known(fp_from_bytes(&OWNER_PK)),
            randomness: Value::known(fp_from_bytes(&RANDOMNESS)),
        };

        // Instance column: [commitment, bond_id]
        let instances = vec![commitment, bond_id_fp];
        (circuit, vec![instances])
    }

    #[test]
    fn mint_circuit_satisfies_constraints() {
        let (circuit, instances) = test_circuit();
        let k = 9;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert_eq!(prover.verify(), Ok(()), "MINT MockProver failed");
    }

    #[test]
    fn mint_circuit_fails_with_wrong_commitment() {
        let (circuit, mut instances) = test_circuit();
        instances[0][0] = Fp::from(0xdeadbeefu64);
        let k = 9;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert!(prover.verify().is_err(), "should fail with wrong commitment");
    }

    #[test]
    fn mint_circuit_fails_with_wrong_bond_id() {
        let (circuit, mut instances) = test_circuit();
        instances[0][1] = Fp::from(0xdeadbeefu64);
        let k = 9;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert!(prover.verify().is_err(), "should fail with wrong bond_id");
    }
}
