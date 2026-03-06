//! Client for the nyxforge-mcp server (MCP protocol + provider management REST).

use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Bond-assist response types (mirrors server's BondAssistance schema)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SimilarBond {
    pub bond_id:     String,
    pub title:       String,
    /// "high", "medium", or "low"
    pub similarity:  String,
    pub explanation: String,
}

#[derive(Debug, Deserialize)]
pub struct SuggestedBondParams {
    pub title:           String,
    pub description:     String,
    pub data_id:         String,
    /// "lt" | "lte" | "gt" | "gte" | "eq"
    pub operator:        String,
    pub threshold:       String,
    pub aggregation:     Option<String>,
    pub evidence_format: Option<String>,
    pub deadline:        String,
    pub notes:           Option<String>,
}

impl SuggestedBondParams {
    /// Map operator string to Select widget index (0-based).
    pub fn operator_idx(&self) -> usize {
        match self.operator.as_str() {
            "lt"  => 0,
            "lte" => 1,
            "gt"  => 2,
            "gte" => 3,
            "eq"  => 4,
            _     => 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BondAssistance {
    pub similar_bonds:  Vec<SimilarBond>,
    pub suggested_bond: SuggestedBondParams,
    pub analysis:       String,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct McpClient {
    base_url: String,
    client:   reqwest::Client,
}

impl McpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client:   reqwest::Client::new(),
        }
    }

    // -- MCP protocol --------------------------------------------------------

    /// Call the `bond_assist` tool on the MCP server.
    pub async fn bond_assist(
        &self,
        description:    &str,
        existing_bonds: &[Value],
    ) -> Result<BondAssistance> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method":  "tools/call",
            "params": {
                "name": "bond_assist",
                "arguments": {
                    "description":    description,
                    "existing_bonds": existing_bonds,
                }
            },
            "id": 1,
        });

        let resp: Value = self.client
            .post(&self.base_url)
            .header("content-type", "application/json")
            .json(&body)
            .send().await
            .map_err(|e| anyhow!(
                "Cannot reach MCP server at {}.\n\
                 Is it running?  Start it with: nyxforge-mcp\n\
                 Error: {e}",
                self.base_url
            ))?
            .json().await
            .map_err(|e| anyhow!("Failed to parse MCP server response: {e}"))?;

        if let Some(err) = resp.get("error") {
            return Err(anyhow!("MCP server error: {err}"));
        }

        let text = resp["result"]["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Unexpected MCP response shape:\n{resp}"))?;

        serde_json::from_str::<BondAssistance>(text)
            .map_err(|e| anyhow!("Could not parse bond_assist result: {e}\n\nRaw:\n{text}"))
    }

    // -- Provider management REST --------------------------------------------

    pub async fn providers(&self) -> Result<Value> {
        self.client
            .get(format!("{}/providers", self.base_url))
            .send().await
            .map_err(|e| anyhow!("MCP server unreachable: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parse error: {e}"))
    }

    pub async fn add_provider(
        &self,
        name:     &str,
        kind:     &str,
        api_key:  Option<&str>,
        base_url: Option<&str>,
        model:    Option<&str>,
    ) -> Result<Value> {
        let body = serde_json::json!({
            "name":     name,
            "kind":     kind,
            "api_key":  api_key,
            "base_url": base_url,
            "model":    model,
        });
        self.client
            .post(format!("{}/providers", self.base_url))
            .json(&body)
            .send().await
            .map_err(|e| anyhow!("MCP server unreachable: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parse error: {e}"))
    }

    pub async fn remove_provider(&self, name: &str) -> Result<Value> {
        self.client
            .delete(format!("{}/providers/{name}", self.base_url))
            .send().await
            .map_err(|e| anyhow!("MCP server unreachable: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parse error: {e}"))
    }

    pub async fn set_default(&self, name: &str) -> Result<Value> {
        let body = serde_json::json!({ "name": name });
        self.client
            .put(format!("{}/providers/default", self.base_url))
            .json(&body)
            .send().await
            .map_err(|e| anyhow!("MCP server unreachable: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Parse error: {e}"))
    }

    pub async fn health(&self) -> Result<String> {
        let text = self.client
            .get(format!("{}/health", self.base_url))
            .send().await
            .map_err(|e| anyhow!(
                "Cannot reach MCP server at {}.\n\
                 Start it with: nyxforge-mcp\n\
                 Error: {e}",
                self.base_url
            ))?
            .text().await
            .map_err(|e| anyhow!("Parse error: {e}"))?;
        Ok(text)
    }
}
