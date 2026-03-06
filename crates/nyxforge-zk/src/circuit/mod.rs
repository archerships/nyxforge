//! Real Halo2 circuits for NyxForge's ZK operations.
//!
//! Each circuit is a PLONK circuit using the `Pow5Chip` from `halo2_gadgets`
//! for Poseidon hashing.  The commitment and nullifier computations in the
//! circuit match the native implementations in [`crate::primitives`].

pub mod burn;
pub mod mint;
pub mod transfer;
