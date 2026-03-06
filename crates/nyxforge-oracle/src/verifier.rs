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
    /// The canonical data_id this source handles.
    fn data_id(&self) -> &str;

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
    fn data_id(&self) -> &str { &self.data_id }
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
    fn data_id(&self) -> &str { &self.data_id }
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

    fn fixed_source(data_id: &str, value: f64) -> MockDataSource {
        MockDataSource {
            data_id: data_id.into(),
            value:   Decimal::try_from(value).unwrap(),
        }
    }

    fn source_below(data_id: &str, threshold: Decimal) -> MockDataSource {
        MockDataSource { data_id: data_id.into(), value: threshold - Decimal::ONE }
    }

    fn source_at(data_id: &str, threshold: Decimal) -> MockDataSource {
        MockDataSource { data_id: data_id.into(), value: threshold }
    }

    fn source_above(data_id: &str, threshold: Decimal) -> MockDataSource {
        MockDataSource { data_id: data_id.into(), value: threshold + Decimal::ONE }
    }

    // --- MockDataSource / fixed_source ---

    #[tokio::test]
    async fn mock_source_returns_configured_value() {
        let src = fixed_source("test.metric", 42.0);
        assert!(src.supports("test.metric"));
        assert!(!src.supports("other.metric"));
        let v = src.fetch("test.metric").await.unwrap();
        assert_eq!(v, Decimal::try_from(42.0).unwrap());
    }

    #[tokio::test]
    async fn mock_source_name_is_mock() {
        let src = fixed_source("x", 0.0);
        assert_eq!(src.name(), "mock");
    }

    // --- Boundary helpers ---

    #[tokio::test]
    async fn source_below_is_strictly_less_than_threshold() {
        let threshold = Decimal::from(50_000u32);
        let src = source_below("us.hud.pit_count", threshold);
        let v = src.fetch("us.hud.pit_count").await.unwrap();
        assert!(v < threshold, "expected v < {threshold}, got {v}");
    }

    #[tokio::test]
    async fn source_at_equals_threshold() {
        let threshold = Decimal::from(50_000u32);
        let src = source_at("us.hud.pit_count", threshold);
        let v = src.fetch("us.hud.pit_count").await.unwrap();
        assert_eq!(v, threshold);
    }

    #[tokio::test]
    async fn source_above_is_strictly_greater_than_threshold() {
        let threshold = Decimal::from(50_000u32);
        let src = source_above("us.hud.pit_count", threshold);
        let v = src.fetch("us.hud.pit_count").await.unwrap();
        assert!(v > threshold, "expected v > {threshold}, got {v}");
    }

    // --- Goal evaluation using GoalMetric.operator.evaluate() ---
    // These tests simulate what the oracle verifier does after fetching data.

    #[tokio::test]
    async fn homelessness_goal_met_when_below_threshold() {
        use nyxforge_core::bond::ComparisonOp;
        let threshold = Decimal::from(50_000u32);
        let src = source_below("us.hud.pit_count.unsheltered", threshold);
        let value = src.fetch("us.hud.pit_count.unsheltered").await.unwrap();
        assert!(ComparisonOp::LessThan.evaluate(value, threshold));
    }

    #[tokio::test]
    async fn homelessness_goal_not_met_at_threshold() {
        use nyxforge_core::bond::ComparisonOp;
        let threshold = Decimal::from(50_000u32);
        let src = source_at("us.hud.pit_count.unsheltered", threshold);
        let value = src.fetch("us.hud.pit_count.unsheltered").await.unwrap();
        assert!(!ComparisonOp::LessThan.evaluate(value, threshold));
    }

    #[tokio::test]
    async fn lte_goal_met_exactly_at_threshold() {
        use nyxforge_core::bond::ComparisonOp;
        let threshold = Decimal::from(50_000u32);
        let src = source_at("m", threshold);
        let value = src.fetch("m").await.unwrap();
        assert!(ComparisonOp::LessThanOrEqual.evaluate(value, threshold));
    }
}
