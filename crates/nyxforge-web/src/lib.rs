//! NyxForge browser WASM frontend.
//!
//! Built with `wasm-pack build --target web`.
//! The resulting JS/WASM is loaded by a plain HTML file — no app store required.
//!
//! Architecture:
//!   - `wallet`   : key generation, note decryption, balance
//!   - `client`   : JSON-RPC calls to the local nyxforge-node
//!   - `ui`       : DOM helpers and page renderers

mod client;
mod ui;
mod wallet;

use wasm_bindgen::prelude::*;

/// Called by the JS bootstrap (`init()`) after WASM is loaded.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Route panics to the browser console.
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"NyxForge WASM loaded".into());
    ui::render_app()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Public JS-callable API
// ---------------------------------------------------------------------------

/// Generate a new wallet keypair, return public key as hex.
#[wasm_bindgen]
pub fn generate_wallet() -> Result<String, JsValue> {
    wallet::generate_and_store()
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Fetch the list of active bonds from the local node.
#[wasm_bindgen]
pub async fn list_bonds() -> Result<String, JsValue> {
    let bonds = client::rpc_call("bonds.list", serde_json::json!({}))
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(bonds.to_string())
}

/// Place a bid or ask order.
#[wasm_bindgen]
pub async fn place_order(order_json: String) -> Result<String, JsValue> {
    let params: serde_json::Value = serde_json::from_str(&order_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = client::rpc_call("orders.place", params)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(result.to_string())
}
