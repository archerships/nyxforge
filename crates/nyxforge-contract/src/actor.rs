use crate::ContractResult;
use nyxforge_core::bounty_v2::{BountyV2, BountyStateV2};
use nyxforge_core::types::Amount;

pub enum BountyMessage {
    Fund(Amount),
    Attest { goal_met: bool, judge_id: u32 },
    Redeem { holder_id: u32 },
}

/// The BountyActor trait as defined in v2.0 spec (DarkFi Actor Model).
pub trait BountyActor {
    fn on_message(&mut self, msg: BountyMessage) -> ContractResult<()>;
}

pub struct SimpleBountyActor {
    pub bounty: BountyV2,
}

impl BountyActor for SimpleBountyActor {
    fn on_message(&mut self, msg: BountyMessage) -> ContractResult<()> {
        match msg {
            BountyMessage::Fund(amt) => {
                self.bounty.yield_accrued.0 += amt.0 / 10; // Mock yield accrual logic
                tracing::info!("Bounty funded, yield accruing");
            },
            BountyMessage::Attest { goal_met, .. } => {
                if goal_met {
                    self.bounty.state = BountyStateV2::Redeemable;
                }
            },
            BountyMessage::Redeem { .. } => {
                if self.bounty.state == BountyStateV2::Redeemable {
                    self.bounty.state = BountyStateV2::Settled;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::types::PublicKey;
    use nyxforge_core::bounty_v2::{GoalSpecV2, AdjudicationMode, CollateralSpecV2};
    use chrono::Utc;

    #[test]
    fn test_actor_state_transitions() {
        let issuer = PublicKey([0u8; 32]);
        let spec = GoalSpecV2 {
            title: "Actor Test".into(),
            description: "Test".into(),
            mode: AdjudicationMode::Automated,
            metric: None,
            metric_vk: None,
            inception: Utc::now(),
            expiry: Utc::now(),
            check_interval_secs: 0,
            collateral: CollateralSpecV2 {
                currency: "DRK".into(),
                amount: Amount::ZERO,
                use_yield_as_endowment: false,
            },
            judges: vec![],
            quorum_threshold: 0,
            dispute_period_secs: 0,
        };

        let mut actor = SimpleBountyActor {
            bounty: BountyV2 {
                id: BountyV2::compute_id(&spec, &issuer),
                issuer,
                spec,
                state: BountyStateV2::Active,
                yield_accrued: Amount::ZERO,
                last_check_at: None,
            }
        };

        actor.on_message(BountyMessage::Attest { goal_met: true, judge_id: 1 }).unwrap();
        assert_eq!(actor.bounty.state, BountyStateV2::Redeemable);

        actor.on_message(BountyMessage::Redeem { holder_id: 1 }).unwrap();
        assert_eq!(actor.bounty.state, BountyStateV2::Settled);
    }
}
