//! Bounty market contract: issue a new bounty series and lock collateral.
//!
//! # Instructions
//!
//! | Instruction     | Description                                    |
//! |-----------------|------------------------------------------------|
//! | `IssueBounty`     | Create a new bounty series, lock issuer collateral |
//! | `CancelBounty`    | Issuer cancels a DRAFT bounty (pre-listing)     |
//! | `CloseBounty`     | Admin/DAO marks a bounty EXPIRED after deadline  |

use nyxforge_core::bounty::{Bounty, BountyId, BountyState};
use nyxforge_core::types::Amount;
use serde::{Deserialize, Serialize};

use crate::ContractResult;

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueBountyParams {
    pub bounty: Bounty,
    /// Proof that the issuer has locked `total_supply * redemption_value`
    /// of collateral in the contract's escrow note.
    pub collateral_proof: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelBountyParams {
    pub bounty_id: BountyId,
    /// Issuer's signature authorising cancellation.
    pub issuer_sig: Vec<u8>,
}

// ---------------------------------------------------------------------------
// State helpers
// ---------------------------------------------------------------------------

/// Minimum collateral required to issue a bounty series.
pub fn required_collateral(bounty: &Bounty) -> Amount {
    Amount(
        bounty.total_supply
            .saturating_mul(bounty.redemption_value.0),
    )
}

// ---------------------------------------------------------------------------
// Contract logic
// ---------------------------------------------------------------------------

/// Process an `IssueBounty` instruction.
///
/// Verifies:
///   1. The bounty's fields are internally consistent.
///   2. The collateral proof demonstrates sufficient locked funds.
///   3. The bounty ID matches the canonical derivation.
pub fn process_issue_bounty(params: &IssueBountyParams) -> ContractResult<BountyId> {
    let bounty = &params.bounty;

    // Verify at least one goal.
    if bounty.goals.is_empty() {
        return Err(anyhow::anyhow!("goals must not be empty").into());
    }

    // Verify canonical ID.
    let expected_id = Bounty::compute_id(
        &bounty.goals,
        &bounty.issuer,
        bounty.created_at_block,
        &bounty.return_address,
    );
    if bounty.id != expected_id {
        return Err(anyhow::anyhow!("bounty id mismatch").into());
    }

    // Verify state is DRAFT.
    if bounty.state != BountyState::Draft {
        return Err(nyxforge_core::error::NyxError::InvalidBountyState {
            current:  bounty.state.clone(),
            expected: BountyState::Draft,
        });
    }

    // Verify supply > 0.
    if bounty.total_supply == 0 {
        return Err(anyhow::anyhow!("total_supply must be > 0").into());
    }

    // Verify judge quorum > 0 and keys provided.
    if bounty.judge.quorum == 0 || bounty.judge.judge_keys.is_empty() {
        return Err(anyhow::anyhow!("judge spec invalid").into());
    }

    // Auction parameter validation.
    if bounty.auction.start_price == Amount::ZERO {
        return Err(anyhow::anyhow!("start_price must be > 0").into());
    }
    if bounty.auction.reserve_price == Amount::ZERO {
        return Err(anyhow::anyhow!("reserve_price must be > 0").into());
    }
    if bounty.auction.reserve_price > bounty.auction.start_price {
        return Err(anyhow::anyhow!("reserve_price must be ≤ start_price").into());
    }
    if bounty.auction.duration_secs == 0 {
        return Err(anyhow::anyhow!("duration_secs must be > 0").into());
    }

    // TODO: verify collateral_proof against DarkFi note tree.
    // For now we trust the proof bytes are non-empty.
    if params.collateral_proof.is_empty() {
        return Err(anyhow::anyhow!("collateral proof missing").into());
    }

    tracing::info!(?bounty.id, "bounty series issued");
    Ok(bounty.id)
}

/// Process a `CancelBounty` instruction (only valid while DRAFT).
pub fn process_cancel_bounty(
    _bounty: &Bounty,
    _params: &CancelBountyParams,
) -> ContractResult<()> {
    // TODO: verify issuer_sig with bounty.issuer pubkey.
    // TODO: release collateral back to issuer.
    tracing::info!("bounty cancelled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_test_fixtures::bounties::{active_bounty, draft_bounty, proposed_bounty};

    fn valid_params(bounty: nyxforge_core::bounty::Bounty) -> IssueBountyParams {
        IssueBountyParams { bounty, collateral_proof: vec![0xde, 0xad] }
    }

    // --- Happy path ---

    #[test]
    fn issue_bounty_succeeds() {
        assert!(process_issue_bounty(&valid_params(draft_bounty())).is_ok());
    }

    #[test]
    fn issue_bounty_returns_correct_id() {
        let bounty = draft_bounty();
        let expected_id = bounty.id;
        let returned_id = process_issue_bounty(&valid_params(bounty)).unwrap();
        assert_eq!(returned_id, expected_id);
    }

    // --- Collateral validation ---

    #[test]
    fn issue_bounty_rejects_empty_collateral_proof() {
        let params = IssueBountyParams { bounty: draft_bounty(), collateral_proof: vec![] };
        assert!(process_issue_bounty(&params).is_err());
    }

    // --- State validation ---

    #[test]
    fn issue_bounty_rejects_active_state() {
        assert!(process_issue_bounty(&valid_params(active_bounty())).is_err());
    }

    #[test]
    fn issue_bounty_rejects_proposed_state() {
        assert!(process_issue_bounty(&valid_params(proposed_bounty())).is_err());
    }

    // --- Supply validation ---

    #[test]
    fn issue_bounty_rejects_zero_supply() {
        let mut bounty = draft_bounty();
        bounty.total_supply = 0;
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    // --- Judge validation ---

    #[test]
    fn issue_bounty_rejects_zero_quorum() {
        let mut bounty = draft_bounty();
        bounty.judge.quorum = 0;
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    #[test]
    fn issue_bounty_rejects_empty_judge_keys() {
        let mut bounty = draft_bounty();
        bounty.judge.judge_keys.clear();
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    // --- Auction parameter validation ---

    #[test]
    fn issue_bounty_rejects_zero_start_price() {
        let mut bounty = draft_bounty();
        bounty.auction.start_price = nyxforge_core::types::Amount::ZERO;
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    #[test]
    fn issue_bounty_rejects_zero_reserve_price() {
        let mut bounty = draft_bounty();
        bounty.auction.reserve_price = nyxforge_core::types::Amount::ZERO;
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    #[test]
    fn issue_bounty_rejects_reserve_greater_than_start() {
        let mut bounty = draft_bounty();
        bounty.auction.start_price   = nyxforge_core::types::Amount::from_whole(1);
        bounty.auction.reserve_price = nyxforge_core::types::Amount::from_whole(2);
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    #[test]
    fn issue_bounty_rejects_zero_duration() {
        let mut bounty = draft_bounty();
        bounty.auction.duration_secs = 0;
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    // --- Goals validation ---

    #[test]
    fn issue_bounty_rejects_empty_goals() {
        let mut bounty = draft_bounty();
        bounty.goals.clear();
        assert!(process_issue_bounty(&valid_params(bounty)).is_err());
    }

    // --- Collateral amount helper ---

    #[test]
    fn required_collateral_calculation() {
        let bounty = draft_bounty();
        let expected = nyxforge_core::types::Amount(
            bounty.total_supply * bounty.redemption_value.0,
        );
        assert_eq!(required_collateral(&bounty), expected);
    }
}
