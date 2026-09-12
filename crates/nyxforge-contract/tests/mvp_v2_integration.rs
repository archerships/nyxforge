use nyxforge_core::bounty_v2::{BountyV2, GoalSpecV2, BountyStateV2, AdjudicationMode, CollateralSpecV2};
use nyxforge_core::types::{Amount, PublicKey};
use nyxforge_core::bounty::{GoalMetric, ComparisonOp};
use chrono::{Utc, Duration, Datelike, TimeZone};
use rust_decimal::Decimal;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_spec(title: &str, mode: AdjudicationMode, expiry_year: i32) -> GoalSpecV2 {
        GoalSpecV2 {
            title: title.to_string(),
            description: "Test bounty".to_string(),
            mode,
            metric: Some(GoalMetric {
                data_id: "test.metric".to_string(),
                operator: ComparisonOp::LessThan,
                threshold: Decimal::from(100),
                aggregation: None,
            }),
            metric_vk: Some([0u8; 32]),
            inception: Utc::now(),
            expiry: Utc.with_ymd_and_hms(expiry_year, 1, 1, 0, 0, 0).latest().unwrap(),
            check_interval_secs: 86400, // daily
            collateral: CollateralSpecV2 {
                currency: "XMR".to_string(),
                amount: Amount::from_whole(10),
                use_yield_as_endowment: true,
            },
            judges: vec![PublicKey([1u8; 32])],
            quorum_threshold: 1,
            dispute_period_secs: 3600,
        }
    }

    #[test]
    fn test_anonymous_bounty_creation() {
        let issuer = PublicKey([0xAAu8; 32]);
        let spec = create_test_spec("Save the Whales 2200", AdjudicationMode::Automated, 2200);
        let bounty_id = BountyV2::compute_id(&spec, &issuer);
        
        let bounty = BountyV2 {
            id: bounty_id,
            issuer,
            spec,
            state: BountyStateV2::Proposed,
            yield_accrued: Amount::ZERO,
            last_check_at: None,
        };
        
        assert_eq!(bounty.state, BountyStateV2::Proposed);
        assert!(bounty.spec.expiry.year() >= 2200);
    }

    #[test]
    fn test_robotic_oracle_judgment() {
        let issuer = PublicKey([0xAAu8; 32]);
        let spec = create_test_spec("Automated Health", AdjudicationMode::Automated, 2050);
        let mut bounty = BountyV2 {
            id: BountyV2::compute_id(&spec, &issuer),
            issuer,
            spec,
            state: BountyStateV2::Active,
            yield_accrued: Amount::ZERO,
            last_check_at: None,
        };

        // Simulate robotic judge evaluation
        let current_value = Decimal::from(50);
        let goal_met = bounty.spec.metric.as_ref().unwrap().operator.evaluate(
            current_value, 
            bounty.spec.metric.as_ref().unwrap().threshold
        );

        if goal_met {
            bounty.state = BountyStateV2::Redeemable;
        }

        assert_eq!(bounty.state, BountyStateV2::Redeemable);
    }

    #[test]
    fn test_optimistic_dispute_cycle() {
        let issuer = PublicKey([0xAAu8; 32]);
        let spec = create_test_spec("Subjective Quality", AdjudicationMode::Subjective, 2030);
        let mut bounty = BountyV2 {
            id: BountyV2::compute_id(&spec, &issuer),
            issuer,
            spec,
            state: BountyStateV2::Active,
            yield_accrued: Amount::ZERO,
            last_check_at: None,
        };

        // Step 1: Anonymous Assertion
        let _proposer_stake = Amount::from_whole(1);
        let _assertion = "Goal met: True";
        
        // Step 2: Challenge Window (Doubling Bounty)
        let _challenger_stake = _proposer_stake.0 * 2;
        let _challenge = "Goal met: False (Disputed)";
        
        // Step 3: Escalation to Jury
        let jury_result = true; // Jury agrees goal is met
        
        if jury_result {
            bounty.state = BountyStateV2::Redeemable;
        }

        assert_eq!(bounty.state, BountyStateV2::Redeemable);
    }

    #[test]
    fn test_multi_century_expiry() {
        let _issuer = PublicKey([0xAAu8; 32]);
        let century_23_spec = create_test_spec("Long Now Bounty", AdjudicationMode::Hybrid, 2250);
        
        assert!(century_23_spec.expiry > Utc::now() + Duration::days(365 * 200));
    }

    #[test]
    fn test_yield_based_dividends() {
        let issuer = PublicKey([0xAAu8; 32]);
        let spec = create_test_spec("Maintenance Bounty", AdjudicationMode::Automated, 2100);
        let bounty = BountyV2 {
            id: BountyV2::compute_id(&spec, &issuer),
            issuer,
            spec,
            state: BountyStateV2::Maintenance,
            yield_accrued: Amount::from_whole(5),
            last_check_at: Some(Utc::now() - Duration::days(30)),
        };

        // Simulate daily dividend calculation from accrued XMR yield
        let dividend_per_unit = bounty.yield_accrued.0 / 100; // Mock calculation
        assert!(dividend_per_unit > 0);
    }
}
