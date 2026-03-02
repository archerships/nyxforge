use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Fixed-point amount denominated in the network's base token (e.g. DRK).
/// Stored as integer micro-units (1 DRK = 1_000_000 μDRK).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
