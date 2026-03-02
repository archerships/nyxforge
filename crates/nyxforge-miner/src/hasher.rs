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

    #[test]
    fn target_check() {
        let easy_target = "ffffffff";
        let hash = [0u8; 32];
        assert!(meets_target(&hash, easy_target));
    }
}
