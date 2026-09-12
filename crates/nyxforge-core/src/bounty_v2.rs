use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::types::{Amount, Digest, PublicKey};
use crate::bounty::GoalMetric;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjudicationMode {
    Automated,
    Subjective,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralSpecV2 {
    pub currency: String, // "XMR", "DRK", "AR", etc.
    pub amount: Amount,
    pub use_yield_as_endowment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSpecV2 {
    pub title: String,
    pub description: String,
    pub mode: AdjudicationMode,
    pub metric: Option<GoalMetric>, // None for pure subjective
    pub metric_vk: Option<[u8; 32]>, // Halo2 Verifier Key for automated logic
    pub inception: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub check_interval_secs: u64,
    pub collateral: CollateralSpecV2,
    pub judges: Vec<PublicKey>,
    pub quorum_threshold: u32,
    pub dispute_period_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyV2 {
    pub id: Digest,
    pub issuer: PublicKey,
    pub spec: GoalSpecV2,
    pub state: BountyStateV2,
    pub yield_accrued: Amount,
    pub last_check_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BountyStateV2 {
    Proposed,
    Active,
    Redeemable, // Goal met, final payout available
    Maintenance, // Goal met, stability dividends being paid
    Expired,
    Settled,
}

impl BountyV2 {
    pub fn compute_id(spec: &GoalSpecV2, issuer: &PublicKey) -> Digest {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"nyxforge::bounty_v2_id");
        hasher.update(&issuer.0);
        hasher.update(spec.title.as_bytes());
        hasher.update(&spec.inception.timestamp().to_le_bytes());
        hasher.update(&spec.expiry.timestamp().to_le_bytes());
        Digest::from(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn mock_spec() -> GoalSpecV2 {
        GoalSpecV2 {
            title: "Test v2".to_string(),
            description: "Desc".to_string(),
            mode: AdjudicationMode::Automated,
            metric: None,
            metric_vk: None,
            inception: Utc::now(),
            expiry: Utc::now() + Duration::days(365 * 100),
            check_interval_secs: 3600,
            collateral: CollateralSpecV2 {
                currency: "DRK".to_string(),
                amount: Amount::from_whole(100),
                use_yield_as_endowment: false,
            },
            judges: vec![],
            quorum_threshold: 1,
            dispute_period_secs: 60,
        }
    }

    #[test]
    fn test_id_derivation() {
        let issuer = PublicKey([1u8; 32]);
        let spec = mock_spec();
        let id1 = BountyV2::compute_id(&spec, &issuer);
        let id2 = BountyV2::compute_id(&spec, &issuer);
        assert_eq!(id1, id2);
    }
}
