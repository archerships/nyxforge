use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Fixed-point amount denominated in the network's base token (e.g. DRK).
/// Stored as integer micro-units (1 DRK = 1_000_000 μDRK).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Zeroize)]
pub struct Amount(pub u64);

impl Amount {
    pub const ZERO: Self = Self(0);

    pub fn from_whole(units: u64) -> Self {
        Self(units * 1_000_000)
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

/// 32-byte opaque commitment / hash identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize)]
pub struct Digest([u8; 32]);

impl Digest {
    pub fn from_bytes(b: [u8; 32]) -> Self { Self(b) }
    pub fn as_bytes(&self) -> &[u8; 32] { &self.0 }
    pub fn zero() -> Self { Self([0u8; 32]) }
}

impl From<blake3::Hash> for Digest {
    fn from(h: blake3::Hash) -> Self { Self(*h.as_bytes()) }
}

/// Nullifier: revealed when spending a bond note, prevents double-spend.
pub type Nullifier = Digest;

/// Compressed Ristretto255 public key (32 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize)]
pub struct PublicKey(pub [u8; 32]);

/// Secret key — zeroised on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecretKey(pub [u8; 32]);

impl SecretKey {
    pub fn generate(rng: &mut impl rand::RngCore) -> Self {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }
}

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretKey([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Amount::from_whole ---

    #[test]
    fn from_whole_scales_by_micro_units() {
        assert_eq!(Amount::from_whole(1).0, 1_000_000);
        assert_eq!(Amount::from_whole(100).0, 100_000_000);
    }

    #[test]
    fn from_whole_zero_equals_amount_zero() {
        assert_eq!(Amount::from_whole(0), Amount::ZERO);
    }

    // --- checked_add ---

    #[test]
    fn checked_add_returns_correct_sum() {
        let a = Amount::from_whole(3);
        let b = Amount::from_whole(7);
        assert_eq!(a.checked_add(b), Some(Amount::from_whole(10)));
    }

    #[test]
    fn checked_add_overflow_returns_none() {
        let big = Amount(u64::MAX);
        assert_eq!(big.checked_add(Amount(1)), None);
    }

    #[test]
    fn checked_add_with_zero_is_identity() {
        let a = Amount::from_whole(42);
        assert_eq!(a.checked_add(Amount::ZERO), Some(a));
    }

    // --- checked_sub ---

    #[test]
    fn checked_sub_returns_correct_difference() {
        let a = Amount::from_whole(10);
        let b = Amount::from_whole(3);
        assert_eq!(a.checked_sub(b), Some(Amount::from_whole(7)));
    }

    #[test]
    fn checked_sub_to_zero_returns_zero() {
        let a = Amount::from_whole(5);
        assert_eq!(a.checked_sub(a), Some(Amount::ZERO));
    }

    #[test]
    fn checked_sub_underflow_returns_none() {
        let a = Amount::from_whole(1);
        let b = Amount::from_whole(2);
        assert_eq!(a.checked_sub(b), None);
    }

    // --- ordering ---

    #[test]
    fn amount_ordering_is_consistent() {
        assert!(Amount::from_whole(1) < Amount::from_whole(2));
        assert!(Amount::from_whole(2) > Amount::from_whole(1));
        assert_eq!(Amount::from_whole(5), Amount::from_whole(5));
    }

    // --- SecretKey::generate ---

    #[test]
    fn secret_key_generate_is_not_all_zero() {
        let mut rng = rand::thread_rng();
        let sk = SecretKey::generate(&mut rng);
        assert_ne!(sk.0, [0u8; 32]);
    }

    #[test]
    fn secret_key_debug_redacts_bytes() {
        let mut rng = rand::thread_rng();
        let sk = SecretKey::generate(&mut rng);
        assert_eq!(format!("{sk:?}"), "SecretKey([REDACTED])");
    }

    // --- Digest ---

    #[test]
    fn digest_zero_is_all_zeros() {
        assert_eq!(Digest::zero().as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn digest_from_bytes_roundtrips() {
        let bytes = [0xABu8; 32];
        assert_eq!(Digest::from_bytes(bytes).as_bytes(), &bytes);
    }
}
