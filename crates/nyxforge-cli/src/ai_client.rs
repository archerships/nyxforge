//! Claude API client for AI-assisted bond creation.
//!
//! Reads ANTHROPIC_API_KEY from the environment.  The node does not need to
//! know about this — all AI calls happen directly from the CLI.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const MODEL:   &str = "claude-sonnet-4-6";

// ---------------------------------------------------------------------------
// Response types
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
    /// Canonical data source ID, e.g. "us.hud.pit_count.unsheltered"
    pub data_id:         String,
    /// "lt" | "lte" | "gt" | "gte" | "eq"
    pub operator:        String,
    /// Decimal number as string, e.g. "50000"
    pub threshold:       String,
    pub aggregation:     Option<String>,
    pub evidence_format: Option<String>,
    /// YYYY-MM-DD
    pub deadline:        String,
    /// Caveats, measurement methodology notes, or open questions
    pub notes:           Option<String>,
}

impl SuggestedBondParams {
    /// Map the operator string to an index in the Select widget (0-based).
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
    /// 2-3 sentence summary of findings and recommendations.
    pub analysis:       String,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct AnthropicClient {
    api_key: String,
    client:  reqwest::Client,
}

impl AnthropicClient {
    /// Create a client.  Call `from_env()` to read the key automatically.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), client: reqwest::Client::new() }
    }

    /// Read `ANTHROPIC_API_KEY` from the environment.
    pub fn from_env() -> Result<Self> {
        let key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            anyhow!(
                "ANTHROPIC_API_KEY environment variable is not set.\n\
                 Get a key at https://console.anthropic.com/ and run:\n\
                 export ANTHROPIC_API_KEY=sk-ant-..."
            )
        })?;
        Ok(Self::new(key))
    }

    /// Analyse a natural-language goal description against a list of existing
    /// bonds (JSON values from `bonds.list`) and return matching bonds plus a
    /// suggested new bond specification.
    pub async fn assist_bond_creation(
        &self,
        description: &str,
        existing_bonds: &[Value],
    ) -> Result<BondAssistance> {
        // Build a compact summary of existing bonds to stay within token limits.
        let bonds_summary: Vec<Value> = existing_bonds.iter().take(30).map(|b| {
            serde_json::json!({
                "bond_id":    b["id"],
                "title":      b["goal"]["title"],
                "description":b["goal"]["description"],
                "data_id":    b["goal"]["metric"]["data_id"],
                "operator":   b["goal"]["metric"]["operator"],
                "threshold":  b["goal"]["metric"]["threshold"],
                "deadline":   b["goal"]["deadline"],
                "state":      b["state"],
            })
        }).collect();

        let user_message = format!(
            "## User's goal\n{description}\n\n\
             ## Existing bonds on this network\n{}",
            serde_json::to_string_pretty(&bonds_summary)?
        );

        let body = serde_json::json!({
            "model":      MODEL,
            "max_tokens": 2048,
            "system":     SYSTEM_PROMPT,
            "messages": [{ "role": "user", "content": user_message }],
        });

        let resp: Value = self.client
            .post(API_URL)
            .header("x-api-key",           &self.api_key)
            .header("anthropic-version",   "2023-06-01")
            .header("content-type",        "application/json")
            .json(&body)
            .send().await
            .map_err(|e| anyhow!("Claude API request failed: {e}"))?
            .json().await
            .map_err(|e| anyhow!("Failed to parse Claude API response: {e}"))?;

        // Check for API-level errors.
        if let Some(err) = resp.get("error") {
            return Err(anyhow!("Claude API error: {err}"));
        }

        let text = resp["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Unexpected Claude API response shape:\n{resp}"))?;

        let json_str = extract_json(text)
            .ok_or_else(|| anyhow!("Could not find JSON in Claude response:\n{text}"))?;

        serde_json::from_str::<BondAssistance>(&json_str)
            .map_err(|e| anyhow!("Could not parse Claude response as BondAssistance: {e}\n\nRaw JSON:\n{json_str}"))
    }
}

// ---------------------------------------------------------------------------
// JSON extraction helper
// ---------------------------------------------------------------------------

/// Extract the first JSON object from text that may contain markdown fences.
fn extract_json(text: &str) -> Option<String> {
    // Try stripping ```json ... ``` fences first.
    if let Some(start) = text.find("```json") {
        let inner = &text[start + 7..];
        if let Some(end) = inner.find("```") {
            return Some(inner[..end].trim().to_owned());
        }
    }
    // Try stripping ``` ... ``` fences.
    if let Some(start) = text.find("```") {
        let inner = &text[start + 3..];
        if let Some(end) = inner.find("```") {
            return Some(inner[..end].trim().to_owned());
        }
    }
    // Try finding a bare JSON object.
    let start = text.find('{')?;
    let end   = text.rfind('}')?;
    if end >= start {
        return Some(text[start..=end].to_owned());
    }
    None
}

// ---------------------------------------------------------------------------
// System prompt
// ---------------------------------------------------------------------------

const SYSTEM_PROMPT: &str = r#"
You are an expert in social policy bonds — financial instruments that pay out
only when a measurable social or environmental goal is achieved.  You help
users design precise, verifiable bonds with unambiguous success criteria.

Given a user's goal description and a list of existing bonds on this network,
you must:
1. Identify existing bonds that overlap with the user's goal (may be empty).
2. Draft a new bond specification based on the user's description.

Reply with ONLY a valid JSON object — no markdown, no prose outside the JSON.
Use this exact schema:

{
  "similar_bonds": [
    {
      "bond_id":     "<id field from the existing bonds list>",
      "title":       "<bond title>",
      "similarity":  "high|medium|low",
      "explanation": "<1–2 sentences: why similar and what differs>"
    }
  ],
  "suggested_bond": {
    "title":           "<concise title, max 60 chars>",
    "description":     "<detailed description of goal and how it will be measured>",
    "data_id":         "<canonical data source id>",
    "operator":        "<lt|lte|gt|gte|eq>",
    "threshold":       "<decimal number as a string>",
    "aggregation":     "<e.g. annual_point_in_time — or null if not needed>",
    "evidence_format": "<e.g. HUD Annual Report PDF — or null>",
    "deadline":        "<YYYY-MM-DD>",
    "notes":           "<caveats, measurement methodology questions, or null>"
  },
  "analysis": "<2–3 sentences summarising your findings and recommendations>"
}

Data ID naming convention: <country_or_org>.<agency>.<metric>[.<variant>]
Common examples:
  us.hud.pit_count.unsheltered          US unsheltered homeless count (HUD PIT)
  us.hud.pit_count.sheltered_and_unsheltered  total US homeless count
  noaa.co2.monthly_mean_ppm             atmospheric CO2 (Mauna Loa)
  who.malaria.deaths_per_100k           WHO malaria mortality
  us.epa.aqi.pm25                       EPA particulate matter index
  world.wb.extreme_poverty_rate         World Bank extreme poverty (<$2.15/day)
  us.cdc.overdose_deaths_per_100k       CDC drug overdose mortality
  us.bls.unemployment_rate              Bureau of Labor Statistics unemployment

Only include bonds with at least "low" similarity.  If none match, return [].
Prefer specificity in data_id — use the most precise sub-metric available.
"#;
