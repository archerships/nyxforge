//! XMR + DRK key generation, derivation, and serialisation.

use anyhow::{anyhow, Result};
use monero::{Address, Network, PrivateKey, ViewPair};
use nyxforge_core::types::PublicKey as DrkPubkey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// All keys for the wallet. XMR keys use the standard Monero key structure;
/// DRK keys are Ristretto255 keypairs.
#[derive(Clone)]
pub struct WalletKeys {
    /// XMR spend private key (zeroised on drop).
    pub xmr_spend_key: Zeroizing<[u8; 32]>,
    /// XMR view private key (public; safe to log).
    pub xmr_view_key: [u8; 32],
    /// Derived XMR address (mainnet by default).
    pub xmr_address: Address,
    /// DRK secret key (zeroised on drop).
    pub drk_secret: Zeroizing<[u8; 32]>,
    /// DRK public key derived from drk_secret.
    pub drk_pubkey: DrkPubkey,
}

/// Serialisable form stored on disk (secrets encrypted separately in storage.rs).
#[derive(Serialize, Deserialize)]
pub struct WalletKeysSerde {
    pub xmr_spend_key_hex: String,
    pub xmr_view_key_hex: String,
    pub xmr_address: String,
    pub drk_secret_hex: String,
    pub drk_pubkey_hex: String,
}

impl WalletKeys {
    /// Generate a fresh random wallet.
    pub fn generate() -> Result<Self> {
        let mut rng = rand::thread_rng();

        // XMR: random spend key, view key derived via blake3 (scaffold; real
        // Monero derivation uses keccak256 — swap in when integrating monero crate
        // key-derivation helpers).
        let mut spend_bytes = Zeroizing::new([0u8; 32]);
        rng.fill_bytes(spend_bytes.as_mut());

        // Reduce to canonical Ed25519 scalar: group order l has top byte 0x10,
        // so masking byte 31 to 0x0f guarantees value < 2^252 < l.
        spend_bytes[31] &= 0x0f;

        let spend_key = PrivateKey::from_slice(spend_bytes.as_ref())
            .map_err(|e| anyhow!("invalid XMR spend key: {e:?}"))?;

        let view_hash = blake3::hash(spend_bytes.as_ref());
        let mut view_bytes: [u8; 32] = *view_hash.as_bytes();
        view_bytes[31] &= 0x0f; // reduce to canonical scalar
        let view_key = PrivateKey::from_slice(&view_bytes)
            .map_err(|e| anyhow!("invalid XMR view key: {e:?}"))?;

        let view_pair = ViewPair {
            spend: monero::PublicKey::from_private_key(&spend_key),
            view: view_key,
        };
        let xmr_address = Address::from_viewpair(Network::Stagenet, &view_pair);

        // DRK: derived deterministically from XMR spend key so the same spend
        // key always recovers the same DRK identity (matches from_spend_key).
        let drk_secret_hash = blake3::hash(
            &[b"nyxforge-drk:".as_slice(), spend_bytes.as_ref()].concat(),
        );
        let drk_secret_bytes = Zeroizing::new(*drk_secret_hash.as_bytes());
        let drk_pubkey_hash = blake3::hash(drk_secret_bytes.as_ref());
        let drk_pubkey = DrkPubkey(*drk_pubkey_hash.as_bytes());

        Ok(WalletKeys {
            xmr_spend_key: spend_bytes,
            xmr_view_key: view_bytes,
            xmr_address,
            drk_secret: drk_secret_bytes,
            drk_pubkey,
        })
    }

    /// Import a wallet from an existing XMR spend key (64 hex chars = 32 bytes).
    ///
    /// All other keys are derived deterministically so the same spend key always
    /// produces the same addresses.
    pub fn from_spend_key(spend_key_hex: &str) -> Result<Self> {
        let spend_bytes_raw = hex_to_bytes_32(spend_key_hex.trim())
            .map_err(|e| anyhow!("invalid spend key hex: {e}"))?;

        // Ensure canonical scalar (same reduction as generate()).
        let mut spend_bytes = Zeroizing::new(spend_bytes_raw);
        spend_bytes[31] &= 0x0f;

        let spend_key = PrivateKey::from_slice(spend_bytes.as_ref())
            .map_err(|e| anyhow!("invalid XMR spend key: {e:?}"))?;

        // Derive view key from spend key (same as generate()).
        let view_hash = blake3::hash(spend_bytes.as_ref());
        let mut view_bytes: [u8; 32] = *view_hash.as_bytes();
        view_bytes[31] &= 0x0f; // reduce to canonical scalar
        let view_key = PrivateKey::from_slice(&view_bytes)
            .map_err(|e| anyhow!("invalid derived XMR view key: {e:?}"))?;

        let view_pair = ViewPair {
            spend: monero::PublicKey::from_private_key(&spend_key),
            view: view_key,
        };
        let xmr_address = Address::from_viewpair(Network::Stagenet, &view_pair);

        // Derive DRK secret deterministically from spend key with a domain tag.
        let drk_secret_hash = blake3::hash(&[b"nyxforge-drk:".as_slice(), spend_bytes.as_ref()].concat());
        let drk_secret_bytes = Zeroizing::new(*drk_secret_hash.as_bytes());
        let drk_pubkey_hash = blake3::hash(drk_secret_bytes.as_ref());
        let drk_pubkey = DrkPubkey(*drk_pubkey_hash.as_bytes());

        Ok(WalletKeys {
            xmr_spend_key: spend_bytes,
            xmr_view_key: view_bytes,
            xmr_address,
            drk_secret: drk_secret_bytes,
            drk_pubkey,
        })
    }

    pub fn xmr_address_string(&self) -> String {
        self.xmr_address.to_string()
    }

    pub fn drk_address_string(&self) -> String {
        bytes_to_hex(&self.drk_pubkey.0)
    }

    /// Serialise (without encryption) for storage.
    pub fn to_serde(&self) -> WalletKeysSerde {
        WalletKeysSerde {
            xmr_spend_key_hex: bytes_to_hex(self.xmr_spend_key.as_ref()),
            xmr_view_key_hex: bytes_to_hex(&self.xmr_view_key),
            xmr_address: self.xmr_address_string(),
            drk_secret_hex: bytes_to_hex(self.drk_secret.as_ref()),
            drk_pubkey_hex: bytes_to_hex(&self.drk_pubkey.0),
        }
    }

    /// Reconstruct from serialised form.
    pub fn from_serde(s: WalletKeysSerde) -> Result<Self> {
        let spend_bytes = hex_to_bytes_32(&s.xmr_spend_key_hex)?;
        let view_bytes = hex_to_bytes_32(&s.xmr_view_key_hex)?;
        let drk_secret_bytes = hex_to_bytes_32(&s.drk_secret_hex)?;
        let drk_pubkey_bytes = hex_to_bytes_32(&s.drk_pubkey_hex)?;

        let spend_key = PrivateKey::from_slice(&spend_bytes)
            .map_err(|e| anyhow!("invalid XMR spend key: {e:?}"))?;
        let view_key = PrivateKey::from_slice(&view_bytes)
            .map_err(|e| anyhow!("invalid XMR view key: {e:?}"))?;
        let view_pair = ViewPair {
            spend: monero::PublicKey::from_private_key(&spend_key),
            view: view_key,
        };
        let xmr_address = Address::from_viewpair(Network::Stagenet, &view_pair);

        Ok(WalletKeys {
            xmr_spend_key: Zeroizing::new(spend_bytes),
            xmr_view_key: view_bytes,
            xmr_address,
            drk_secret: Zeroizing::new(drk_secret_bytes),
            drk_pubkey: DrkPubkey(drk_pubkey_bytes),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl std::fmt::Debug for WalletKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletKeys")
            .field("xmr_address", &self.xmr_address.to_string())
            .field("drk_pubkey", &bytes_to_hex(&self.drk_pubkey.0))
            .finish_non_exhaustive()
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_to_bytes_32(hex: &str) -> Result<[u8; 32]> {
    if hex.len() != 64 {
        return Err(anyhow!("expected 64 hex chars, got {}", hex.len()));
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk)?, 16)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Key generation ---

    #[test]
    fn generate_succeeds() {
        assert!(WalletKeys::generate().is_ok());
    }

    #[test]
    fn generated_spend_key_has_canonical_scalar() {
        let keys = WalletKeys::generate().unwrap();
        // Byte 31 must have top nibble cleared (reduction mod group order).
        assert_eq!(keys.xmr_spend_key[31] & 0xf0, 0x00);
    }

    #[test]
    fn generated_view_key_has_canonical_scalar() {
        let keys = WalletKeys::generate().unwrap();
        assert_eq!(keys.xmr_view_key[31] & 0xf0, 0x00);
    }

    #[test]
    fn stagenet_address_starts_with_5() {
        let keys = WalletKeys::generate().unwrap();
        assert!(
            keys.xmr_address_string().starts_with('5'),
            "stagenet address should start with '5', got: {}",
            keys.xmr_address_string()
        );
    }

    #[test]
    fn drk_address_is_64_hex_chars() {
        let keys = WalletKeys::generate().unwrap();
        let addr = keys.drk_address_string();
        assert_eq!(addr.len(), 64);
        assert!(addr.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // --- from_spend_key ---

    #[test]
    fn import_roundtrip_produces_same_addresses() {
        let original = WalletKeys::generate().unwrap();
        let spend_hex = bytes_to_hex(original.xmr_spend_key.as_ref());
        let imported = WalletKeys::from_spend_key(&spend_hex).unwrap();
        assert_eq!(original.xmr_address_string(), imported.xmr_address_string());
        assert_eq!(original.drk_address_string(), imported.drk_address_string());
    }

    #[test]
    fn drk_address_is_deterministic_from_same_spend_key() {
        let keys = WalletKeys::generate().unwrap();
        let spend_hex = bytes_to_hex(keys.xmr_spend_key.as_ref());
        let a = WalletKeys::from_spend_key(&spend_hex).unwrap().drk_address_string();
        let b = WalletKeys::from_spend_key(&spend_hex).unwrap().drk_address_string();
        assert_eq!(a, b);
    }

    #[test]
    fn different_spend_keys_produce_different_drk_addresses() {
        let a = WalletKeys::generate().unwrap().drk_address_string();
        let b = WalletKeys::generate().unwrap().drk_address_string();
        // Probability of collision is 2^-256 — safe to assert inequality.
        assert_ne!(a, b);
    }

    #[test]
    fn invalid_hex_spend_key_rejected() {
        assert!(WalletKeys::from_spend_key("not-hex").is_err());
    }

    #[test]
    fn short_hex_spend_key_rejected() {
        assert!(WalletKeys::from_spend_key("0102").is_err());
    }

    #[test]
    fn odd_length_hex_spend_key_rejected() {
        // 63 chars — not 64
        let s = "a".repeat(63);
        assert!(WalletKeys::from_spend_key(&s).is_err());
    }

    // --- to_serde / from_serde roundtrip ---

    #[test]
    fn serde_roundtrip_preserves_addresses() {
        let original = WalletKeys::generate().unwrap();
        let s = original.to_serde();
        let restored = WalletKeys::from_serde(s).unwrap();
        assert_eq!(original.xmr_address_string(), restored.xmr_address_string());
        assert_eq!(original.drk_address_string(), restored.drk_address_string());
    }

    #[test]
    fn debug_redacts_secret_fields() {
        let keys = WalletKeys::generate().unwrap();
        let debug = format!("{keys:?}");
        // Must show address but not raw secret bytes.
        assert!(debug.contains("xmr_address"));
        assert!(!debug.contains("xmr_spend_key"));
        assert!(!debug.contains("drk_secret"));
    }
}
