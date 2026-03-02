use thiserror::Error;

#[derive(Debug, Error)]
pub enum NyxError {
    #[error("bond not found: {0:?}")]
    BondNotFound(crate::types::Digest),

    #[error("bond is in state {current:?}, expected {expected:?}")]
    InvalidBondState {
        current:  crate::bond::BondState,
        expected: crate::bond::BondState,
    },

    #[error("nullifier already spent: {0:?}")]
    DoubleSpend(crate::types::Nullifier),

    #[error("ZK proof verification failed")]
    ProofInvalid,

    #[error("oracle quorum not met: {attested}/{required}")]
    QuorumNotMet { attested: u32, required: u32 },

    #[error("oracle attestation is fraudulent or inconsistent")]
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
