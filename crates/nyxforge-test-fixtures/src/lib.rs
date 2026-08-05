//! Shared test fixtures for NyxForge.
//!
//! Add to any crate's `[dev-dependencies]`:
//! ```toml
//! nyxforge-test-fixtures = { path = "../nyxforge-test-fixtures" }
//! ```
//!
//! # Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`bounties`] | Canonical [`Bounty`], [`GoalSpec`], [`JudgeSpec`] constructors |
//! | [`notes`] | [`BountyNote`] constructors; lazy commitment/nullifier statics |
//! | [`attestations`] | [`JudgeAttestation`] and [`QuorumResult`] constructors |
//! | [`proofs`] | [`MintProof`], [`TransferProof`], [`BurnProof`] constructors |
//! | [`mock_judge`] | [`MockDataSource`] with configurable return values |
//! | [`mock_rpc`] | [`MockRpcClient`] for testing CLI commands without a live node |

pub mod attestations;
pub mod bounties;
pub mod mock_judge;
pub mod mock_rpc;
pub mod notes;
pub mod proofs;

// Convenient flat re-exports for the most common fixtures.
pub use bounties::{
    active_bounty, draft_bounty, expired_bounty, homelessness_goal, minimal_bounty, minimal_goal,
    proposed_bounty, ORACLE_KEY_A, ORACLE_KEY_B, ORACLE_KEY_C, ISSUER_KEY,
};
pub use notes::{default_note, note_for_bounty, OWNER_KEY, OWNER_SECRET};
pub use mock_judge::fixed_source;
pub use mock_rpc::MockRpcClient;
