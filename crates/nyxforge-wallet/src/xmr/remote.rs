//! `RemoteMonerod`: connects to an existing monerod node via its JSON-RPC API.
//!
//! Uses raw HTTP calls to the monerod daemon JSON-RPC endpoint rather than
//! a crate wrapper, so the exact monero-rpc version doesn't matter.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

use super::source::{MoneroSource, Output, TxHash};

/// Community nodes useful as defaults (no auth required).
pub const DEFAULT_NODE: &str = "http://node.community.rino.io:18081";

/// Connects to a remote `monerod` via its HTTP JSON-RPC interface.
pub struct RemoteMonerod {
    /// Node URL, e.g. `"http://node.community.rino.io:18081"`.
    pub url: String,
    client: reqwest::Client,
}

impl RemoteMonerod {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::new(),
        }
    }

    async fn rpc<R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<R> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params,
        });
        let resp = self
            .client
            .post(format!("{}/json_rpc", self.url))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let json: serde_json::Value = resp.json().await?;
        if let Some(err) = json.get("error") {
            return Err(anyhow!("monerod RPC error: {err}"));
        }
        serde_json::from_value(json["result"].clone())
            .map_err(|e| anyhow!("failed to parse monerod response: {e}"))
    }
}

#[derive(Deserialize)]
struct BlockCountResult {
    count: u64,
}

#[derive(Deserialize)]
struct SendRawResult {
    #[serde(default)]
    not_relayed: bool,
    #[serde(default)]
    top_hash: String,
}

#[async_trait]
impl MoneroSource for RemoteMonerod {
    async fn get_height(&self) -> Result<u64> {
        debug!(url = %self.url, "get_height");
        let result: BlockCountResult = self
            .rpc("get_block_count", serde_json::json!({}))
            .await?;
        Ok(result.count)
    }

    async fn get_outputs(&self, indices: &[u64]) -> Result<Vec<Output>> {
        debug!(url = %self.url, count = indices.len(), "get_outputs");
        if indices.is_empty() {
            return Ok(Vec::new());
        }
        // TODO: call /get_outs with output indices and parse results.
        // Returning empty for now — scanner.rs drives this and is also stubbed.
        let _ = indices;
        Ok(Vec::new())
    }

    async fn submit_tx(&self, tx_hex: &str) -> Result<TxHash> {
        debug!(url = %self.url, "submit_tx");
        // /sendrawtransaction is a non-JSON-RPC endpoint on monerod.
        let resp = self
            .client
            .post(format!("{}/sendrawtransaction", self.url))
            .json(&serde_json::json!({ "tx_as_hex": tx_hex }))
            .send()
            .await?
            .error_for_status()?;
        let result: SendRawResult = resp.json().await?;
        if result.not_relayed {
            Err(anyhow!("transaction not relayed"))
        } else {
            Ok(result.top_hash)
        }
    }
}
