//! Provider configuration — loaded from / saved to
//! `$HOME/.config/nyxforge/mcp.json`.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Supported AI backend kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Anthropic,
    OpenAI,
    Ollama,
    /// Any OpenAI-compatible endpoint (custom base URL).
    Custom,
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderKind::Anthropic => write!(f, "anthropic"),
            ProviderKind::OpenAI   => write!(f, "openai"),
            ProviderKind::Ollama   => write!(f, "ollama"),
            ProviderKind::Custom   => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "anthropic"          => Ok(Self::Anthropic),
            "openai"             => Ok(Self::OpenAI),
            "ollama"             => Ok(Self::Ollama),
            "custom"             => Ok(Self::Custom),
            other                => anyhow::bail!("unknown provider kind: {other}"),
        }
    }
}

/// Configuration for a single named AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEntry {
    /// Provider kind: anthropic | openai | ollama | custom.
    pub kind: ProviderKind,
    /// API key (required for Anthropic and OpenAI; optional for others).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Base URL — required for Ollama and Custom; ignored for the cloud defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Model name/ID.  Falls back to a sensible default per kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl ProviderEntry {
    /// Return the effective model string (supplied or per-kind default).
    pub fn effective_model(&self) -> &str {
        if let Some(m) = &self.model {
            return m.as_str();
        }
        match self.kind {
            ProviderKind::Anthropic => "claude-sonnet-4-6",
            ProviderKind::OpenAI    => "gpt-4o",
            ProviderKind::Ollama    => "llama3",
            ProviderKind::Custom    => "default",
        }
    }

    /// Return the effective base URL (supplied or per-kind default).
    pub fn effective_base_url(&self) -> &str {
        if let Some(u) = &self.base_url {
            return u.as_str();
        }
        match self.kind {
            ProviderKind::Anthropic => "https://api.anthropic.com",
            ProviderKind::OpenAI    => "https://api.openai.com",
            ProviderKind::Ollama    => "http://localhost:11434",
            ProviderKind::Custom    => "http://localhost:8080",
        }
    }
}

// ---------------------------------------------------------------------------
// Config file
// ---------------------------------------------------------------------------

/// Top-level structure of `~/.config/nyxforge/mcp.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// Name of the currently active provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    /// Named providers.
    #[serde(default)]
    pub providers: HashMap<String, ProviderEntry>,
}

impl McpConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home)
            .join(".config")
            .join("nyxforge")
            .join("mcp.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&raw)
            .with_context(|| format!("parsing {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, raw)
            .with_context(|| format!("writing {}", path.display()))
    }

    /// Add a named provider and return `&mut Self` for chaining.
    pub fn add_provider(&mut self, name: impl Into<String>, entry: ProviderEntry) -> &mut Self {
        self.providers.insert(name.into(), entry);
        self
    }

    /// Remove a named provider. Returns `true` if it existed.
    pub fn remove_provider(&mut self, name: &str) -> bool {
        self.providers.remove(name).is_some()
    }

    /// Return the active `ProviderEntry`, or an error if none is configured.
    pub fn active_provider(&self) -> Result<(&str, &ProviderEntry)> {
        let name = self.default_provider.as_deref()
            .ok_or_else(|| anyhow::anyhow!(
                "No default AI provider set.\n\
                 Run: nyxforge-cli mcp add <name> --kind <anthropic|openai|ollama|custom>"
            ))?;
        let entry = self.providers.get(name)
            .ok_or_else(|| anyhow::anyhow!(
                "Default provider '{name}' not found in config. \
                 Add it with: nyxforge-cli mcp add {name} ..."
            ))?;
        Ok((name, entry))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn anthropic_entry() -> ProviderEntry {
        ProviderEntry { kind: ProviderKind::Anthropic, api_key: Some("sk-test".into()), base_url: None, model: None }
    }

    fn ollama_entry() -> ProviderEntry {
        ProviderEntry { kind: ProviderKind::Ollama, api_key: None, base_url: None, model: None }
    }

    // --- active_provider ---

    #[test]
    fn active_provider_returns_configured_entry() {
        let mut cfg = McpConfig::default();
        cfg.providers.insert("main".into(), anthropic_entry());
        cfg.default_provider = Some("main".into());
        let (name, entry) = cfg.active_provider().unwrap();
        assert_eq!(name, "main");
        assert_eq!(entry.kind, ProviderKind::Anthropic);
    }

    #[test]
    fn missing_default_provider_returns_error() {
        assert!(McpConfig::default().active_provider().is_err());
    }

    #[test]
    fn default_set_but_provider_absent_returns_error() {
        let mut cfg = McpConfig::default();
        cfg.default_provider = Some("ghost".into());
        assert!(cfg.active_provider().is_err());
    }

    // --- effective_model ---

    #[test]
    fn effective_model_anthropic_default() {
        assert_eq!(anthropic_entry().effective_model(), "claude-sonnet-4-6");
    }

    #[test]
    fn effective_model_openai_default() {
        let e = ProviderEntry { kind: ProviderKind::OpenAI, api_key: None, base_url: None, model: None };
        assert_eq!(e.effective_model(), "gpt-4o");
    }

    #[test]
    fn effective_model_ollama_default() {
        assert_eq!(ollama_entry().effective_model(), "llama3");
    }

    #[test]
    fn explicit_model_overrides_default() {
        let e = ProviderEntry { kind: ProviderKind::Anthropic, api_key: None, base_url: None, model: Some("claude-opus-4-6".into()) };
        assert_eq!(e.effective_model(), "claude-opus-4-6");
    }

    // --- effective_base_url ---

    #[test]
    fn effective_base_url_ollama_is_localhost() {
        assert_eq!(ollama_entry().effective_base_url(), "http://localhost:11434");
    }

    #[test]
    fn effective_base_url_anthropic_is_api() {
        assert_eq!(anthropic_entry().effective_base_url(), "https://api.anthropic.com");
    }

    #[test]
    fn explicit_base_url_overrides_default() {
        let e = ProviderEntry { kind: ProviderKind::Ollama, api_key: None, base_url: Some("http://gpu-box:11434".into()), model: None };
        assert_eq!(e.effective_base_url(), "http://gpu-box:11434");
    }

    // --- add / remove provider ---

    #[test]
    fn add_and_remove_provider() {
        let mut cfg = McpConfig::default();
        cfg.add_provider("test", ollama_entry());
        assert!(cfg.providers.contains_key("test"));
        assert!(cfg.remove_provider("test"));
        assert!(!cfg.providers.contains_key("test"));
    }

    #[test]
    fn remove_absent_provider_returns_false() {
        let mut cfg = McpConfig::default();
        assert!(!cfg.remove_provider("nonexistent"));
    }

    // --- JSON roundtrip ---

    #[test]
    fn config_roundtrips_via_json() {
        let mut cfg = McpConfig::default();
        cfg.add_provider("claude", anthropic_entry());
        cfg.default_provider = Some("claude".into());

        let json = serde_json::to_string(&cfg).unwrap();
        let restored: McpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.default_provider.as_deref(), Some("claude"));
        assert!(restored.providers.contains_key("claude"));
    }

    // --- ProviderKind parsing ---

    #[test]
    fn provider_kind_from_str_roundtrips() {
        for (s, expected) in [
            ("anthropic", ProviderKind::Anthropic),
            ("openai",    ProviderKind::OpenAI),
            ("ollama",    ProviderKind::Ollama),
            ("custom",    ProviderKind::Custom),
        ] {
            let parsed: ProviderKind = s.parse().unwrap();
            assert_eq!(parsed, expected);
            assert_eq!(parsed.to_string(), s);
        }
    }

    #[test]
    fn unknown_provider_kind_returns_error() {
        assert!("gemini".parse::<ProviderKind>().is_err());
    }
}
