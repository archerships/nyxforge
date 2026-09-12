use thiserror::Error;

#[derive(Debug, Error)]
pub enum NyxError {
    #[error("bounty not found: {0:?}")]
    BountyNotFound(crate::types::Digest),

    #[error("bounty is in state {current:?}, expected {expected:?}")]
    InvalidBountyState {
        current:  crate::bounty::BountyState,
        expected: crate::bounty::BountyState,
    },

    #[error("nullifier already spent: {0:?}")]
    DoubleSpend(crate::types::Nullifier),

    #[error("ZK proof verification failed")]
    ProofInvalid,

    #[error("judge quorum not met: {attested}/{required}")]
    QuorumNotMet { attested: u32, required: u32 },

    #[error("judge attestation is fraudulent or inconsistent")]
    FraudulentAttestation,

    #[error("insufficient collateral: have {have}, need {need}")]
    InsufficientCollateral { have: u64, need: u64 },

    #[error("order not found: {0}")]
    OrderNotFound(uuid::Uuid),

    #[error("price-time mismatch: bid {bid} < ask {ask}")]
    NoMatch { bid: u64, ask: u64 },

    #[error("serialisation error: {0}")]
    Serialisation(#[from] bincode::error::EncodeError),

    #[error("deserialisation error: {0}")]
    Deserialisation(#[from] bincode::error::DecodeError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
