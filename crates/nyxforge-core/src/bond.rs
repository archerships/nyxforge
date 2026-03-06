use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::{Amount, Digest, PublicKey};

/// Parameters governing a Dutch (descending-clock) auction for initial bond sales.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuctionParams {
    /// Ask price at the moment the bond is activated.
    pub start_price: Amount,
    /// Minimum floor; price never falls below this.
    pub reserve_price: Amount,
    /// Length of the descending-price window in seconds.
    pub duration_secs: u64,
}

impl AuctionParams {
    /// Current ask price given `elapsed_secs` since activation.
    /// Returns `reserve_price` once the window has passed.
    pub fn current_price(&self, elapsed_secs: u64) -> Amount {
        if elapsed_secs >= self.duration_secs || self.start_price == self.reserve_price {
            return self.reserve_price;
        }
        let range = self.start_price.0.saturating_sub(self.reserve_price.0);
        let drop  = range.saturating_mul(elapsed_secs) / self.duration_secs;
        Amount(self.start_price.0.saturating_sub(drop))
    }
}

/// Unique 32-byte identifier for a bond series, derived as
/// `blake3(goal_spec_bytes || issuance_timestamp || issuer_pubkey)`.
pub type BondId = Digest;

/// High-level lifecycle state of a bond series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondState {
    /// Published for community review; open for questions and suggestions.
    /// No collateral locked; not tradeable. Issuer can revise before issuing.
    Proposed,
    /// Submitted to the listed oracle nodes for acceptance.
    /// Bond waits here until every oracle accepts. Any rejection returns it
    /// here after the issuer revises the oracle list or threshold.
    PendingOracleApproval,
    /// All listed oracles have accepted responsibility. Ready for collateral lock.
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

impl ComparisonOp {
    pub fn evaluate(&self, value: Decimal, threshold: Decimal) -> bool {
        match self {
            Self::LessThan           => value < threshold,
            Self::LessThanOrEqual    => value <= threshold,
            Self::GreaterThan        => value > threshold,
            Self::GreaterThanOrEqual => value >= threshold,
            Self::Equal              => value == threshold,
        }
    }
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

/// A community question or suggestion on a proposed bond.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondComment {
    /// Unique ID: blake3(bond_id || author || body || created_at).
    pub id: Digest,

    /// The bond this comment is attached to.
    pub bond_id: BondId,

    /// DRK public key of the commenter.
    pub author: PublicKey,

    /// Free-form text — question, suggestion, or correction.
    pub body: String,

    /// Wall-clock time the comment was created.
    pub created_at: DateTime<Utc>,
}

impl BondComment {
    pub fn new(bond_id: BondId, author: PublicKey, body: String) -> Self {
        let created_at = Utc::now();
        let mut h = blake3::Hasher::new();
        h.update(b"nyxforge::bond_comment");
        h.update(bond_id.as_bytes());
        h.update(&author.0);
        h.update(body.as_bytes());
        h.update(&created_at.timestamp().to_le_bytes());
        Self {
            id: Digest::from(h.finalize()),
            bond_id,
            author,
            body,
            created_at,
        }
    }
}

/// An oracle node's acceptance or rejection of responsibility for a bond.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    /// The bond this response is for.
    pub bond_id: BondId,

    /// The oracle that responded.
    pub oracle_key: PublicKey,

    /// `true` = accepted, `false` = rejected.
    pub accepted: bool,

    /// Required when `accepted = false`; explains why the oracle cannot judge
    /// this bond (ambiguous goal, unsupported data source, etc.).
    pub reason: Option<String>,

    /// Wall-clock time the oracle responded.
    pub responded_at: DateTime<Utc>,

    /// Oracle signature over (bond_id || accepted || reason || responded_at).
    /// Stub until real Ed25519 signing is wired in.
    pub signature: Vec<u8>,
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

    /// Dutch auction parameters for initial bond sales.
    pub auction: AuctionParams,

    /// Bonds still available for purchase (decremented on each buy).
    pub bonds_remaining: u64,

    /// Unix timestamp (seconds) when this bond was activated (issued).
    pub activated_at_secs: Option<u64>,

    /// Current lifecycle state.
    pub state: BondState,

    /// Goals that must ALL be met for redemption (AND semantics).
    pub goals: Vec<GoalSpec>,

    /// Oracle network configuration.
    pub oracle: OracleSpec,

    /// Verification and settlement rules.
    pub verification: VerificationCriteria,

    /// Block height at which this series was recorded.
    pub created_at_block: u64,

    /// Address to which collateral is returned if the goal is not met by deadline.
    pub return_address: PublicKey,
}

impl Bond {
    /// Compute the bond's identifier from its canonical fields.
    pub fn compute_id(
        goals: &[GoalSpec],
        issuer: &PublicKey,
        created_at_block: u64,
        return_address: &PublicKey,
    ) -> BondId {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"nyxforge::bond_id");
        hasher.update(&issuer.0);
        hasher.update(&created_at_block.to_le_bytes());
        for goal in goals {
            hasher.update(goal.title.as_bytes());
            hasher.update(goal.deadline.timestamp().to_le_bytes().as_ref());
        }
        hasher.update(&return_address.0);
        Digest::from(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    const ISSUER_KEY: PublicKey = PublicKey([0x11u8; 32]);

    fn homelessness_goal() -> GoalSpec {
        GoalSpec {
            title:           "US street homelessness < 50 000 by 2030".into(),
            description:     "Measured via HUD annual PIT count.".into(),
            metric:          GoalMetric {
                data_id:     "us.hud.pit_count.unsheltered".into(),
                operator:    ComparisonOp::LessThan,
                threshold:   Decimal::from(50_000u32),
                aggregation: None,
            },
            evidence_format: None,
            deadline:        chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
                                 .unwrap()
                                 .with_timezone(&chrono::Utc),
        }
    }

    const RETURN_ADDR: PublicKey = PublicKey([0xAAu8; 32]);

    // --- Bond ID ---

    #[test]
    fn bond_id_is_deterministic() {
        let goal = homelessness_goal();
        let id1 = Bond::compute_id(&[goal.clone()], &ISSUER_KEY, 1000, &RETURN_ADDR);
        let id2 = Bond::compute_id(&[goal], &ISSUER_KEY, 1000, &RETURN_ADDR);
        assert_eq!(id1, id2);
    }

    #[test]
    fn bond_id_differs_by_issuer() {
        let goal = homelessness_goal();
        let id1 = Bond::compute_id(&[goal.clone()], &PublicKey([0x11u8; 32]), 1000, &RETURN_ADDR);
        let id2 = Bond::compute_id(&[goal], &PublicKey([0x22u8; 32]), 1000, &RETURN_ADDR);
        assert_ne!(id1, id2);
    }

    #[test]
    fn bond_id_differs_by_block() {
        let goal = homelessness_goal();
        let id1 = Bond::compute_id(&[goal.clone()], &ISSUER_KEY, 100, &RETURN_ADDR);
        let id2 = Bond::compute_id(&[goal], &ISSUER_KEY, 101, &RETURN_ADDR);
        assert_ne!(id1, id2);
    }

    #[test]
    fn bond_id_differs_by_return_address() {
        let goal = homelessness_goal();
        let id1 = Bond::compute_id(&[goal.clone()], &ISSUER_KEY, 0, &RETURN_ADDR);
        let id2 = Bond::compute_id(&[goal], &ISSUER_KEY, 0, &PublicKey([0xFFu8; 32]));
        assert_ne!(id1, id2);
    }

    #[test]
    fn bond_id_differs_by_title() {
        let mut g1 = homelessness_goal();
        let mut g2 = homelessness_goal();
        g2.title = "Different Title".into();
        let id1 = Bond::compute_id(&[g1.clone()], &ISSUER_KEY, 0, &RETURN_ADDR);
        let id2 = Bond::compute_id(&[g2.clone()], &ISSUER_KEY, 0, &RETURN_ADDR);
        assert_ne!(id1, id2);
        // same title again → same id
        g1.title = g2.title.clone();
        assert_eq!(Bond::compute_id(&[g1], &ISSUER_KEY, 0, &RETURN_ADDR), id2);
    }

    #[test]
    fn bond_id_differs_by_goals_order() {
        let g1 = homelessness_goal();
        let mut g2 = homelessness_goal();
        g2.title = "Second Goal".into();
        let id_ab = Bond::compute_id(&[g1.clone(), g2.clone()], &ISSUER_KEY, 0, &RETURN_ADDR);
        let id_ba = Bond::compute_id(&[g2, g1], &ISSUER_KEY, 0, &RETURN_ADDR);
        assert_ne!(id_ab, id_ba);
    }

    // --- ComparisonOp::evaluate ---

    #[test]
    fn op_lt_passes_below_threshold() {
        assert!(ComparisonOp::LessThan.evaluate(Decimal::from(49u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_lt_fails_at_threshold() {
        assert!(!ComparisonOp::LessThan.evaluate(Decimal::from(50u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_lt_fails_above_threshold() {
        assert!(!ComparisonOp::LessThan.evaluate(Decimal::from(51u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_lte_passes_at_threshold() {
        assert!(ComparisonOp::LessThanOrEqual.evaluate(Decimal::from(50u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_lte_passes_below_threshold() {
        assert!(ComparisonOp::LessThanOrEqual.evaluate(Decimal::from(49u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_lte_fails_above_threshold() {
        assert!(!ComparisonOp::LessThanOrEqual.evaluate(Decimal::from(51u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_gt_passes_above_threshold() {
        assert!(ComparisonOp::GreaterThan.evaluate(Decimal::from(51u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_gt_fails_at_threshold() {
        assert!(!ComparisonOp::GreaterThan.evaluate(Decimal::from(50u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_gte_passes_at_threshold() {
        assert!(ComparisonOp::GreaterThanOrEqual.evaluate(Decimal::from(50u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_gte_passes_above_threshold() {
        assert!(ComparisonOp::GreaterThanOrEqual.evaluate(Decimal::from(51u32), Decimal::from(50u32)));
    }

    #[test]
    fn op_eq_passes_exact_match() {
        assert!(ComparisonOp::Equal.evaluate(Decimal::from(42u32), Decimal::from(42u32)));
    }

    #[test]
    fn op_eq_fails_off_by_one() {
        assert!(!ComparisonOp::Equal.evaluate(Decimal::from(43u32), Decimal::from(42u32)));
        assert!(!ComparisonOp::Equal.evaluate(Decimal::from(41u32), Decimal::from(42u32)));
    }

    #[test]
    fn op_lt_works_with_decimal_threshold() {
        // CO₂ scenario: 349.5 < 350.0 → goal met
        let value     = Decimal::new(3495, 1); // 349.5
        let threshold = Decimal::new(350, 0);  // 350.0
        assert!(ComparisonOp::LessThan.evaluate(value, threshold));
    }

    // --- AuctionParams::current_price ---

    fn test_auction() -> AuctionParams {
        AuctionParams {
            start_price:   Amount(1_000_000), // 1 DRK
            reserve_price: Amount(100_000),   // 0.1 DRK
            duration_secs: 100,
        }
    }

    #[test]
    fn auction_price_at_zero_is_start_price() {
        let a = test_auction();
        assert_eq!(a.current_price(0), a.start_price);
    }

    #[test]
    fn auction_price_at_half_window_is_midpoint() {
        let a = test_auction();
        // range = 900_000, drop = 450_000, price = 550_000
        assert_eq!(a.current_price(50), Amount(550_000));
    }

    #[test]
    fn auction_price_at_full_window_is_reserve() {
        let a = test_auction();
        assert_eq!(a.current_price(100), a.reserve_price);
    }

    #[test]
    fn auction_price_past_window_is_reserve() {
        let a = test_auction();
        assert_eq!(a.current_price(999), a.reserve_price);
    }

    #[test]
    fn auction_price_equal_start_reserve_always_reserve() {
        let a = AuctionParams {
            start_price:   Amount(500_000),
            reserve_price: Amount(500_000),
            duration_secs: 100,
        };
        assert_eq!(a.current_price(0),   Amount(500_000));
        assert_eq!(a.current_price(50),  Amount(500_000));
        assert_eq!(a.current_price(100), Amount(500_000));
    }
}
