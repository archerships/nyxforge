//! Mock JSON-RPC client for testing CLI commands without a live node.
//!
//! # Usage
//!
//! ```rust
//! use nyxforge_test_fixtures::MockRpcClient;
//! use serde_json::json;
//!
//! let rpc = MockRpcClient::new()
//!     .with_response("bonds.list", json!([]))
//!     .with_response("wallet.addresses", json!({
//!         "xmr": "5...",
//!         "drk": "aabbcc...",
//!     }));
//!
//! // Pass `rpc` into the command under test.
//! ```
//!
//! Any method without a configured response returns an error, which exercises
//! the CLI's error-handling path.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

/// A synchronous/async mock RPC client.
///
/// Callers configure per-method responses with [`with_response`].
/// Calls to unconfigured methods return an error.
pub struct MockRpcClient {
    responses: HashMap<String, Value>,
    /// If `true`, every call records its method+params for later inspection.
    pub record_calls: bool,
    calls: std::sync::Mutex<Vec<(String, Value)>>,
}

impl MockRpcClient {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            record_calls: false,
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Pre-configure a response for the given RPC method.
    ///
    /// The response value will be returned verbatim (as the `result` field).
    pub fn with_response(mut self, method: impl Into<String>, response: Value) -> Self {
        self.responses.insert(method.into(), response);
        self
    }

    /// Enable call recording.  Inspect via [`recorded_calls`].
    pub fn recording(mut self) -> Self {
        self.record_calls = true;
        self
    }

    /// Simulate a JSON-RPC call.
    ///
    /// Returns the pre-configured `result` value, or an error if none was set.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        if self.record_calls {
            self.calls.lock().unwrap().push((method.to_string(), params));
        }
        self.responses
            .get(method)
            .cloned()
            .ok_or_else(|| anyhow!("MockRpcClient: no response configured for '{method}'"))
    }

    /// Return all recorded (method, params) pairs in call order.
    pub fn recorded_calls(&self) -> Vec<(String, Value)> {
        self.calls.lock().unwrap().clone()
    }

    /// Assert that `method` was called at least once.
    pub fn assert_called(&self, method: &str) {
        let calls = self.recorded_calls();
        assert!(
            calls.iter().any(|(m, _)| m == method),
            "expected RPC method '{method}' to be called, but it was not.\n\
             Calls recorded: {:?}",
            calls.iter().map(|(m, _)| m.as_str()).collect::<Vec<_>>(),
        );
    }

    /// Assert that `method` was never called.
    pub fn assert_not_called(&self, method: &str) {
        let calls = self.recorded_calls();
        assert!(
            !calls.iter().any(|(m, _)| m == method),
            "expected RPC method '{method}' NOT to be called, but it was.",
        );
    }
}

impl Default for MockRpcClient {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pre-built client configurations for common test scenarios
// ---------------------------------------------------------------------------

/// A client pre-loaded with typical happy-path responses.
///
/// Useful as a baseline — override specific methods by calling
/// [`with_response`] after construction.
pub fn happy_path_client(xmr_address: &str, drk_address: &str, bond_id: &str) -> MockRpcClient {
    MockRpcClient::new()
        .with_response("wallet.addresses", json!({
            "xmr": xmr_address,
            "drk": drk_address,
        }))
        .with_response("bonds.list", json!([]))
        .with_response("bonds.issue", json!({ "id": bond_id }))
        .with_response("bonds.get", json!({
            "id": bond_id,
            "state": "Active",
            "goal": { "title": "Test Goal" },
        }))
        .with_response("miner.status", json!({
            "running": false,
            "threads": 0,
            "hashrate_hps": 0,
        }))
}

/// A client where every call fails — for testing error-handling paths.
pub fn failing_client() -> MockRpcClient {
    MockRpcClient::new() // no responses configured → all calls error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn configured_method_returns_value() {
        let rpc = MockRpcClient::new()
            .with_response("bonds.list", json!([]));
        let v = rpc.call("bonds.list", json!({})).await.unwrap();
        assert_eq!(v, json!([]));
    }

    #[tokio::test]
    async fn unconfigured_method_returns_error() {
        let rpc = MockRpcClient::new();
        assert!(rpc.call("unknown.method", json!({})).await.is_err());
    }

    #[tokio::test]
    async fn recording_captures_calls() {
        let rpc = MockRpcClient::new()
            .with_response("bonds.list", json!([]))
            .recording();

        rpc.call("bonds.list", json!({"a": 1})).await.unwrap();
        rpc.call("bonds.list", json!({"b": 2})).await.unwrap();

        let calls = rpc.recorded_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, "bonds.list");
    }

    #[tokio::test]
    async fn assert_called_passes_when_called() {
        let rpc = MockRpcClient::new()
            .with_response("wallet.addresses", json!({}))
            .recording();
        rpc.call("wallet.addresses", json!({})).await.unwrap();
        rpc.assert_called("wallet.addresses");
    }

    #[tokio::test]
    async fn assert_not_called_passes_when_absent() {
        let rpc = MockRpcClient::new().recording();
        rpc.assert_not_called("wallet.addresses");
    }

    #[test]
    #[should_panic]
    fn assert_called_panics_when_absent() {
        let rpc = MockRpcClient::new().recording();
        rpc.assert_called("bonds.list");
    }

    #[tokio::test]
    async fn happy_path_client_responds() {
        let rpc = happy_path_client("5abc", "0xdrk", "deadbeef");
        let v = rpc.call("wallet.addresses", json!({})).await.unwrap();
        assert_eq!(v["xmr"], "5abc");
    }

    #[tokio::test]
    async fn failing_client_always_errors() {
        let rpc = failing_client();
        assert!(rpc.call("bonds.list", json!({})).await.is_err());
        assert!(rpc.call("wallet.create", json!({})).await.is_err());
    }
}
