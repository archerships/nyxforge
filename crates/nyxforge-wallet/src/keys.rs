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

        let spend_key = PrivateKey::from_slice(spend_bytes.as_ref())
            .map_err(|e| anyhow!("invalid XMR spend key: {e:?}"))?;

        let view_hash = blake3::hash(spend_bytes.as_ref());
        let view_bytes: [u8; 32] = *view_hash.as_bytes();
        let view_key = PrivateKey::from_slice(&view_bytes)
            .map_err(|e| anyhow!("invalid XMR view key: {e:?}"))?;

        let view_pair = ViewPair {
            spend: monero::PublicKey::from_private_key(&spend_key),
            view: view_key,
        };
        let xmr_address = Address::from_viewpair(Network::Mainnet, &view_pair);

        // DRK: random secret, pubkey = blake3(secret) as a scaffold placeholder.
        // Replace with proper Ristretto scalar mul when darkfi-sdk is integrated.
        let mut drk_secret_bytes = Zeroizing::new([0u8; 32]);
        rng.fill_bytes(drk_secret_bytes.as_mut());
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
        let xmr_address = Address::from_viewpair(Network::Mainnet, &view_pair);

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
