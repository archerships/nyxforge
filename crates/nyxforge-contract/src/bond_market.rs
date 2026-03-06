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
use nyxforge_core::types::Amount;
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
    pub issuer_sig: Vec<u8>,
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

    // Verify at least one goal.
    if bond.goals.is_empty() {
        return Err(anyhow::anyhow!("goals must not be empty").into());
    }

    // Verify canonical ID.
    let expected_id = Bond::compute_id(
        &bond.goals,
        &bond.issuer,
        bond.created_at_block,
        &bond.return_address,
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

    // Auction parameter validation.
    if bond.auction.start_price == Amount::ZERO {
        return Err(anyhow::anyhow!("start_price must be > 0").into());
    }
    if bond.auction.reserve_price == Amount::ZERO {
        return Err(anyhow::anyhow!("reserve_price must be > 0").into());
    }
    if bond.auction.reserve_price > bond.auction.start_price {
        return Err(anyhow::anyhow!("reserve_price must be ≤ start_price").into());
    }
    if bond.auction.duration_secs == 0 {
        return Err(anyhow::anyhow!("duration_secs must be > 0").into());
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
    use nyxforge_test_fixtures::bonds::{active_bond, draft_bond, proposed_bond};

    fn valid_params(bond: nyxforge_core::bond::Bond) -> IssueBondParams {
        IssueBondParams { bond, collateral_proof: vec![0xde, 0xad] }
    }

    // --- Happy path ---

    #[test]
    fn issue_bond_succeeds() {
        assert!(process_issue_bond(&valid_params(draft_bond())).is_ok());
    }

    #[test]
    fn issue_bond_returns_correct_id() {
        let bond = draft_bond();
        let expected_id = bond.id;
        let returned_id = process_issue_bond(&valid_params(bond)).unwrap();
        assert_eq!(returned_id, expected_id);
    }

    // --- Collateral validation ---

    #[test]
    fn issue_bond_rejects_empty_collateral_proof() {
        let params = IssueBondParams { bond: draft_bond(), collateral_proof: vec![] };
        assert!(process_issue_bond(&params).is_err());
    }

    // --- State validation ---

    #[test]
    fn issue_bond_rejects_active_state() {
        assert!(process_issue_bond(&valid_params(active_bond())).is_err());
    }

    #[test]
    fn issue_bond_rejects_proposed_state() {
        assert!(process_issue_bond(&valid_params(proposed_bond())).is_err());
    }

    // --- Supply validation ---

    #[test]
    fn issue_bond_rejects_zero_supply() {
        let mut bond = draft_bond();
        bond.total_supply = 0;
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    // --- Oracle validation ---

    #[test]
    fn issue_bond_rejects_zero_quorum() {
        let mut bond = draft_bond();
        bond.oracle.quorum = 0;
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    #[test]
    fn issue_bond_rejects_empty_oracle_keys() {
        let mut bond = draft_bond();
        bond.oracle.oracle_keys.clear();
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    // --- Auction parameter validation ---

    #[test]
    fn issue_bond_rejects_zero_start_price() {
        let mut bond = draft_bond();
        bond.auction.start_price = nyxforge_core::types::Amount::ZERO;
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    #[test]
    fn issue_bond_rejects_zero_reserve_price() {
        let mut bond = draft_bond();
        bond.auction.reserve_price = nyxforge_core::types::Amount::ZERO;
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    #[test]
    fn issue_bond_rejects_reserve_greater_than_start() {
        let mut bond = draft_bond();
        bond.auction.start_price   = nyxforge_core::types::Amount::from_whole(1);
        bond.auction.reserve_price = nyxforge_core::types::Amount::from_whole(2);
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    #[test]
    fn issue_bond_rejects_zero_duration() {
        let mut bond = draft_bond();
        bond.auction.duration_secs = 0;
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    // --- Goals validation ---

    #[test]
    fn issue_bond_rejects_empty_goals() {
        let mut bond = draft_bond();
        bond.goals.clear();
        assert!(process_issue_bond(&valid_params(bond)).is_err());
    }

    // --- Collateral amount helper ---

    #[test]
    fn required_collateral_calculation() {
        let bond = draft_bond();
        let expected = nyxforge_core::types::Amount(
            bond.total_supply * bond.redemption_value.0,
        );
        assert_eq!(required_collateral(&bond), expected);
    }
}
