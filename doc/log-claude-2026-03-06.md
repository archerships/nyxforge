# NyxForge session log — 2026-03-06

## Session summary

Continued from 2026-03-04 session. Context was compacted before this session began.

## Completed: Multi-goal bonds (goal → goals)

Changed `Bond.goal: GoalSpec` → `Bond.goals: Vec<GoalSpec>` across the entire
codebase. AND semantics: all goals must be met for payout.

Files changed:
- `crates/nyxforge-core/src/bond.rs` — struct field + `compute_id` signature
- `crates/nyxforge-test-fixtures/src/bonds.rs` — split `lifebond_goal()` into `lifebond_alive_goal()` + `lifebond_health_goal()`
- `crates/nyxforge-contract/src/bond_market.rs` — empty-goals validation
- `crates/nyxforge-node/src/rpc.rs` — `bonds.propose` + `bonds.issue` handlers
- `crates/nyxforge-oracle/src/oracle.rs` — `evaluate()` loop over all goals
- `crates/nyxforge-cli/src/commands/bond.rs` — multi-criterion wizard loop
- `ui/lib/src/node_client.dart` — `BondSummary.goals: List<GoalInfo>`
- `ui/lib/src/bond_market_screen.dart` — expanded detail shows each criterion
- `ui/lib/src/issue_bond_screen.dart` — `_CriterionState` helper, "Add criterion +" button
- `scripts/seed-demo-bonds.py` — `"goals": [...]` arrays

Result: 202+ Rust tests pass, flutter analyze 0 issues.

## AO/privacy design discussion

Read `docs/ao-strategy.md` and `docs/privacy-design.md`. Discussed three-layer
architecture for bond storage on Arweave AO (permanent, 200+ year SLA).

## Completed: Privacy design docs + oracle push model

Updated docs and implemented oracle push model:

- `docs/privacy-design.md` v0.2 — Added §5 "Goal Text Privacy & View Keys"
  (ChaCha20-Poly1305 encryption, `bond_view_key = blake3(spend_key ‖ bond_id)`)
  and §6 "Oracle Attestation Privacy — Noise Bonds" (constant-rate dummy
  attestations as anonymity set)
- `docs/ao-strategy.md` — Added §2.4 "Oracle Attestation Model — Push, Not Poll"
  with rationale table and ASCII flow diagram
- `docs/roadmap.md` — Added Phase 4.5 "Privacy Hardening" (goal text encryption,
  oracle push model, noise bonds); updated dependency map + milestone table
- `crates/nyxforge-oracle/src/oracle.rs` — Implemented `OracleNode::monitor_bonds()`:
  push model, spawns one task per Active bond, fires attestation when ALL goals
  met, returns `Vec<JoinHandle>` for cancellation. 7 new tests, all passing.
- `crates/nyxforge-oracle/Cargo.toml` — Added `nyxforge-test-fixtures` dev-dep

Total: 212 tests pass workspace-wide (6 ignored for slow Halo2 proof roundtrips).

## Design discussion: alternatives to noise bonds

User noted noise bonds are a kludge. Discussed three alternatives:

- **Option A (chosen):** ZK-proven oracle attestations — oracle signatures become
  private inputs to the BURN Halo2 circuit. Oracle attestations never appear on
  the AO ledger at all. Tradeoff: requires bondholder to present evidence of
  oracle fraud for slashing.
- **Option B:** Threshold MPC attestations — off-chain MPC computes combined sig,
  submitted by anonymous relay. Still on-chain but detached from oracle identity.
- **Option C:** Commit-reveal with VDF delay — decouples observation time from
  publication; too coarse for fine-grained events.

## In progress: Option A implementation

Implementing ZK oracle attestations (oracle sigs as private BURN circuit inputs).
See plan below.
