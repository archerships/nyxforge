//! Canonical bond fixtures.
//!
//! All constructors are deterministic — no `Utc::now()`, no `rand`.
//! Fixed-byte keys are chosen to be obviously synthetic (`0x11`, `0x22`, …).

use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;

use nyxforge_core::bond::{
    AuctionParams, Bond, BondState, GoalMetric, GoalSpec, OracleSpec, VerificationCriteria,
    ComparisonOp,
};
use nyxforge_core::types::{Amount, PublicKey};

// ---------------------------------------------------------------------------
// Fixed keys — obviously synthetic, never valid on mainnet.
// ---------------------------------------------------------------------------

/// Canonical issuer key used across test suites.
pub const ISSUER_KEY: PublicKey = PublicKey([0x11u8; 32]);

/// First oracle key (used in single-oracle and quorum tests).
pub const ORACLE_KEY_A: PublicKey = PublicKey([0x22u8; 32]);

/// Second oracle key (used in multi-oracle quorum tests).
pub const ORACLE_KEY_B: PublicKey = PublicKey([0x33u8; 32]);

/// Third oracle key (completes a 3-of-3 quorum).
pub const ORACLE_KEY_C: PublicKey = PublicKey([0x44u8; 32]);

// ---------------------------------------------------------------------------
// GoalSpec constructors
// ---------------------------------------------------------------------------

/// The canonical "homelessness" goal used throughout the test suite.
///
/// `us.hud.pit_count.unsheltered < 50_000` by 2030-01-01.
pub fn homelessness_goal() -> GoalSpec {
    GoalSpec {
        title: "US Unsheltered Homelessness Below 50k by 2030".into(),
        description: "Annual HUD PIT unsheltered count must fall below 50,000.".into(),
        metric: GoalMetric {
            data_id: "us.hud.pit_count.unsheltered".into(),
            operator: ComparisonOp::LessThan,
            threshold: Decimal::from(50_000u32),
            aggregation: Some("annual_point_in_time".into()),
        },
        evidence_format: Some("HUD PIT PDF + SHA-256 checksum".into()),
        deadline: Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap(),
    }
}

/// Atmospheric CO₂ goal — tests decimal thresholds and long deadlines.
///
/// `noaa.co2.monthly_mean_ppm < 350.0` by 2045-01-01.
pub fn co2_goal() -> GoalSpec {
    GoalSpec {
        title: "Atmospheric CO₂ Below 350 ppm by 2045".into(),
        description: "Annual mean CO₂ at Mauna Loa < 350 ppm.".into(),
        metric: GoalMetric {
            data_id: "noaa.co2.monthly_mean_ppm".into(),
            operator: ComparisonOp::LessThan,
            threshold: Decimal::new(350, 0),
            aggregation: Some("annual_mean".into()),
        },
        evidence_format: None,
        deadline: Utc.with_ymd_and_hms(2045, 1, 1, 0, 0, 0).unwrap(),
    }
}

/// Malaria goal — tests `LessThan` with a sub-1 decimal threshold.
///
/// `who.malaria.deaths_per_100k < 1.0` by 2035-01-01.
pub fn malaria_goal() -> GoalSpec {
    GoalSpec {
        title: "Malaria Deaths Below 1 per 100k by 2035".into(),
        description: "WHO global malaria deaths per 100k < 1.0.".into(),
        metric: GoalMetric {
            data_id: "who.malaria.deaths_per_100k".into(),
            operator: ComparisonOp::LessThan,
            threshold: Decimal::new(1, 0),
            aggregation: Some("global_annual".into()),
        },
        evidence_format: None,
        deadline: Utc.with_ymd_and_hms(2035, 1, 1, 0, 0, 0).unwrap(),
    }
}

/// Minimal valid GoalSpec — used when tests need a bond but don't care about the goal.
///
/// `test.metric < 100` by 2030-01-01, no aggregation, no evidence format.
pub fn minimal_goal() -> GoalSpec {
    GoalSpec {
        title: "Test Goal".into(),
        description: String::new(),
        metric: GoalMetric {
            data_id: "test.metric".into(),
            operator: ComparisonOp::LessThan,
            threshold: Decimal::from(100u32),
            aggregation: None,
        },
        evidence_format: None,
        deadline: Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap(),
    }
}

// ---------------------------------------------------------------------------
// OracleSpec constructors
// ---------------------------------------------------------------------------

/// Three-oracle quorum spec (A, B, C) — all must agree.
pub fn quorum_3_oracle_spec() -> OracleSpec {
    OracleSpec {
        quorum: 3,
        oracle_keys: vec![ORACLE_KEY_A, ORACLE_KEY_B, ORACLE_KEY_C],
        required_stake: Amount::from_whole(100),
        slash_fraction: Decimal::new(50, 2), // 0.50
    }
}

/// Single-oracle spec (A only) — used for tests that need the simplest oracle setup.
pub fn single_oracle_spec() -> OracleSpec {
    OracleSpec {
        quorum: 1,
        oracle_keys: vec![ORACLE_KEY_A],
        required_stake: Amount::from_whole(100),
        slash_fraction: Decimal::new(50, 2),
    }
}

// ---------------------------------------------------------------------------
// VerificationCriteria constructors
// ---------------------------------------------------------------------------

/// Default verification: unanimous quorum, 24-hour challenge window, no DAO override.
pub fn default_verification() -> VerificationCriteria {
    VerificationCriteria {
        attestation_threshold: 3,
        challenge_period_secs: 86_400,
        dao_override_allowed: false,
    }
}

/// Relaxed verification for single-oracle tests.
pub fn single_oracle_verification() -> VerificationCriteria {
    VerificationCriteria {
        attestation_threshold: 1,
        challenge_period_secs: 3_600,
        dao_override_allowed: false,
    }
}

// ---------------------------------------------------------------------------
// Bond constructors
// ---------------------------------------------------------------------------

/// Bond in `Draft` state — all oracles accepted, ready for `bonds.issue`.
///
/// Uses [`minimal_goal`] and [`single_oracle_spec`] to minimise boilerplate.
/// `return_address` is set to [`ISSUER_KEY`].
pub fn draft_bond() -> Bond {
    let goals = vec![minimal_goal()];
    let id = Bond::compute_id(&goals, &ISSUER_KEY, 42, &ISSUER_KEY);
    Bond {
        id,
        issuer:            ISSUER_KEY,
        total_supply:      1_000,
        redemption_value:  Amount::from_whole(10),
        auction:           AuctionParams {
            start_price:   Amount::from_whole(1),
            reserve_price: Amount::from_whole(1),
            duration_secs: 604_800,
        },
        bonds_remaining:   1_000,
        activated_at_secs: None,
        state:             BondState::Draft,
        goals,
        oracle:            single_oracle_spec(),
        verification:      single_oracle_verification(),
        created_at_block:  42,
        return_address:    ISSUER_KEY,
    }
}

/// Bond in `Active` state — collateral locked, notes circulating.
pub fn active_bond() -> Bond {
    Bond { state: BondState::Active, ..draft_bond() }
}

/// Bond in `Proposed` state — published for community review.
pub fn proposed_bond() -> Bond {
    Bond { state: BondState::Proposed, ..draft_bond() }
}

/// Bond in `Expired` state — deadline passed without goal achievement.
pub fn expired_bond() -> Bond {
    Bond { state: BondState::Expired, ..draft_bond() }
}

/// Bond in `Redeemable` state — goal verified, holders may redeem.
pub fn redeemable_bond() -> Bond {
    Bond { state: BondState::Redeemable, ..draft_bond() }
}

/// Minimal bond used in tests that just need a valid [`Bond`] struct.
///
/// Identical to [`draft_bond`] — provided as a named alias for readability
/// in tests that don't care about state.
pub fn minimal_bond() -> Bond {
    draft_bond()
}

// ---------------------------------------------------------------------------
// Lifebond keys (separate from general oracle pool)
// ---------------------------------------------------------------------------

/// Oracle A — geriatric physician, panel lead.
pub const LIFEBOND_ORACLE_KEY_A: PublicKey = PublicKey([0x55u8; 32]);

/// Oracle B — second geriatric physician.
pub const LIFEBOND_ORACLE_KEY_B: PublicKey = PublicKey([0x66u8; 32]);

/// Oracle C — third geriatric physician.
pub const LIFEBOND_ORACLE_KEY_C: PublicKey = PublicKey([0x77u8; 32]);

/// Oracle D — independent actuary (probabilistic verification).
pub const LIFEBOND_ORACLE_KEY_D: PublicKey = PublicKey([0x88u8; 32]);

/// Oracle E — government vital records certifier.
pub const LIFEBOND_ORACLE_KEY_E: PublicKey = PublicKey([0x99u8; 32]);

// ---------------------------------------------------------------------------
// Lifebond constructors
// ---------------------------------------------------------------------------

/// Lifebond criterion 1: subject is alive.
///
/// `subject.lifebond_001.vital_status >= 1` (1 = alive) by 2125-01-01.
pub fn lifebond_alive_goal() -> GoalSpec {
    GoalSpec {
        title: "Subject Is Alive".into(),
        description: "The bond subject must be certified alive by vital records authorities.".into(),
        metric: GoalMetric {
            data_id:     "subject.lifebond_001.vital_status".into(),
            operator:    ComparisonOp::GreaterThanOrEqual,
            threshold:   Decimal::from(1u32),
            aggregation: Some("vital_records_certification".into()),
        },
        evidence_format: Some("Government-issued vital records certification".into()),
        deadline: Utc.with_ymd_and_hms(2125, 1, 1, 0, 0, 0).unwrap(),
    }
}

/// Lifebond criterion 2: subject is in good health.
///
/// `subject.lifebond_001.health_score >= 80` by 2125-01-01.
pub fn lifebond_health_goal() -> GoalSpec {
    GoalSpec {
        title: "Subject Is in Good Health (Score ≥ 80)".into(),
        description: concat!(
            "The bond subject must be independently certified in good health ",
            "by a licensed geriatric physician panel (biometric health score ≥ 80/100).",
        ).into(),
        metric: GoalMetric {
            data_id:     "subject.lifebond_001.health_score".into(),
            operator:    ComparisonOp::GreaterThanOrEqual,
            threshold:   Decimal::from(80u32),
            aggregation: Some("medical_panel_assessment".into()),
        },
        evidence_format: Some(
            "Biometric health score ≥ 80 from independent geriatric physician panel".into(),
        ),
        deadline: Utc.with_ymd_and_hms(2125, 1, 1, 0, 0, 0).unwrap(),
    }
}

/// Five-oracle medical panel spec for a lifebond (4-of-5 quorum).
///
/// Oracles: three geriatric physicians, one actuary, one vital records certifier.
/// High required stake (10,000 DRK) reflects the gravity of attesting a
/// multi-decade commitment. 75% slash fraction for fraudulent attestation.
pub fn lifebond_oracle_spec() -> OracleSpec {
    OracleSpec {
        quorum:          4,
        oracle_keys:     vec![
            LIFEBOND_ORACLE_KEY_A,
            LIFEBOND_ORACLE_KEY_B,
            LIFEBOND_ORACLE_KEY_C,
            LIFEBOND_ORACLE_KEY_D,
            LIFEBOND_ORACLE_KEY_E,
        ],
        required_stake:  Amount::from_whole(10_000),
        slash_fraction:  Decimal::new(75, 2), // 0.75
    }
}

/// Verification criteria for a lifebond.
///
/// 30-day challenge window: adequate time for independent review of a claimed
/// 120-year milestone. DAO override is permitted because the oracle composition
/// may need governance adjustment over the multi-decade bond lifetime.
pub fn lifebond_verification() -> VerificationCriteria {
    VerificationCriteria {
        attestation_threshold: 4,
        challenge_period_secs: 2_592_000, // 30 days
        dao_override_allowed:  true,
    }
}

/// A lifebond as described in the Archerships "Lifebonds" proposal.
///
/// The issuer locks collateral that pays out to bond holders only when the
/// subject is BOTH alive AND in good health (AND semantics). At the floor
/// price this represents roughly a 2,000× return on goal achievement.
///
/// Economics (in DRK):
/// - 10,000 bonds × 100 DRK face value = 1,000,000 DRK collateral locked
/// - Floor price: 0.05 DRK per bond
///
/// `return_address` is set to [`ISSUER_KEY`] — collateral returns to issuer
/// if the criteria are not both met.
pub fn lifebond() -> Bond {
    let goals = vec![lifebond_alive_goal(), lifebond_health_goal()];
    let id    = Bond::compute_id(&goals, &ISSUER_KEY, 1, &ISSUER_KEY);
    Bond {
        id,
        issuer:            ISSUER_KEY,
        total_supply:      10_000,
        redemption_value:  Amount::from_whole(100), // 100 DRK face value
        auction:           AuctionParams {
            start_price:   Amount::from_whole(1),
            reserve_price: Amount(50_000),           // 0.05 DRK reserve
            duration_secs: 604_800,
        },
        bonds_remaining:   10_000,
        activated_at_secs: Some(1),
        state:             BondState::Active,
        goals,
        oracle:            lifebond_oracle_spec(),
        verification:      lifebond_verification(),
        created_at_block:  1,
        return_address:    ISSUER_KEY,
    }
}

/// A bond using the full three-oracle quorum and the homelessness goal.
///
/// Use this when the goal content or oracle set matters for the test.
/// `return_address` is set to [`ISSUER_KEY`].
pub fn homelessness_bond() -> Bond {
    let goals = vec![homelessness_goal()];
    let id = Bond::compute_id(&goals, &ISSUER_KEY, 100, &ISSUER_KEY);
    Bond {
        id,
        issuer:            ISSUER_KEY,
        total_supply:      10_000,
        redemption_value:  Amount::from_whole(50),
        auction:           AuctionParams {
            start_price:   Amount::from_whole(5),
            reserve_price: Amount::from_whole(5),
            duration_secs: 604_800,
        },
        bonds_remaining:   10_000,
        activated_at_secs: Some(100),
        state:             BondState::Active,
        goals,
        oracle:            quorum_3_oracle_spec(),
        verification:      default_verification(),
        created_at_block:  100,
        return_address:    ISSUER_KEY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_bond_id_is_deterministic() {
        assert_eq!(draft_bond().id, draft_bond().id);
    }

    #[test]
    fn homelessness_bond_id_differs_from_draft_bond() {
        assert_ne!(draft_bond().id, homelessness_bond().id);
    }

    #[test]
    fn state_variants_share_id() {
        assert_eq!(draft_bond().id, active_bond().id);
        assert_eq!(draft_bond().id, expired_bond().id);
    }

    #[test]
    fn collateral_amount() {
        let b = draft_bond();
        let expected = Amount(b.total_supply * b.redemption_value.0);
        let computed = Amount(b.total_supply.checked_mul(b.redemption_value.0).unwrap());
        assert_eq!(expected, computed);
    }

    // --- lifebond ---

    #[test]
    fn lifebond_id_is_deterministic() {
        assert_eq!(lifebond().id, lifebond().id);
    }

    #[test]
    fn lifebond_id_differs_from_other_bonds() {
        assert_ne!(lifebond().id, draft_bond().id);
        assert_ne!(lifebond().id, homelessness_bond().id);
    }

    #[test]
    fn lifebond_has_two_goals() {
        let b = lifebond();
        assert_eq!(b.goals.len(), 2);
    }

    #[test]
    fn lifebond_alive_goal_requires_vital_status_gte_1() {
        let goal = lifebond_alive_goal();
        assert_eq!(goal.metric.operator, ComparisonOp::GreaterThanOrEqual);
        assert_eq!(goal.metric.threshold, Decimal::from(1u32));
        assert_eq!(goal.metric.data_id, "subject.lifebond_001.vital_status");
    }

    #[test]
    fn lifebond_health_goal_requires_score_gte_80() {
        let goal = lifebond_health_goal();
        assert_eq!(goal.metric.operator, ComparisonOp::GreaterThanOrEqual);
        assert_eq!(goal.metric.threshold, Decimal::from(80u32));
        assert_eq!(goal.metric.data_id, "subject.lifebond_001.health_score");
    }

    #[test]
    fn lifebond_both_goals_must_pass() {
        let alive  = lifebond_alive_goal();
        let health = lifebond_health_goal();
        // Both pass
        assert!(alive.metric.operator.evaluate(Decimal::from(1u32), alive.metric.threshold));
        assert!(health.metric.operator.evaluate(Decimal::from(80u32), health.metric.threshold));
        // Alive passes but health fails
        assert!(!health.metric.operator.evaluate(Decimal::from(79u32), health.metric.threshold));
        // Health passes but alive fails
        assert!(!alive.metric.operator.evaluate(Decimal::from(0u32), alive.metric.threshold));
    }

    #[test]
    fn lifebond_oracle_requires_4_of_5_quorum() {
        let spec = lifebond_oracle_spec();
        assert_eq!(spec.quorum, 4);
        assert_eq!(spec.oracle_keys.len(), 5);
    }

    #[test]
    fn lifebond_collateral_does_not_overflow() {
        let b = lifebond();
        // 10,000 bonds × 100_000_000 micro-DRK = 1_000_000_000_000 — well within u64
        let collateral = b.total_supply.checked_mul(b.redemption_value.0);
        assert!(collateral.is_some());
        assert_eq!(collateral.unwrap(), 1_000_000_000_000);
    }

    #[test]
    fn lifebond_auction_reserve_is_fractional_drk() {
        let b = lifebond();
        // 50_000 micro-DRK = 0.05 DRK
        assert_eq!(b.auction.reserve_price.0, 50_000);
    }

    #[test]
    fn lifebond_verification_allows_dao_override() {
        assert!(lifebond_verification().dao_override_allowed);
    }

    #[test]
    fn lifebond_challenge_period_is_30_days() {
        assert_eq!(lifebond_verification().challenge_period_secs, 2_592_000);
    }
}
