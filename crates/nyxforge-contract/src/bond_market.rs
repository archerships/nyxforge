//! Bond market contract: issue a new bond series and lock collateral.
//!
//! # Instructions
//!
//! | Instruction     | Description                                    |
//! |-----------------|------------------------------------------------|
//! | `IssueBond`     | Create a new bond series, lock issuer collateral |
//! | `CancelBond`    | Issuer cancels a DRAFT bond (pre-listing)     |
//! | `CloseBond`     | Admin/DAO marks a bond EXPIRED after deadline  |

use nyxforge_core::bond::{Bond, BondId, BondState};
use nyxforge_core::types::{Amount, PublicKey};
use serde::{Deserialize, Serialize};

use crate::ContractResult;

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueBondParams {
    pub bond: Bond,
    /// Proof that the issuer has locked `total_supply * redemption_value`
    /// of collateral in the contract's escrow note.
    pub collateral_proof: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelBondParams {
    pub bond_id: BondId,
    /// Issuer's signature authorising cancellation.
    pub issuer_sig: [u8; 64],
}

// ---------------------------------------------------------------------------
// State helpers
// ---------------------------------------------------------------------------

/// Minimum collateral required to issue a bond series.
pub fn required_collateral(bond: &Bond) -> Amount {
    Amount(
        bond.total_supply
            .saturating_mul(bond.redemption_value.0),
    )
}

// ---------------------------------------------------------------------------
// Contract logic
// ---------------------------------------------------------------------------

/// Process an `IssueBond` instruction.
///
/// Verifies:
///   1. The bond's fields are internally consistent.
///   2. The collateral proof demonstrates sufficient locked funds.
///   3. The bond ID matches the canonical derivation.
pub fn process_issue_bond(params: &IssueBondParams) -> ContractResult<BondId> {
    let bond = &params.bond;

    // Verify canonical ID.
    let expected_id = Bond::compute_id(
        &bond.goal,
        &bond.issuer,
        bond.created_at_block,
    );
    if bond.id != expected_id {
        return Err(anyhow::anyhow!("bond id mismatch").into());
    }

    // Verify state is DRAFT.
    if bond.state != BondState::Draft {
        return Err(nyxforge_core::error::NyxError::InvalidBondState {
            current:  bond.state.clone(),
            expected: BondState::Draft,
        });
    }

    // Verify supply > 0.
    if bond.total_supply == 0 {
        return Err(anyhow::anyhow!("total_supply must be > 0").into());
    }

    // Verify oracle quorum > 0 and keys provided.
    if bond.oracle.quorum == 0 || bond.oracle.oracle_keys.is_empty() {
        return Err(anyhow::anyhow!("oracle spec invalid").into());
    }

    // TODO: verify collateral_proof against DarkFi note tree.
    // For now we trust the proof bytes are non-empty.
    if params.collateral_proof.is_empty() {
        return Err(anyhow::anyhow!("collateral proof missing").into());
    }

    tracing::info!(?bond.id, "bond series issued");
    Ok(bond.id)
}

/// Process a `CancelBond` instruction (only valid while DRAFT).
pub fn process_cancel_bond(
    _bond: &Bond,
    _params: &CancelBondParams,
) -> ContractResult<()> {
    // TODO: verify issuer_sig with bond.issuer pubkey.
    // TODO: release collateral back to issuer.
    tracing::info!("bond cancelled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_core::bond::*;
    use nyxforge_core::types::*;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn minimal_bond() -> Bond {
        let goal = GoalSpec {
            title: "Test goal".into(),
            description: "".into(),
            metric: GoalMetric {
                data_id:     "test.metric".into(),
                operator:    ComparisonOp::LessThan,
                threshold:   Decimal::from(100u32),
                aggregation: None,
            },
            evidence_format: None,
            deadline: Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap(),
        };
        let issuer = PublicKey([1u8; 32]);
        let id = Bond::compute_id(&goal, &issuer, 42);
        Bond {
            id,
            issuer: issuer.clone(),
            total_supply: 1000,
            redemption_value: Amount::from_whole(10),
            floor_price: Amount::from_whole(1),
            state: BondState::Draft,
            goal,
            oracle: OracleSpec {
                quorum: 3,
                oracle_keys: vec![PublicKey([2u8; 32])],
                required_stake: Amount::from_whole(100),
                slash_fraction: Decimal::new(5, 2),
            },
            verification: VerificationCriteria {
                attestation_threshold: 3,
                challenge_period_secs: 86_400,
                dao_override_allowed: false,
            },
            created_at_block: 42,
        }
    }

    #[test]
    fn issue_bond_succeeds() {
        let bond = minimal_bond();
        let params = IssueBondParams {
            bond,
            collateral_proof: vec![0xde, 0xad],
        };
        assert!(process_issue_bond(&params).is_ok());
    }

    #[test]
    fn issue_bond_rejects_empty_collateral_proof() {
        let bond = minimal_bond();
        let params = IssueBondParams { bond, collateral_proof: vec![] };
        assert!(process_issue_bond(&params).is_err());
    }
}
