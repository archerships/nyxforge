//! Bond note: the anonymous representation of bond ownership.
//!
//! A note is analogous to a Zcash sapling note.  Its commitment is revealed
//! on-chain; its plaintext (and thus owner identity) is encrypted to the
//! recipient.

use nyxforge_core::types::{Amount, Digest, PublicKey};
use nyxforge_core::bond::BondId;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::primitives;

/// Plaintext note — kept secret by the holder.
#[derive(Debug, Clone, Zeroize, Serialize, Deserialize)]
#[zeroize(drop)]
pub struct BondNote {
    /// Which bond series this note represents.
    pub bond_id: BondId,

    /// Number of bond units held in this note.
    pub quantity: u64,

    /// Value in base token if redeemed (cached for convenience).
    pub redemption_value: Amount,

    /// Owner's public key (Ristretto255 point, compressed).
    pub owner: PublicKey,

    /// Random blinding scalar — unique per note, never reused.
    pub randomness: [u8; 32],

    /// Serial number — hashed with owner secret to produce the nullifier.
    pub serial: [u8; 32],
}

impl BondNote {
    /// Poseidon commitment: `Poseidon2(Poseidon2(bond_id, qty), Poseidon2(owner, r))`.
    ///
    /// Uses the same hash the in-circuit `MintCircuit` and `TransferCircuit`
    /// prove knowledge of.  Switching from the previous blake3 placeholder
    /// means all prior (test) commitments are invalidated.
    pub fn commitment(&self) -> Digest {
        let fp = primitives::note_commitment(
            self.bond_id.as_bytes(),
            self.quantity,
            &self.owner.0,
            &self.randomness,
        );
        Digest::from_bytes(primitives::fp_to_bytes(fp))
    }

    /// Nullifier: `Poseidon2(owner_secret, serial)`.
    ///
    /// Revealed when the note is spent to prevent double-spend.
    /// The actual derivation requires the holder's secret key.
    pub fn nullifier(&self, owner_secret: &[u8; 32]) -> Digest {
        let fp = primitives::note_nullifier(owner_secret, &self.serial);
        Digest::from_bytes(primitives::fp_to_bytes(fp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER_SECRET:   [u8; 32] = [0xAAu8; 32];
    const OWNER_KEY:      PublicKey = PublicKey([0xBBu8; 32]);
    const RECIPIENT_KEY:  PublicKey = PublicKey([0xCCu8; 32]);

    fn default_note() -> BondNote {
        BondNote {
            bond_id:          Digest::from_bytes([0x01u8; 32]),
            quantity:         10,
            redemption_value: nyxforge_core::types::Amount(1_000_000),
            owner:            OWNER_KEY,
            randomness:       [0x42u8; 32],
            serial:           [0x55u8; 32],
        }
    }

    fn note_for_bond(bond_id: Digest, quantity: u64) -> BondNote {
        BondNote {
            bond_id,
            quantity,
            redemption_value: nyxforge_core::types::Amount(1_000_000),
            owner:            OWNER_KEY,
            randomness:       [0x42u8; 32],
            serial:           [0x55u8; 32],
        }
    }

    // --- Commitment ---

    #[test]
    fn commitment_is_deterministic() {
        assert_eq!(default_note().commitment(), default_note().commitment());
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
    fn different_bond_ids_produce_different_commitments() {
        let id_a = Digest::from_bytes([0xAAu8; 32]);
        let id_b = Digest::from_bytes([0xBBu8; 32]);
        assert_ne!(note_for_bond(id_a, 5).commitment(), note_for_bond(id_b, 5).commitment());
    }

    // --- Nullifier ---

    #[test]
    fn nullifier_is_deterministic() {
        assert_eq!(
            default_note().nullifier(&OWNER_SECRET),
            default_note().nullifier(&OWNER_SECRET),
        );
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
    fn owner_field_does_not_affect_nullifier() {
        // Nullifier is PRF(secret, serial) — owner_pk is not an input.
        let mut other = default_note();
        other.owner = RECIPIENT_KEY;
        assert_eq!(
            default_note().nullifier(&OWNER_SECRET),
            other.nullifier(&OWNER_SECRET),
        );
    }

    #[test]
    fn commitment_and_nullifier_are_independent() {
        // Changing randomness shifts commitment but leaves nullifier unchanged.
        let mut other = default_note();
        other.randomness = [0xFFu8; 32];
        assert_ne!(default_note().commitment(), other.commitment());
        assert_eq!(
            default_note().nullifier(&OWNER_SECRET),
            other.nullifier(&OWNER_SECRET),
        );
    }
}
