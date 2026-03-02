use anyhow::{anyhow, Result};
use serde_json::Value;

pub struct RpcClient {
    url: String,
    client: reqwest::Client,
}

impl RpcClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            client: reqwest::Client::new(),
        }
    }

    /// Call a JSON-RPC method and return the `result` field.
    /// Returns `Err` if the response contains an `error` field.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let body = serde_json::json!({ "method": method, "params": params });

        let resp: Value = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("RPC request failed: {e}\n  Is the node running?  ./nyxforge --start"))?
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse RPC response: {e}"))?;

        if let Some(err) = resp.get("error").and_then(|e| e.as_str()) {
            return Err(anyhow!("Node error: {err}"));
        }

        resp.get("result")
            .cloned()
            .ok_or_else(|| anyhow!("RPC response missing 'result' field"))
    }
}
