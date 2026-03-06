//! Bond note fixtures and lazily-computed test vectors.
//!
//! All notes use fixed inputs so their commitments and nullifiers are stable
//! across runs.  The lazy statics hold the *computed* values of those
//! deterministic operations so other crates can assert against them without
//! re-computing inline.
//!
//! # Golden-value assertions
//!
//! The `GOLDEN_*` constants are the hex-encoded output of the commitment and
//! nullifier functions for the standard `default_note()`.  They were recorded
//! on first run and hardcoded here.  If the hash function or domain separator
//! ever changes, these tests will fail — which is the intended behaviour.
//!
//! To recompute: run `cargo test -p nyxforge-test-fixtures -- --nocapture`
//! and look for the "GOLDEN" lines.

use once_cell::sync::Lazy;

use nyxforge_core::bond::BondId;
use nyxforge_core::types::{Amount, Digest, PublicKey};
use nyxforge_zk::note::BondNote;

// ---------------------------------------------------------------------------
// Fixed scalars — all obviously synthetic.
// ---------------------------------------------------------------------------

/// Secret key used when deriving nullifiers in tests.
pub const OWNER_SECRET: [u8; 32] = [0xAAu8; 32];

/// Public key paired with [`OWNER_SECRET`] in tests.
///
/// Note: this is a raw byte array used as a placeholder public key.
/// It is NOT a valid Ed25519 public key derived from `OWNER_SECRET`.
pub const OWNER_KEY: PublicKey = PublicKey([0xBBu8; 32]);

/// A second owner key — used as the recipient in transfer tests.
pub const RECIPIENT_KEY: PublicKey = PublicKey([0xCCu8; 32]);

// ---------------------------------------------------------------------------
// Note constructors
// ---------------------------------------------------------------------------

/// The canonical test note — fixed bond_id (zero), quantity 5, owner [`OWNER_KEY`].
///
/// Use this when the note content is irrelevant to the test.
pub fn default_note() -> BondNote {
    BondNote {
        bond_id: Digest::zero(),
        quantity: 5,
        redemption_value: Amount::from_whole(100),
        owner: OWNER_KEY,
        randomness: [0x07u8; 32],
        serial: [0x03u8; 32],
    }
}

/// A note for a specific bond ID and quantity.
///
/// All other fields are fixed.  Use when the bond ID or quantity matters.
pub fn note_for_bond(bond_id: BondId, quantity: u64) -> BondNote {
    BondNote {
        bond_id,
        quantity,
        redemption_value: Amount::from_whole(10),
        owner: OWNER_KEY,
        randomness: [0x07u8; 32],
        serial: [0x03u8; 32],
    }
}

/// A note for a specific bond owned by [`RECIPIENT_KEY`].
///
/// Useful as the "new" note in transfer tests, distinct from notes
/// owned by [`OWNER_KEY`].
pub fn recipient_note(bond_id: BondId, quantity: u64) -> BondNote {
    BondNote {
        bond_id,
        quantity,
        redemption_value: Amount::from_whole(10),
        owner: RECIPIENT_KEY,
        randomness: [0x08u8; 32],
        serial: [0x04u8; 32],
    }
}

// ---------------------------------------------------------------------------
// Lazily-computed test vectors
//
// The `Lazy` wrapper ensures the computation happens exactly once per process,
// not per test call.  Tests across crates that import these statics all see
// the same computed values.
// ---------------------------------------------------------------------------

/// Commitment of `default_note()`.
///
/// Computed once; compare against this in tests that need the expected
/// commitment of the canonical note without re-invoking the function.
pub static DEFAULT_NOTE_COMMITMENT: Lazy<Digest> =
    Lazy::new(|| default_note().commitment());

/// Nullifier of `default_note()` derived with [`OWNER_SECRET`].
pub static DEFAULT_NOTE_NULLIFIER: Lazy<Digest> =
    Lazy::new(|| default_note().nullifier(&OWNER_SECRET));

// ---------------------------------------------------------------------------
// Golden test vectors
//
// These hex strings were recorded on first run (see module doc).
// They exist to catch unintentional changes to the commitment or nullifier
// domain separators / hash functions.
//
// Mark as `todo!()` until first run produces the values; then fill them in.
// ---------------------------------------------------------------------------

/// Hex encoding of [`DEFAULT_NOTE_COMMITMENT`].
///
/// Recorded on 2026-03-04 using Poseidon (P128Pow5T3) over the Pallas field.
/// Updated from blake3 as part of Phase 1 real Halo2 ZK circuit implementation.
/// If this test fails, the commitment algorithm has changed unexpectedly.
pub const GOLDEN_DEFAULT_COMMITMENT_HEX: Option<&str> =
    Some("2c6e6a2aa4e2009088f45f9631889d04931091c6fb4c6425392d7f54acd8b624");

/// Hex encoding of [`DEFAULT_NOTE_NULLIFIER`].
///
/// Recorded on 2026-03-04 using Poseidon (P128Pow5T3) over the Pallas field.
/// Updated from blake3 as part of Phase 1 real Halo2 ZK circuit implementation.
pub const GOLDEN_DEFAULT_NULLIFIER_HEX: Option<&str> =
    Some("06386003b4da00bf77c0643f4c630eea73b2c2423dacaad16bf2ecc80c7ad40a");

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

pub fn digest_to_hex(d: &Digest) -> String {
    d.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commitment_is_deterministic() {
        assert_eq!(default_note().commitment(), default_note().commitment());
    }

    #[test]
    fn lazy_commitment_matches_fresh_call() {
        assert_eq!(*DEFAULT_NOTE_COMMITMENT, default_note().commitment());
    }

    #[test]
    fn lazy_nullifier_matches_fresh_call() {
        assert_eq!(*DEFAULT_NOTE_NULLIFIER, default_note().nullifier(&OWNER_SECRET));
    }

    #[test]
    fn different_owners_produce_different_commitments() {
        let mut other = default_note();
        other.owner = RECIPIENT_KEY;
        assert_ne!(default_note().commitment(), other.commitment());
    }

    #[test]
    fn different_quantities_produce_different_commitments() {
        let mut other = default_note();
        other.quantity = 99;
        assert_ne!(default_note().commitment(), other.commitment());
    }

    #[test]
    fn different_secrets_produce_different_nullifiers() {
        let n1 = default_note().nullifier(&OWNER_SECRET);
        let n2 = default_note().nullifier(&[0xFFu8; 32]);
        assert_ne!(n1, n2);
    }

    #[test]
    fn different_serials_produce_different_nullifiers() {
        let mut other = default_note();
        other.serial = [0x99u8; 32];
        assert_ne!(
            default_note().nullifier(&OWNER_SECRET),
            other.nullifier(&OWNER_SECRET),
        );
    }

    #[test]
    fn note_for_bond_uses_supplied_bond_id() {
        let id = Digest::from_bytes([0x55u8; 32]);
        let note = note_for_bond(id, 3);
        assert_eq!(note.bond_id, id);
        assert_eq!(note.quantity, 3);
    }

    /// Prints computed values so they can be hardcoded as golden vectors.
    #[test]
    fn print_golden_values() {
        println!("GOLDEN_DEFAULT_COMMITMENT_HEX = \"{}\"",
            digest_to_hex(&DEFAULT_NOTE_COMMITMENT));
        println!("GOLDEN_DEFAULT_NULLIFIER_HEX  = \"{}\"",
            digest_to_hex(&DEFAULT_NOTE_NULLIFIER));
    }

    /// Asserts golden values once they have been filled in.
    #[test]
    fn golden_commitment_unchanged() {
        if let Some(hex) = GOLDEN_DEFAULT_COMMITMENT_HEX {
            assert_eq!(digest_to_hex(&DEFAULT_NOTE_COMMITMENT), hex,
                "Commitment algorithm has changed — update GOLDEN_DEFAULT_COMMITMENT_HEX");
        }
    }

    #[test]
    fn golden_nullifier_unchanged() {
        if let Some(hex) = GOLDEN_DEFAULT_NULLIFIER_HEX {
            assert_eq!(digest_to_hex(&DEFAULT_NOTE_NULLIFIER), hex,
                "Nullifier algorithm has changed — update GOLDEN_DEFAULT_NULLIFIER_HEX");
        }
    }
}
