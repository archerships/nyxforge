//! Bond note: the anonymous representation of bond ownership.
//!
//! A note is analogous to a Zcash sapling note.  Its commitment is revealed
//! on-chain; its plaintext (and thus owner identity) is encrypted to the
//! recipient.

use blake3::Hasher;
use nyxforge_core::types::{Amount, Digest, PublicKey};
use nyxforge_core::bond::BondId;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

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
    /// Pedersen-style commitment: `Com(bond_id, qty, owner, r)`.
    ///
    /// In production this uses DarkFi's zkVM gadget.  Here we use
    /// blake3 as a placeholder until the circuit is wired up.
    pub fn commitment(&self) -> Digest {
        let mut h = Hasher::new();
        h.update(b"nyxforge::note_commit");
        h.update(self.bond_id.as_bytes());
        h.update(&self.quantity.to_le_bytes());
        h.update(&self.owner.0);
        h.update(&self.randomness);
        Digest::from(h.finalize())
    }

    /// Nullifier: `PRF(owner_secret, serial)`.
    ///
    /// Revealed when the note is spent to prevent double-spend.
    /// The actual derivation needs the holder's secret key.
    pub fn nullifier(&self, owner_secret: &[u8; 32]) -> Digest {
        let mut h = Hasher::new();
        h.update(b"nyxforge::nullifier");
        h.update(owner_secret);
        h.update(&self.serial);
        Digest::from(h.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::Amount;

    fn dummy_note() -> BondNote {
        BondNote {
            bond_id:          Digest::zero(),
            quantity:         5,
            redemption_value: Amount::from_whole(100),
            owner:            PublicKey([1u8; 32]),
            randomness:       [7u8; 32],
            serial:           [3u8; 32],
        }
    }

    #[test]
    fn commitment_is_deterministic() {
        let note = dummy_note();
        assert_eq!(note.commitment(), note.commitment());
    }

    #[test]
    fn nullifier_depends_on_secret() {
        let note = dummy_note();
        let n1 = note.nullifier(&[0u8; 32]);
        let n2 = note.nullifier(&[1u8; 32]);
        assert_ne!(n1, n2);
    }
}
