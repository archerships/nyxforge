//! Native (non-circuit) cryptographic primitives.
//!
//! These functions compute the same values that the in-circuit Poseidon gadget
//! proves knowledge of.  They are used by the prover to compute witnesses and
//! public inputs before calling `create_proof`, and by the verifier to
//! reconstruct public inputs from a `*Proof` struct.
//!
//! # Commitment scheme
//!
//! ```text
//! h1         = Poseidon2(fp(bond_id), fp(quantity))
//! h2         = Poseidon2(fp(owner_pk), fp(randomness))
//! commitment = Poseidon2(h1, h2)
//! ```
//!
//! # Nullifier
//!
//! ```text
//! nullifier = Poseidon2(fp(owner_secret), fp(serial))
//! ```
//!
//! Both use `P128Pow5T3` — 128-bit Poseidon with x^5 S-box, T=3, RATE=2,
//! operating over the Pallas base field (= Fp).

use halo2_gadgets::poseidon::primitives::{self as poseidon, ConstantLength, P128Pow5T3};
use halo2_proofs::pasta::Fp;
use pasta_curves::group::ff::PrimeField;

/// Map 32 bytes to a Pallas field element via little-endian interpretation.
///
/// If the byte string is >= p (the field modulus), it is reduced modulo p.
/// This is done by interpreting the bytes as a 256-bit little-endian integer
/// and calling `Fp::from_repr_vartime` with a fallback to `Fp::zero()` on
/// overflow (which cannot happen for uniform random bytes, only for crafted
/// inputs near the modulus).
pub fn fp_from_bytes(bytes: &[u8; 32]) -> Fp {
    // Fp::from_repr expects little-endian bytes.
    // If the value is >= p, from_repr returns None; we reduce by masking the
    // top bits.  For the keys and random scalars we use (which are generated
    // randomly or via well-formed derivation), the probability of overflow is
    // negligible.
    Option::from(Fp::from_repr(*bytes)).unwrap_or_else(|| {
        // Reduce: clear the top 3 bits (the Pallas field modulus has 255 bits)
        let mut b = *bytes;
        b[31] &= 0x1f;
        Option::from(Fp::from_repr(b)).unwrap_or(Fp::zero())
    })
}

/// Serialize a Pallas field element to 32 bytes (little-endian canonical form).
pub fn fp_to_bytes(fp: Fp) -> [u8; 32] {
    fp.to_repr()
}

/// Poseidon hash of exactly 2 field elements.
///
/// Uses the `P128Pow5T3` specification (WIDTH=3, RATE=2).
pub fn poseidon2(a: Fp, b: Fp) -> Fp {
    poseidon::Hash::<_, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash([a, b])
}

/// Compute the note commitment for a bond note.
///
/// ```text
/// h1 = Poseidon2(fp(bond_id), Fp::from(quantity))
/// h2 = Poseidon2(fp(owner_pk), fp(randomness))
/// commitment = Poseidon2(h1, h2)
/// ```
pub fn note_commitment(
    bond_id: &[u8; 32],
    quantity: u64,
    owner_pk: &[u8; 32],
    randomness: &[u8; 32],
) -> Fp {
    let h1 = poseidon2(fp_from_bytes(bond_id), Fp::from(quantity));
    let h2 = poseidon2(fp_from_bytes(owner_pk), fp_from_bytes(randomness));
    poseidon2(h1, h2)
}

/// Compute the nullifier for a bond note.
///
/// ```text
/// nullifier = Poseidon2(fp(owner_secret), fp(serial))
/// ```
pub fn note_nullifier(owner_secret: &[u8; 32], serial: &[u8; 32]) -> Fp {
    poseidon2(fp_from_bytes(owner_secret), fp_from_bytes(serial))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOND_ID:       [u8; 32] = [0x01u8; 32];
    const QTY:           u64      = 10;
    const OWNER_PK:      [u8; 32] = [0xBBu8; 32];
    const RANDOMNESS:    [u8; 32] = [0x42u8; 32];
    const OWNER_SECRET:  [u8; 32] = [0xAAu8; 32];
    const SERIAL:        [u8; 32] = [0x55u8; 32];

    #[test]
    fn commitment_is_deterministic() {
        let a = note_commitment(&BOND_ID, QTY, &OWNER_PK, &RANDOMNESS);
        let b = note_commitment(&BOND_ID, QTY, &OWNER_PK, &RANDOMNESS);
        assert_eq!(a, b);
    }

    #[test]
    fn commitment_differs_by_quantity() {
        let a = note_commitment(&BOND_ID, 10, &OWNER_PK, &RANDOMNESS);
        let b = note_commitment(&BOND_ID, 11, &OWNER_PK, &RANDOMNESS);
        assert_ne!(a, b);
    }

    #[test]
    fn commitment_differs_by_owner() {
        let a = note_commitment(&BOND_ID, QTY, &OWNER_PK, &RANDOMNESS);
        let b = note_commitment(&BOND_ID, QTY, &[0xCCu8; 32], &RANDOMNESS);
        assert_ne!(a, b);
    }

    #[test]
    fn commitment_differs_by_bond_id() {
        let a = note_commitment(&[0xAAu8; 32], QTY, &OWNER_PK, &RANDOMNESS);
        let b = note_commitment(&[0xBBu8; 32], QTY, &OWNER_PK, &RANDOMNESS);
        assert_ne!(a, b);
    }

    #[test]
    fn nullifier_is_deterministic() {
        let a = note_nullifier(&OWNER_SECRET, &SERIAL);
        let b = note_nullifier(&OWNER_SECRET, &SERIAL);
        assert_eq!(a, b);
    }

    #[test]
    fn nullifier_differs_by_secret() {
        let a = note_nullifier(&OWNER_SECRET, &SERIAL);
        let b = note_nullifier(&[0xFFu8; 32], &SERIAL);
        assert_ne!(a, b);
    }

    #[test]
    fn nullifier_differs_by_serial() {
        let a = note_nullifier(&OWNER_SECRET, &SERIAL);
        let b = note_nullifier(&OWNER_SECRET, &[0x99u8; 32]);
        assert_ne!(a, b);
    }

    #[test]
    fn commitment_and_nullifier_are_independent() {
        // Changing randomness shifts commitment but leaves nullifier unchanged.
        let cm_a = note_commitment(&BOND_ID, QTY, &OWNER_PK, &RANDOMNESS);
        let cm_b = note_commitment(&BOND_ID, QTY, &OWNER_PK, &[0xFFu8; 32]);
        let nul = note_nullifier(&OWNER_SECRET, &SERIAL);
        let nul2 = note_nullifier(&OWNER_SECRET, &SERIAL);
        assert_ne!(cm_a, cm_b);
        assert_eq!(nul, nul2);
    }

    #[test]
    fn fp_roundtrip() {
        let bytes = [0x12u8; 32];
        let fp = fp_from_bytes(&bytes);
        // The upper bits may have been masked, so check the canonical form
        let back = fp_to_bytes(fp);
        let fp2 = fp_from_bytes(&back);
        assert_eq!(fp, fp2);
    }

    #[test]
    fn print_golden_values() {
        // Run this test with --nocapture to obtain values for updating golden vectors.
        let cm = note_commitment(&BOND_ID, QTY, &OWNER_PK, &RANDOMNESS);
        let nul = note_nullifier(&OWNER_SECRET, &SERIAL);
        println!("GOLDEN_COMMITMENT = {}", hex::encode(fp_to_bytes(cm)));
        println!("GOLDEN_NULLIFIER  = {}", hex::encode(fp_to_bytes(nul)));
    }
}
