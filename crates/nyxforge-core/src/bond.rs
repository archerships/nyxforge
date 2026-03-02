use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::{Amount, Digest, PublicKey};

/// Unique 32-byte identifier for a bond series, derived as
/// `blake3(goal_spec_bytes || issuance_timestamp || issuer_pubkey)`.
pub type BondId = Digest;

/// High-level lifecycle state of a bond series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondState {
    /// Defined but not yet collateralised / listed.
    Draft,
    /// Collateral locked, bonds circulating on the market.
    Active,
    /// Oracle network has confirmed goal achievement; holders may redeem.
    Redeemable,
    /// All bonds in series have been redeemed.
    Settled,
    /// Deadline passed without goal achievement; collateral returned to issuer.
    Expired,
}

/// Human-readable, machine-parseable specification of the social goal
/// that must be achieved for redemption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSpec {
    /// Short human-readable title (e.g. "US street homelessness < 50 000 by 2030").
    pub title: String,

    /// Detailed description and measurement methodology.
    pub description: String,

    /// The specific metric that must be satisfied.
    pub metric: GoalMetric,

    /// Optional supporting evidence format expected from oracles.
    pub evidence_format: Option<String>,

    /// Latest datetime by which the goal must be achieved.
    pub deadline: DateTime<Utc>,
}

/// A measurable predicate over a data stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalMetric {
    /// Canonical identifier for the data series (e.g. "us.hud.pit_count").
    pub data_id: String,

    /// Comparison operator applied to the data value.
    pub operator: ComparisonOp,

    /// Target value (stored as decimal string to avoid float imprecision).
    pub threshold: Decimal,

    /// Optional measurement aggregation (annual average, single observation, etc.).
    pub aggregation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}

/// Specification of the oracle network trusted to verify the goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleSpec {
    /// Minimum number of oracles that must independently attest the result.
    pub quorum: u32,

    /// Public keys of approved oracle nodes for this bond series.
    pub oracle_keys: Vec<PublicKey>,

    /// Stake (in base token) that each oracle must post as collateral.
    pub required_stake: Amount,

    /// Slash fraction (0.0–1.0) if oracle posts fraudulent data.
    pub slash_fraction: Decimal,
}

/// Additional conditions that govern verification and settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCriteria {
    /// Number of independent oracle attestations required.
    pub attestation_threshold: u32,

    /// How long the challenge window is open after an attestation.
    pub challenge_period_secs: u64,

    /// Whether a DAO governance vote can override oracle consensus.
    pub dao_override_allowed: bool,
}

/// A bond series — the on-chain record that backs individual bond notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bond {
    /// Unique identifier for this bond series.
    pub id: BondId,

    /// Compressed public key of the issuer.
    pub issuer: PublicKey,

    /// Total number of bond units issued.
    pub total_supply: u64,

    /// Amount (in base token) redeemable per bond unit upon goal achievement.
    pub redemption_value: Amount,

    /// Minimum ask price at issuance (denominated in base token per bond).
    pub floor_price: Amount,

    /// Current lifecycle state.
    pub state: BondState,

    /// Goal that must be met for redemption.
    pub goal: GoalSpec,

    /// Oracle network configuration.
    pub oracle: OracleSpec,

    /// Verification and settlement rules.
    pub verification: VerificationCriteria,

    /// Block height at which this series was recorded.
    pub created_at_block: u64,
}

impl Bond {
    /// Compute the bond's identifier from its canonical fields.
    pub fn compute_id(
        goal: &GoalSpec,
        issuer: &PublicKey,
        created_at_block: u64,
    ) -> BondId {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"nyxforge::bond_id");
        hasher.update(&issuer.0);
        hasher.update(&created_at_block.to_le_bytes());
        // include title as disambiguator
        hasher.update(goal.title.as_bytes());
        hasher.update(goal.deadline.timestamp().to_le_bytes().as_ref());
        Digest::from(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn example_goal() -> GoalSpec {
        GoalSpec {
            title: "US street homelessness below 50 000 by 2030".into(),
            description: "Annual HUD PIT count < 50 000 in any year before deadline".into(),
            metric: GoalMetric {
                data_id: "us.hud.pit_count.sheltered_and_unsheltered".into(),
                operator: ComparisonOp::LessThan,
                threshold: Decimal::from(50_000u32),
                aggregation: Some("annual_point_in_time".into()),
            },
            evidence_format: Some("HUD PIT PDF + SHA-256 checksum".into()),
            deadline: Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn bond_id_is_deterministic() {
        let goal = example_goal();
        let issuer = PublicKey([42u8; 32]);
        let id1 = Bond::compute_id(&goal, &issuer, 1000);
        let id2 = Bond::compute_id(&goal, &issuer, 1000);
        assert_eq!(id1, id2);
    }

    #[test]
    fn bond_id_differs_by_issuer() {
        let goal = example_goal();
        let id1 = Bond::compute_id(&goal, &PublicKey([1u8; 32]), 1000);
        let id2 = Bond::compute_id(&goal, &PublicKey([2u8; 32]), 1000);
        assert_ne!(id1, id2);
    }
}
