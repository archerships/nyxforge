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
//! | [`bonds`] | Canonical [`Bond`], [`GoalSpec`], [`OracleSpec`] constructors |
//! | [`notes`] | [`BondNote`] constructors; lazy commitment/nullifier statics |
//! | [`attestations`] | [`OracleAttestation`] and [`QuorumResult`] constructors |
//! | [`proofs`] | [`MintProof`], [`TransferProof`], [`BurnProof`] constructors |
//! | [`mock_oracle`] | [`MockDataSource`] with configurable return values |
//! | [`mock_rpc`] | [`MockRpcClient`] for testing CLI commands without a live node |

pub mod attestations;
pub mod bonds;
pub mod mock_oracle;
pub mod mock_rpc;
pub mod notes;
pub mod proofs;

// Convenient flat re-exports for the most common fixtures.
pub use bonds::{
    active_bond, draft_bond, expired_bond, homelessness_goal, minimal_bond, minimal_goal,
    proposed_bond, ORACLE_KEY_A, ORACLE_KEY_B, ORACLE_KEY_C, ISSUER_KEY,
};
pub use notes::{default_note, note_for_bond, OWNER_KEY, OWNER_SECRET};
pub use mock_oracle::fixed_source;
pub use mock_rpc::MockRpcClient;
