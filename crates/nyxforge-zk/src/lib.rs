//! ZK circuits for NyxForge's anonymous bond operations.
//!
//! All proofs use DarkFi's zkVM / Halo2 backend.  The circuits here describe
//! the *statements* to be proved; witness generation lives alongside each circuit.
//!
//! # Overview
//!
//! ```text
//!   MINT     — prove: I know a valid bond series + randomness → commitment
//!   TRANSFER — prove: I own note[old] and am creating note[new] with same value
//!   BURN     — prove: I own note + goal is met → nullifier (for redemption)
//! ```

pub mod burn;
pub mod mint;
pub mod note;
pub mod transfer;

pub use mint::{MintProof, MintWitness};
pub use transfer::{TransferProof, TransferWitness};
pub use burn::{BurnProof, BurnWitness};
pub use note::BondNote;

/// Errors arising from proof generation or verification.
#[derive(Debug, thiserror::Error)]
pub enum ZkError {
    #[error("proof generation failed: {0}")]
    ProvingFailed(String),

    #[error("proof verification failed")]
    VerificationFailed,

    #[error("invalid witness: {0}")]
    InvalidWitness(String),
}
