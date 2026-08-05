//! Settlement contract: judge-triggered goal verification and bounty redemption.
//!
//! # Option A — ZK judge attestation (current path)
//!
//! Bounty holders redeem directly from `Active` state.  The judge never posts
//! attestations to AO; instead it shares a per-bounty `judge_attest_key` with
//! the bounty holder privately.  The bounty holder proves knowledge of the key in
//! the BURN ZK circuit.  The settlement contract checks that the resulting
//! `judge_attest_pk` is in `bounty.judge.judge_attest_pks`.
//!
//! # Instructions
//!
//! | Instruction              | Description                                         |
//! |--------------------------|-----------------------------------------------------|
//! | `SubmitAttestation`      | (deprecated) Judge posts signed attestation        |
//! | `FinaliseVerification`   | (deprecated) Close judge window once quorum met    |
//! | `RedeemBounty`             | Bounty holder redeems with ZK BURN proof               |
//! | `ClaimExpiredCollateral` | Issuer reclaims collateral after deadline/fail      |

use nyxforge_core::bounty::{Bounty, BountyId, BountyState};
use nyxforge_core::judge_spec::{JudgeAttestation, QuorumResult};
use nyxforge_core::types::Digest;
use nyxforge_zk::burn::BurnProof;
use serde::{Deserialize, Serialize};

use crate::ContractResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitAttestationParams {
    pub bounty_id:     BountyId,
    pub attestation: JudgeAttestation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinaliseVerificationParams {
    pub bounty_id:  BountyId,
    pub quorum:   QuorumResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedeemBountyParams {
    pub bounty_id:    BountyId,
    pub burn_proof: BurnProof,
}

/// Record a new judge attestation (legacy / slashing-evidence path).
///
/// # Deprecation note
/// Under Option A (ZK judge attestation), judges share `judge_attest_key`
/// privately with the bounty holder.  Redemption goes directly through
/// [`process_redeem_bounty`] without posting attestations to AO.  This function
/// is retained for slashing adjudication only.
pub fn process_submit_attestation(
    bounty: &Bounty,
    params: &SubmitAttestationParams,
) -> ContractResult<u32> {
    // Check the judge is in the approved set.
    let key = &params.attestation.judge_key;
    if !bounty.judge.judge_keys.contains(key) {
        return Err(anyhow::anyhow!("judge not authorised for this bounty").into());
    }

    // TODO: verify Ed25519 signature over canonical bytes.
    // TODO: persist attestation to contract state, return running count.

    tracing::info!(
        bounty_id = ?params.bounty_id,
        goal_met = params.attestation.goal_met,
        "attestation recorded (slashing evidence only)",
    );
    Ok(1) // placeholder count
}

/// Close the judge window once quorum is reached (legacy path).
///
/// # Deprecation note
/// Under Option A, bounty holders redeem directly from `Active` state via
/// [`process_redeem_bounty`].  This function handles the old two-phase path
/// (Active → Redeemable) for bounties that do not have `judge_attest_pks`.
pub fn process_finalise_verification(
    bounty: &Bounty,
    params: &FinaliseVerificationParams,
) -> ContractResult<BountyState> {
    let quorum = &params.quorum;

    let attested = quorum.attestations.len() as u32;
    if attested < bounty.judge.quorum {
        return Err(nyxforge_core::error::NyxError::QuorumNotMet {
            attested,
            required: bounty.judge.quorum,
        });
    }

    if !quorum.is_consistent() {
        return Err(nyxforge_core::error::NyxError::FraudulentAttestation);
    }

    let new_state = if quorum.goal_met {
        BountyState::Redeemable
    } else {
        BountyState::Expired
    };

    tracing::info!(?new_state, "bounty verification finalised");
    Ok(new_state)
}

/// Process a bounty redemption: verify the ZK BURN proof and issue a payout note.
///
/// # Option A path (bounties with `judge_attest_pks`)
///
/// The bounty may be in `Active` or `Redeemable` state.  The contract checks
/// that `burn_proof.judge_attest_pk` is listed in
/// `bounty.judge.judge_attest_pks`, ensuring only an judge-endorsed
/// bounty holder can redeem.  This replaces the on-chain attestation quorum.
///
/// # Legacy path (bounties without `judge_attest_pks`)
///
/// The bounty must be in `Redeemable` state (set by
/// [`process_finalise_verification`]).
pub fn process_redeem_bounty(
    bounty: &Bounty,
    params: &RedeemBountyParams,
) -> ContractResult<Digest> {
    let is_option_a = !bounty.judge.judge_attest_pks.is_empty();

    if is_option_a {
        // Option A: accept Active or Redeemable bounties.
        if bounty.state != BountyState::Active && bounty.state != BountyState::Redeemable {
            return Err(anyhow::anyhow!(
                "bounty must be Active or Redeemable for ZK judge redemption (got {:?})",
                bounty.state
            ).into());
        }

        // Verify the judge_attest_pk from the proof is registered on this bounty.
        let attested_pk = &params.burn_proof.judge_attest_pk;
        if !bounty.judge.judge_attest_pks.contains(attested_pk) {
            return Err(anyhow::anyhow!(
                "judge_attest_pk in burn proof is not registered for this bounty"
            ).into());
        }
    } else {
        // Legacy path: bounty must have reached Redeemable via quorum.
        if bounty.state != BountyState::Redeemable {
            return Err(nyxforge_core::error::NyxError::InvalidBountyState {
                current:  bounty.state.clone(),
                expected: BountyState::Redeemable,
            });
        }
    }

    // Verify the ZK burn proof (proves ownership + correct payout commitment).
    params.burn_proof.verify()
        .map_err(|e| anyhow::anyhow!("burn proof: {e}"))?;

    // TODO: check burn_proof.nullifier not already spent.
    // TODO: emit payout_commitment to DarkFi note tree.
    // TODO: update bounty series supply counter.

    tracing::info!(
        payout = ?params.burn_proof.payout_amount,
        "bounty redeemed",
    );
    Ok(params.burn_proof.payout_commitment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyxforge_test_fixtures::bounties::{active_bounty, single_judge_spec};
    use nyxforge_test_fixtures::proofs::{burn_proof, BURN_JUDGE_ATTEST_KEY};

    /// Helper: an active bounty with one judge_attest_pk registered.
    fn active_bounty_with_attest_pk() -> Bounty {
        let mut bounty = active_bounty();
        let pk = nyxforge_zk::primitives::judge_attest_pk_from_key(&BURN_JUDGE_ATTEST_KEY);
        bounty.judge = single_judge_spec();
        bounty.judge.judge_attest_pks = vec![pk];
        bounty
    }

    #[test]
    fn redeem_active_bond_with_matching_pk_succeeds() {
        let bounty = active_bounty_with_attest_pk();
        let proof = burn_proof(bounty.id);
        let params = RedeemBountyParams { bounty_id: bounty.id, burn_proof: proof };
        assert!(process_redeem_bounty(&bounty, &params).is_ok());
    }

    #[test]
    fn redeem_rejects_unregistered_oracle_attest_pk() {
        let mut bounty = active_bounty_with_attest_pk();
        // Swap in a PK derived from a different key — not in the registered list.
        bounty.judge.judge_attest_pks = vec![[0x00u8; 32]];
        let proof = burn_proof(bounty.id);
        let params = RedeemBountyParams { bounty_id: bounty.id, burn_proof: proof };
        let err = process_redeem_bounty(&bounty, &params).unwrap_err();
        assert!(err.to_string().contains("judge_attest_pk"), "unexpected error: {err}");
    }

    #[test]
    fn redeem_legacy_bond_rejects_active_state() {
        // Legacy bounty: no judge_attest_pks → must be Redeemable, not Active.
        let bounty = active_bounty(); // has judge_attest_pks: [] via quorum_3_judge_spec
        let proof = burn_proof(bounty.id);
        let params = RedeemBountyParams { bounty_id: bounty.id, burn_proof: proof };
        let err = process_redeem_bounty(&bounty, &params).unwrap_err();
        assert!(err.to_string().contains("Redeemable"), "unexpected error: {err}");
    }

    #[test]
    fn submit_attestation_rejects_unauthorised_oracle() {
        use nyxforge_core::judge_spec::JudgeAttestation;
        use nyxforge_core::types::PublicKey;
        use chrono::Utc;

        let bounty = active_bounty_with_attest_pk();
        let unknown_oracle = PublicKey([0xFFu8; 32]);
        let att = JudgeAttestation {
            bounty_id:      bounty.id,
            goal_met:     true,
            evidence_hash: nyxforge_core::types::Digest::zero(),
            evidence_uri: None,
            judge_key:   unknown_oracle,
            signature:    vec![0u8; 64],
            attested_at:  Utc::now(),
        };
        let params = SubmitAttestationParams { bounty_id: bounty.id, attestation: att };
        assert!(process_submit_attestation(&bounty, &params).is_err());
    }
}
