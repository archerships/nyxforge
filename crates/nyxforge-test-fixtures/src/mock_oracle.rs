//! Mock data source for oracle tests.
//!
//! Re-exports [`MockDataSource`] from `nyxforge_oracle::verifier` and adds
//! a convenience constructor for the common case of a single fixed value.

pub use nyxforge_oracle::verifier::MockDataSource;
pub use nyxforge_oracle::verifier::DataSource;

use rust_decimal::Decimal;

/// Create a [`MockDataSource`] that always returns `value` for `data_id`.
///
/// ```
/// # use nyxforge_test_fixtures::mock_oracle::fixed_source;
/// let src = fixed_source("test.metric", 42.0);
/// ```
pub fn fixed_source(data_id: impl Into<String>, value: f64) -> MockDataSource {
    MockDataSource {
        data_id: data_id.into(),
        value: Decimal::try_from(value).expect("value must be finite"),
    }
}

/// Create a [`MockDataSource`] that returns a value just below the given threshold.
///
/// Convenient for tests that need the goal to be *met* (i.e. `value < threshold`).
pub fn source_below(data_id: impl Into<String>, threshold: Decimal) -> MockDataSource {
    MockDataSource {
        data_id: data_id.into(),
        value: threshold - Decimal::new(1, 0),
    }
}

/// Create a [`MockDataSource`] that returns a value equal to the threshold.
///
/// Convenient for testing boundary conditions (`lte` vs `lt`).
pub fn source_at(data_id: impl Into<String>, threshold: Decimal) -> MockDataSource {
    MockDataSource {
        data_id: data_id.into(),
        value: threshold,
    }
}

/// Create a [`MockDataSource`] that returns a value just above the threshold.
pub fn source_above(data_id: impl Into<String>, threshold: Decimal) -> MockDataSource {
    MockDataSource {
        data_id: data_id.into(),
        value: threshold + Decimal::new(1, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fixed_source_returns_value() {
        let src = fixed_source("test.metric", 99.5);
        assert!(src.supports("test.metric"));
        assert!(!src.supports("other"));
        let v = src.fetch("test.metric").await.unwrap();
        assert_eq!(v, Decimal::try_from(99.5).unwrap());
    }

    #[tokio::test]
    async fn source_below_is_strictly_less() {
        let threshold = Decimal::from(100u32);
        let src = source_below("m", threshold);
        let v = src.fetch("m").await.unwrap();
        assert!(v < threshold);
    }

    #[tokio::test]
    async fn source_at_equals_threshold() {
        let threshold = Decimal::from(100u32);
        let src = source_at("m", threshold);
        let v = src.fetch("m").await.unwrap();
        assert_eq!(v, threshold);
    }

    #[tokio::test]
    async fn source_above_is_strictly_greater() {
        let threshold = Decimal::from(100u32);
        let src = source_above("m", threshold);
        let v = src.fetch("m").await.unwrap();
        assert!(v > threshold);
    }
}
