//! RandomX VM wrapper — set up the VM for a given seed hash and hash blobs.

use anyhow::Result;
use randomx_rs::{RandomXCache, RandomXFlag, RandomXVM};

/// A configured RandomX VM ready to hash blobs.
pub struct RandomXHasher {
    vm: RandomXVM,
}

impl RandomXHasher {
    /// Create a new hasher for the given seed hash (32-byte hex string).
    ///
    /// This is expensive — takes ~1-2 s on first call due to dataset init.
    /// Reuse the same `RandomXHasher` for all blobs sharing a seed hash.
    pub fn new(seed_hash: &[u8]) -> Result<Self> {
        let flags = RandomXFlag::get_recommended_flags();
        let cache = RandomXCache::new(flags, seed_hash)
            .map_err(|e| anyhow::anyhow!("RandomX cache init failed: {e}"))?;
        let vm = RandomXVM::new(flags, Some(cache), None)
            .map_err(|e| anyhow::anyhow!("RandomX VM init failed: {e}"))?;
        Ok(Self { vm })
    }

    /// Hash a blob and return 32 output bytes.
    pub fn hash(&mut self, input: &[u8]) -> Result<[u8; 32]> {
        let hash = self.vm.calculate_hash(input)
            .map_err(|e| anyhow::anyhow!("RandomX hash failed: {e}"))?;
        let mut out = [0u8; 32];
        out.copy_from_slice(&hash);
        Ok(out)
    }
}

/// Return true if `hash` (little-endian) meets the compact `target`.
///
/// P2Pool encodes target as a 4-byte big-endian difficulty:
///   difficulty = 0xFFFFFFFF / compact_target
pub fn meets_target(hash: &[u8; 32], target_hex: &str) -> bool {
    // Parse target from hex string (8 hex chars = 4 bytes compact form).
    // Compare last 4 bytes of hash (LE) against target.
    let target_bytes = match u32::from_str_radix(target_hex.trim_start_matches("0x"), 16) {
        Ok(t) => t.to_le_bytes(),
        Err(_) => return false,
    };
    // Check: hash[28..32] (LE u32) <= target
    let hash_tail = u32::from_le_bytes([hash[28], hash[29], hash[30], hash[31]]);
    let target_val = u32::from_le_bytes(target_bytes);
    hash_tail <= target_val
}

#[cfg(test)]
mod tests {
    use super::meets_target;

    // Helper: build a hash whose last 4 bytes (LE u32) equal `tail`.
    fn hash_with_tail(tail: u32) -> [u8; 32] {
        let mut h = [0u8; 32];
        let bytes = tail.to_le_bytes();
        h[28..32].copy_from_slice(&bytes);
        h
    }

    // --- Easy targets (all-zeroes hash should always pass) ---

    #[test]
    fn all_zero_hash_beats_max_target() {
        assert!(meets_target(&[0u8; 32], "ffffffff"));
    }

    #[test]
    fn all_zero_hash_beats_mid_target() {
        assert!(meets_target(&[0u8; 32], "0000ffff"));
    }

    // --- Hard targets (large hash tail should fail) ---

    #[test]
    fn large_tail_fails_strict_target() {
        // tail = 0x0001_0000 > target 0x0000_ffff
        let hash = hash_with_tail(0x0001_0000);
        assert!(!meets_target(&hash, "0000ffff"));
    }

    #[test]
    fn impossible_target_fails() {
        // target = 0x0000_0000 → only all-zero tail passes
        let hash = hash_with_tail(1);
        assert!(!meets_target(&hash, "00000000"));
    }

    // --- Boundary: tail == target ---

    #[test]
    fn tail_equal_to_target_passes() {
        // meets_target uses <=, so tail == target should pass
        let target_val: u32 = 0x0000_ffff;
        let hash = hash_with_tail(target_val);
        assert!(meets_target(&hash, "0000ffff"));
    }

    #[test]
    fn tail_one_above_target_fails() {
        let target_val: u32 = 0x0000_ffff;
        let hash = hash_with_tail(target_val + 1);
        assert!(!meets_target(&hash, "0000ffff"));
    }

    // --- Malformed target ---

    #[test]
    fn invalid_hex_target_returns_false() {
        assert!(!meets_target(&[0u8; 32], "not-hex!"));
    }

    #[test]
    fn empty_target_returns_false() {
        assert!(!meets_target(&[0u8; 32], ""));
    }
}
