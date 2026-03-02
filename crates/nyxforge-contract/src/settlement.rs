//! Settlement contract: oracle-triggered goal verification and bond redemption.
//!
//! # Instructions
//!
//! | Instruction          | Description                                         |
//! |----------------------|-----------------------------------------------------|
//! | `SubmitAttestation`  | Oracle node posts a signed attestation              |
//! | `FinaliseVerification` | Close oracle window once quorum is met            |
//! | `RedeemBond`         | Bond holder redeems after goal verified             |
//! | `ClaimExpiredCollateral` | Issuer reclaims collateral after deadline/fail  |

use nyxforge_core::bond::{Bond, BondId, BondState};
use nyxforge_core::oracle_spec::{OracleAttestation, QuorumResult};
use nyxforge_core::types::Digest;
use nyxforge_zk::burn::BurnProof;
use serde::{Deserialize, Serialize};

use crate::ContractResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitAttestationParams {
    pub bond_id:     BondId,
    pub attestation: OracleAttestation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinaliseVerificationParams {
    pub bond_id:  BondId,
    pub quorum:   QuorumResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedeemBondParams {
    pub bond_id:   BondId,
    pub burn_proof: BurnProof,
}

/// Record a new oracle attestation.
///
/// Verifies the oracle is registered for this bond series and the signature
/// is valid.  Returns the number of attestations collected so far.
pub fn process_submit_attestation(
    bond: &Bond,
    params: &SubmitAttestationParams,
) -> ContractResult<u32> {
    // Check the oracle is in the approved set.
    let key = &params.attestation.oracle_key;
    if !bond.oracle.oracle_keys.contains(key) {
        return Err(anyhow::anyhow!("oracle not authorised for this bond").into());
    }

    // TODO: verify Ed25519 signature params.attestation.signature over canonical bytes.
    // TODO: persist attestation to contract state, return running count.

    tracing::info!(
        bond_id = ?params.bond_id,
        goal_met = params.attestation.goal_met,
        "attestation recorded",
    );
    Ok(1) // placeholder count
}

/// Close the oracle window once quorum is reached and set bond state.
pub fn process_finalise_verification(
    bond: &Bond,
    params: &FinaliseVerificationParams,
) -> ContractResult<BondState> {
    let quorum = &params.quorum;

    // Quorum check.
    let attested = quorum.attestations.len() as u32;
    if attested < bond.oracle.quorum {
        return Err(nyxforge_core::error::NyxError::QuorumNotMet {
            attested,
            required: bond.oracle.quorum,
        });
    }

    // Consistency check — all attestations must agree.
    if !quorum.is_consistent() {
        return Err(nyxforge_core::error::NyxError::FraudulentAttestation);
    }

    let new_state = if quorum.goal_met {
        BondState::Redeemable
    } else {
        BondState::Expired
    };

    tracing::info!(?new_state, "bond verification finalised");
    Ok(new_state)
}

/// Process a bond redemption: verify the ZK burn proof and issue a payout note.
pub fn process_redeem_bond(
    bond: &Bond,
    params: &RedeemBondParams,
) -> ContractResult<Digest> {
    if bond.state != BondState::Redeemable {
        return Err(nyxforge_core::error::NyxError::InvalidBondState {
            current:  bond.state.clone(),
            expected: BondState::Redeemable,
        });
    }

    // Verify the burn proof.
    params.burn_proof.verify()
        .map_err(|e| anyhow::anyhow!("burn proof: {e}"))?;

    // TODO: check burn_proof.nullifier not already spent.
    // TODO: emit payout_commitment to DarkFi note tree.
    // TODO: update bond series supply counter.

    tracing::info!(
        payout = ?params.burn_proof.payout_amount,
        "bond redeemed",
    );
    Ok(params.burn_proof.payout_commitment)
}
