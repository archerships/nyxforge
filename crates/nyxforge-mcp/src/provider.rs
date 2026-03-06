//! Routes `bond_assist` tool calls to the configured AI provider.

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::config::{ProviderEntry, ProviderKind};

// ---------------------------------------------------------------------------
// System prompt (shared across all providers)
// ---------------------------------------------------------------------------

pub const SYSTEM_PROMPT: &str = r#"
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

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Call the AI provider and return the raw text content (JSON string).
pub async fn call_provider(
    entry:          &ProviderEntry,
    description:    &str,
    existing_bonds: &[Value],
) -> Result<String> {
    let user_msg = build_user_message(description, existing_bonds);
    let client   = reqwest::Client::new();

    match entry.kind {
        ProviderKind::Anthropic => call_anthropic(&client, entry, &user_msg).await,
        ProviderKind::OpenAI    => call_openai_compat(&client, entry, &user_msg).await,
        ProviderKind::Ollama    => call_ollama(&client, entry, &user_msg).await,
        ProviderKind::Custom    => call_openai_compat(&client, entry, &user_msg).await,
    }
}

fn build_user_message(description: &str, existing_bonds: &[Value]) -> String {
    let summary: Vec<Value> = existing_bonds.iter().take(30).map(|b| {
        serde_json::json!({
            "bond_id":     b["id"],
            "title":       b["goal"]["title"],
            "description": b["goal"]["description"],
            "data_id":     b["goal"]["metric"]["data_id"],
            "operator":    b["goal"]["metric"]["operator"],
            "threshold":   b["goal"]["metric"]["threshold"],
            "deadline":    b["goal"]["deadline"],
            "state":       b["state"],
        })
    }).collect();

    format!(
        "## User's goal\n{description}\n\n## Existing bonds on this network\n{}",
        serde_json::to_string_pretty(&summary).unwrap_or_default()
    )
}

// ---------------------------------------------------------------------------
// Anthropic
// ---------------------------------------------------------------------------

async fn call_anthropic(
    client:   &reqwest::Client,
    entry:    &ProviderEntry,
    user_msg: &str,
) -> Result<String> {
    let api_key = entry.api_key.as_deref()
        .ok_or_else(|| anyhow!("Anthropic provider requires an api_key"))?;
    let url = format!("{}/v1/messages", entry.effective_base_url());

    let body = serde_json::json!({
        "model":      entry.effective_model(),
        "max_tokens": 2048,
        "system":     SYSTEM_PROMPT,
        "messages": [{ "role": "user", "content": user_msg }],
    });

    let resp: Value = client
        .post(&url)
        .header("x-api-key",          api_key)
        .header("anthropic-version",  "2023-06-01")
        .header("content-type",       "application/json")
        .json(&body)
        .send().await
        .map_err(|e| anyhow!("Anthropic request failed: {e}"))?
        .json().await
        .map_err(|e| anyhow!("Failed to parse Anthropic response: {e}"))?;

    if let Some(err) = resp.get("error") {
        return Err(anyhow!("Anthropic API error: {err}"));
    }

    resp["content"][0]["text"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("Unexpected Anthropic response shape:\n{resp}"))
}

// ---------------------------------------------------------------------------
// OpenAI / Custom (OpenAI-compatible)
// ---------------------------------------------------------------------------

async fn call_openai_compat(
    client:   &reqwest::Client,
    entry:    &ProviderEntry,
    user_msg: &str,
) -> Result<String> {
    let url = format!("{}/v1/chat/completions", entry.effective_base_url());

    let mut req = client.post(&url);
    if let Some(key) = &entry.api_key {
        req = req.bearer_auth(key);
    }

    let body = serde_json::json!({
        "model":      entry.effective_model(),
        "max_tokens": 2048,
        "messages": [
            { "role": "system",  "content": SYSTEM_PROMPT },
            { "role": "user",    "content": user_msg },
        ],
    });

    let resp: Value = req
        .header("content-type", "application/json")
        .json(&body)
        .send().await
        .map_err(|e| anyhow!("OpenAI-compat request failed: {e}"))?
        .json().await
        .map_err(|e| anyhow!("Failed to parse OpenAI-compat response: {e}"))?;

    if let Some(err) = resp.get("error") {
        return Err(anyhow!("AI API error: {err}"));
    }

    resp["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("Unexpected OpenAI-compat response shape:\n{resp}"))
}

// ---------------------------------------------------------------------------
// Ollama
// ---------------------------------------------------------------------------

async fn call_ollama(
    client:   &reqwest::Client,
    entry:    &ProviderEntry,
    user_msg: &str,
) -> Result<String> {
    let url = format!("{}/api/chat", entry.effective_base_url());

    let body = serde_json::json!({
        "model":  entry.effective_model(),
        "stream": false,
        "messages": [
            { "role": "system",  "content": SYSTEM_PROMPT },
            { "role": "user",    "content": user_msg },
        ],
    });

    let resp: Value = client
        .post(&url)
        .header("content-type", "application/json")
        .json(&body)
        .send().await
        .map_err(|e| anyhow!("Ollama request failed: {e}"))?
        .json().await
        .map_err(|e| anyhow!("Failed to parse Ollama response: {e}"))?;

    if let Some(err) = resp.get("error") {
        return Err(anyhow!("Ollama error: {err}"));
    }

    resp["message"]["content"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("Unexpected Ollama response shape:\n{resp}"))
}

// ---------------------------------------------------------------------------
// JSON extraction helper (strip markdown fences from model output)
// ---------------------------------------------------------------------------

pub fn extract_json(text: &str) -> Option<String> {
    // Try ```json ... ``` first.
    if let Some(start) = text.find("```json") {
        let inner = &text[start + 7..];
        if let Some(end) = inner.find("```") {
            return Some(inner[..end].trim().to_owned());
        }
    }
    // Try ``` ... ```.
    if let Some(start) = text.find("```") {
        let inner = &text[start + 3..];
        if let Some(end) = inner.find("```") {
            return Some(inner[..end].trim().to_owned());
        }
    }
    // Bare JSON object.
    let start = text.find('{')?;
    let end   = text.rfind('}')?;
    if end >= start {
        return Some(text[start..=end].to_owned());
    }
    None
}
