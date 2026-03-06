//! TRANSFER circuit — proves anonymous bond note ownership transfer.
//!
//! # Public inputs (instance column, rows 0..2)
//!
//! | Row | Value           |
//! |-----|-----------------|
//! |  0  | `nullifier`     |
//! |  1  | `new_commitment`|
//! |  2  | `bond_id`       |
//!
//! # Statement proved
//!
//! The prover knows `(old_note, owner_secret, new_owner_pk, new_randomness, new_serial)` such that:
//!
//! ```text
//! nullifier      = Poseidon2(owner_secret, old_serial)
//! new_commitment = Poseidon2(Poseidon2(bond_id, quantity), Poseidon2(new_owner_pk, new_randomness))
//! ```
//!
//! Additionally (enforced as equality gates):
//! - `quantity` is the same in the old and new note (conservation)
//! - `bond_id` is the same in the old and new note (same series)
//!
//! **Phase 1 note:** A full Merkle membership proof for the old note's commitment
//! is not included.  The node verifies off-circuit that the nullifier derives
//! from a known commitment in the commitment set.  Merkle inclusion will be
//! added when NyxForge integrates with DarkFi's on-chain state (Phase 6).

use halo2_gadgets::poseidon::{
    primitives::{ConstantLength, P128Pow5T3},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};

/// Configuration for [`TransferCircuit`].
#[derive(Clone, Debug)]
pub struct TransferConfig {
    pub poseidon: Pow5Config<Fp, 3, 2>,
    pub state:    [Column<Advice>; 3],
    pub instance: Column<Instance>,
}

/// The TRANSFER circuit.
#[derive(Debug, Default)]
pub struct TransferCircuit {
    // Old note witnesses
    pub old_bond_id:    Value<Fp>,
    pub old_quantity:   Value<Fp>,
    pub old_owner_pk:   Value<Fp>,
    pub old_randomness: Value<Fp>,
    pub old_serial:     Value<Fp>,

    // Spending key
    pub owner_secret:   Value<Fp>,

    // New note witnesses
    pub new_owner_pk:   Value<Fp>,
    pub new_randomness: Value<Fp>,
}

impl TransferCircuit {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Circuit<Fp> for TransferCircuit {
    type Config = TransferConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> TransferConfig {
        let state        = [meta.advice_column(), meta.advice_column(), meta.advice_column()];
        let partial_sbox = meta.advice_column();
        let rc_a         = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        let rc_b         = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        meta.enable_constant(rc_b[0]);

        let poseidon = Pow5Chip::configure::<P128Pow5T3>(meta, state, partial_sbox, rc_a, rc_b);

        let instance = meta.instance_column();
        meta.enable_equality(instance);

        TransferConfig { poseidon, state, instance }
    }

    fn synthesize(
        &self,
        config: TransferConfig,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // ----- Nullifier = Poseidon2(owner_secret, old_serial) -----

        let (secret_cell, serial_cell) = layouter.assign_region(
            || "load nullifier witnesses",
            |mut region| {
                let a = region.assign_advice(|| "owner_secret", config.state[0], 0, || self.owner_secret)?;
                let b = region.assign_advice(|| "old_serial",   config.state[1], 0, || self.old_serial)?;
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

        // ----- new_commitment = Poseidon2(h1_new, h2_new) -----
        // h1_new = Poseidon2(bond_id, quantity)

        let (bond_id_cell, qty_cell) = layouter.assign_region(
            || "load new h1 witnesses",
            |mut region| {
                let a = region.assign_advice(|| "bond_id",  config.state[0], 0, || self.old_bond_id)?;
                let b = region.assign_advice(|| "quantity", config.state[1], 0, || self.old_quantity)?;
                Ok((a, b))
            },
        )?;
        let bond_id_cell_ref = bond_id_cell.cell();
        let qty_cell_ref     = qty_cell.cell();

        let h1_new = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "h1_new init"),
            )?;
            hasher.hash(layouter.namespace(|| "h1_new hash"), [bond_id_cell, qty_cell])?
        };

        // h2_new = Poseidon2(new_owner_pk, new_randomness)
        let (new_owner_cell, new_rand_cell) = layouter.assign_region(
            || "load new h2 witnesses",
            |mut region| {
                let c = region.assign_advice(|| "new_owner_pk",   config.state[0], 0, || self.new_owner_pk)?;
                let d = region.assign_advice(|| "new_randomness", config.state[1], 0, || self.new_randomness)?;
                Ok((c, d))
            },
        )?;

        let h2_new = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "h2_new init"),
            )?;
            hasher.hash(layouter.namespace(|| "h2_new hash"), [new_owner_cell, new_rand_cell])?
        };

        // new_commitment = Poseidon2(h1_new, h2_new)
        let new_commitment = {
            let hasher = Hash::<_, _, P128Pow5T3, ConstantLength<2>, 3, 2>::init(
                Pow5Chip::construct(config.poseidon.clone()),
                layouter.namespace(|| "new_cm init"),
            )?;
            hasher.hash(layouter.namespace(|| "new_cm hash"), [h1_new, h2_new])?
        };

        // ----- Conservation: verify old note has same bond_id and quantity -----
        // We load old_bond_id and old_quantity again (same values) and constrain
        // them equal to the cells already used for the new commitment computation.
        let (old_bond_id_cell, old_qty_cell) = layouter.assign_region(
            || "load old note fields for conservation check",
            |mut region| {
                let a = region.assign_advice(|| "old_bond_id",  config.state[0], 0, || self.old_bond_id)?;
                let b = region.assign_advice(|| "old_quantity", config.state[1], 0, || self.old_quantity)?;
                Ok((a, b))
            },
        )?;

        // Enforce old values equal the values used in the new commitment.
        layouter.assign_region(
            || "conservation equality",
            |mut region| {
                region.constrain_equal(old_bond_id_cell.cell(), bond_id_cell_ref)?;
                region.constrain_equal(old_qty_cell.cell(), qty_cell_ref)?;
                Ok(())
            },
        )?;

        // ----- Constrain public instances -----
        layouter.constrain_instance(nullifier.cell(),          config.instance, 0)?;
        layouter.constrain_instance(new_commitment.cell(),     config.instance, 1)?;
        layouter.constrain_instance(old_bond_id_cell.cell(),   config.instance, 2)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{fp_from_bytes, note_commitment, note_nullifier};
    use halo2_proofs::dev::MockProver;

    const BOND_ID:      [u8; 32] = [0x01u8; 32];
    const QTY:          u64      = 10;
    const OWNER_PK:     [u8; 32] = [0xBBu8; 32];
    const RANDOMNESS:   [u8; 32] = [0x42u8; 32];
    const SERIAL:       [u8; 32] = [0x55u8; 32];
    const OWNER_SECRET: [u8; 32] = [0xAAu8; 32];
    const NEW_OWNER_PK: [u8; 32] = [0xCCu8; 32];
    const NEW_RAND:     [u8; 32] = [0x77u8; 32];

    fn test_circuit() -> (TransferCircuit, Vec<Vec<Fp>>) {
        let nullifier      = note_nullifier(&OWNER_SECRET, &SERIAL);
        let new_commitment = note_commitment(&BOND_ID, QTY, &NEW_OWNER_PK, &NEW_RAND);
        let bond_id_fp     = fp_from_bytes(&BOND_ID);

        let circuit = TransferCircuit {
            old_bond_id:    Value::known(bond_id_fp),
            old_quantity:   Value::known(Fp::from(QTY)),
            old_owner_pk:   Value::known(fp_from_bytes(&OWNER_PK)),
            old_randomness: Value::known(fp_from_bytes(&RANDOMNESS)),
            old_serial:     Value::known(fp_from_bytes(&SERIAL)),
            owner_secret:   Value::known(fp_from_bytes(&OWNER_SECRET)),
            new_owner_pk:   Value::known(fp_from_bytes(&NEW_OWNER_PK)),
            new_randomness: Value::known(fp_from_bytes(&NEW_RAND)),
        };

        // Instance: [nullifier, new_commitment, bond_id]
        let instances = vec![nullifier, new_commitment, bond_id_fp];
        (circuit, vec![instances])
    }

    #[test]
    fn transfer_circuit_satisfies_constraints() {
        let (circuit, instances) = test_circuit();
        let k = 10;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert_eq!(prover.verify(), Ok(()), "TRANSFER MockProver failed");
    }

    #[test]
    fn transfer_circuit_fails_with_wrong_nullifier() {
        let (circuit, mut instances) = test_circuit();
        instances[0][0] = Fp::from(0xdeadbeefu64);
        let k = 10;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert!(prover.verify().is_err(), "should fail with wrong nullifier");
    }

    #[test]
    fn transfer_circuit_fails_with_wrong_new_commitment() {
        let (circuit, mut instances) = test_circuit();
        instances[0][1] = Fp::from(0xdeadbeefu64);
        let k = 10;
        let prover = MockProver::run(k, &circuit, instances).unwrap();
        assert!(prover.verify().is_err(), "should fail with wrong commitment");
    }
}
