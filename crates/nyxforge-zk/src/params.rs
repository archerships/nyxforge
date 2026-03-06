//! Lazy-initialised circuit proving and verifying keys.
//!
//! Key generation is **deterministic** — `Params::new(k)` uses a
//! hash-to-curve procedure seeded by the circuit size parameter `k`, so
//! the same `k` always produces the same generators.  No RNG is required.
//!
//! The first access to each `Lazy` static incurs a one-time cost:
//!
//! | Circuit  | k  | Rows | Typical keygen time |
//! |----------|----|------|---------------------|
//! | MINT     | 9  | 512  | ~1-3 s              |
//! | TRANSFER | 10 | 1024 | ~2-5 s              |
//! | BURN     | 10 | 1024 | ~2-5 s              |
//!
//! After the first call the result is cached for the lifetime of the process.

use once_cell::sync::Lazy;
use pasta_curves::EqAffine;

use halo2_proofs::{
    plonk::{self, ProvingKey, VerifyingKey},
    poly::commitment::Params,
};

use crate::circuit::{
    burn::BurnCircuit,
    mint::MintCircuit,
    transfer::TransferCircuit,
};

const MINT_K:     u32 = 9;
const TRANSFER_K: u32 = 10;
const BURN_K:     u32 = 10;

/// Pre-computed proving and verifying keys for a single circuit.
pub struct CircuitKeys {
    pub params: Params<EqAffine>,
    pub pk:     ProvingKey<EqAffine>,
    pub vk:     VerifyingKey<EqAffine>,
}

fn build_mint_keys() -> CircuitKeys {
    let params  = Params::new(MINT_K);
    let circuit = MintCircuit::empty();
    let vk      = plonk::keygen_vk(&params, &circuit).expect("MINT keygen_vk failed");
    let pk      = plonk::keygen_pk(&params, vk.clone(), &circuit).expect("MINT keygen_pk failed");
    CircuitKeys { params, pk, vk }
}

fn build_transfer_keys() -> CircuitKeys {
    let params  = Params::new(TRANSFER_K);
    let circuit = TransferCircuit::empty();
    let vk      = plonk::keygen_vk(&params, &circuit).expect("TRANSFER keygen_vk failed");
    let pk      = plonk::keygen_pk(&params, vk.clone(), &circuit).expect("TRANSFER keygen_pk failed");
    CircuitKeys { params, pk, vk }
}

fn build_burn_keys() -> CircuitKeys {
    let params  = Params::new(BURN_K);
    let circuit = BurnCircuit::empty();
    let vk      = plonk::keygen_vk(&params, &circuit).expect("BURN keygen_vk failed");
    let pk      = plonk::keygen_pk(&params, vk.clone(), &circuit).expect("BURN keygen_pk failed");
    CircuitKeys { params, pk, vk }
}

pub static MINT_KEYS:     Lazy<CircuitKeys> = Lazy::new(build_mint_keys);
pub static TRANSFER_KEYS: Lazy<CircuitKeys> = Lazy::new(build_transfer_keys);
pub static BURN_KEYS:     Lazy<CircuitKeys> = Lazy::new(build_burn_keys);
