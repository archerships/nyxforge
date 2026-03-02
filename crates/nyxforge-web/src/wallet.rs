//! In-browser wallet: key generation, local encrypted storage, note management.

use anyhow::Result;

const STORAGE_KEY: &str = "nyxforge_wallet_v1";

/// Generate a new keypair and persist it to localStorage (encrypted).
///
/// In production the secret key is encrypted with a password-derived key
/// (PBKDF2 or Argon2id) before storing.
pub fn generate_and_store() -> Result<String> {
    // TODO: generate a real Ristretto255 keypair using DarkFi's crypto primitives.
    // Placeholder: use random bytes.
    let mut secret = [0u8; 32];
    getrandom::getrandom(&mut secret).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Derive public key (placeholder: identity map).
    let public: [u8; 32] = secret; // TODO: real scalar multiplication

    // Persist (plaintext for now; TODO: encrypt with user passphrase).
    let payload = serde_json::json!({
        "secret": hex::encode(secret),
        "public": hex::encode(public),
    });

    // Store in localStorage via web-sys.
    let storage = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("localStorage not available"))?;

    storage.set_item(STORAGE_KEY, &payload.to_string())
        .map_err(|_| anyhow::anyhow!("localStorage write failed"))?;

    Ok(hex::encode(public))
}

/// Load the stored wallet, returning (secret_bytes, public_hex).
pub fn load_wallet() -> Result<([u8; 32], String)> {
    let storage = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("localStorage not available"))?;

    let raw = storage.get_item(STORAGE_KEY)
        .map_err(|_| anyhow::anyhow!("localStorage read failed"))?
        .ok_or_else(|| anyhow::anyhow!("no wallet found — call generate_wallet() first"))?;

    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let secret_hex = v["secret"].as_str().ok_or_else(|| anyhow::anyhow!("bad wallet format"))?;
    let public_hex = v["public"].as_str().ok_or_else(|| anyhow::anyhow!("bad wallet format"))?.to_string();

    let secret_bytes = hex::decode(secret_hex)?;
    let mut secret = [0u8; 32];
    secret.copy_from_slice(&secret_bytes);

    Ok((secret, public_hex))
}
