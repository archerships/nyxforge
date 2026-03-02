//! Data source adapters — pluggable backends for fetching goal metric data.
//!
//! Each adapter implements `DataSource` and is registered with the oracle node.
//! Multiple adapters for the same data_id provide redundancy.

use anyhow::Result;
use rust_decimal::Decimal;

pub struct VerificationResult {
    pub data_id: String,
    pub value:   Decimal,
    pub source:  String,
}

/// Trait for a pluggable data feed.
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    /// Returns true if this source can supply the given data_id.
    fn supports(&self, data_id: &str) -> bool;

    /// Fetch the current value for the given data_id.
    async fn fetch(&self, data_id: &str) -> Result<Decimal>;

    /// Human-readable name (for logging and attestation metadata).
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Built-in adapters
// ---------------------------------------------------------------------------

/// A mock adapter for testing — always returns a fixed value.
pub struct MockDataSource {
    pub data_id: String,
    pub value:   Decimal,
}

#[async_trait::async_trait]
impl DataSource for MockDataSource {
    fn supports(&self, data_id: &str) -> bool { data_id == self.data_id }
    async fn fetch(&self, _: &str) -> Result<Decimal> { Ok(self.value) }
    fn name(&self) -> &str { "mock" }
}

/// HTTP JSON adapter — fetches a URL and extracts a numeric field via JSON pointer.
pub struct HttpJsonSource {
    pub data_id:      String,
    pub url:          String,
    /// JSON pointer (RFC 6901) to the numeric value, e.g. `/data/value`.
    pub json_pointer: String,
    client:           reqwest::Client,
}

impl HttpJsonSource {
    pub fn new(data_id: impl Into<String>, url: impl Into<String>, json_pointer: impl Into<String>) -> Self {
        Self {
            data_id:      data_id.into(),
            url:          url.into(),
            json_pointer: json_pointer.into(),
            client:       reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl DataSource for HttpJsonSource {
    fn supports(&self, data_id: &str) -> bool { data_id == self.data_id }

    async fn fetch(&self, _: &str) -> Result<Decimal> {
        let json: serde_json::Value = self.client
            .get(&self.url)
            .send().await?
            .json().await?;

        let v = json.pointer(&self.json_pointer)
            .ok_or_else(|| anyhow::anyhow!("pointer {} not found", self.json_pointer))?;

        let s = v.to_string();
        let d = s.trim_matches('"').parse::<Decimal>()
            .map_err(|e| anyhow::anyhow!("parse decimal: {e}"))?;
        Ok(d)
    }

    fn name(&self) -> &str { "http_json" }
}

// ---------------------------------------------------------------------------
// Re-export async-trait
// ---------------------------------------------------------------------------
pub use async_trait;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_source_returns_configured_value() {
        let src = MockDataSource {
            data_id: "test.metric".into(),
            value:   Decimal::from(42u32),
        };
        assert!(src.supports("test.metric"));
        assert!(!src.supports("other.metric"));
        let v = src.fetch("test.metric").await.unwrap();
        assert_eq!(v, Decimal::from(42u32));
    }
}
