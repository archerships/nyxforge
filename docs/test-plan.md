# NyxForge — Test Plan

> Covers all test layers: unit, WASM integration, end-to-end, and performance.
> Written against the codebase as of March 2026 (Phase 0 prototype).

---

## Table of Contents

1. [Scope and Philosophy](#1-scope-and-philosophy)
2. [Test Infrastructure Setup](#2-test-infrastructure-setup)
3. [Layer 1 — Unit Tests](#3-layer-1--unit-tests)
4. [Layer 2 — WASM Integration Tests](#4-layer-2--wasm-integration-tests)
5. [Layer 3 — End-to-End Tests](#5-layer-3--end-to-end-tests)
6. [Layer 4 — Performance and Binary Audits](#6-layer-4--performance-and-binary-audits)
7. [CI Matrix](#7-ci-matrix)
8. [Coverage Targets](#8-coverage-targets)
9. [Known Gaps and Future Work](#9-known-gaps-and-future-work)

---

## 1. Scope and Philosophy

### 1.1 What we test

| Layer | Tool | When to run |
|-------|------|-------------|
| Unit (native Rust) | `cargo test` | Every commit |
| Unit (async) | `cargo test` + `tokio::test` | Every commit |
| WASM integration | `wasm-pack test --headless --chrome` | Every commit |
| End-to-end | Playwright | Pre-merge / nightly |
| Benchmarks | `cargo bench` (Criterion) | Weekly / on perf changes |
| Binary audit | `wasm-opt`, `twiggy`, `cargo bloat` | Pre-release |

### 1.2 Guiding principles

- **Test behaviour, not implementation.** Assert what a function does, not how
  it does it internally.  Refactoring must not require rewriting tests.
- **Determinism first.** Every test that exercises cryptographic or ZK code must
  use fixed seeds or known-good test vectors, never production randomness.
- **Mock at the boundary.** Tests below the RPC layer mock the network and
  the oracle data source; tests above it mock the node.
- **Fail fast on safety invariants.** The nullifier double-spend check, the
  scalar canonicality requirement, and the collateral conservation constraint
  each warrant dedicated negative tests that assert *failure* when violated.

---

## 2. Test Infrastructure Setup

### 2.1 Native unit tests — no changes required

`cargo test --workspace` already works.

### 2.2 Add `wasm-bindgen-test` to WASM crates

In `crates/nyxforge-web/Cargo.toml` and `crates/nyxforge-zk/Cargo.toml`:

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
```

In each test file that uses browser APIs, add at the top of the module:

```rust
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);
```

Run headlessly:

```bash
wasm-pack test crates/nyxforge-web --headless --chrome
wasm-pack test crates/nyxforge-zk  --headless --chrome
```

### 2.3 Playwright (E2E)

```bash
# One-time install (covered by scripts/install-playwright.sh)
npm init playwright@latest tests/e2e

# Run
npx playwright test
```

The Playwright config (`tests/e2e/playwright.config.ts`) should point at
`http://localhost:8080` and depend on the node being started in a `globalSetup`
script.

### 2.4 Criterion (benchmarks)

In any crate that will have benchmarks, add:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "zk_proofs"
harness = false
```

Run:

```bash
cargo bench -p nyxforge-zk
```

### 2.5 Workspace-level dev-dependencies

Add to the workspace root `Cargo.toml`:

```toml
[workspace.dependencies]
wasm-bindgen-test = "0.3"
criterion         = { version = "0.5", features = ["async_tokio"] }
```

---

## 3. Layer 1 — Unit Tests

All tests in this layer run with `cargo test`.  File paths are module paths
within each crate (`#[cfg(test)] mod tests { … }`).

---

### 3.1 `nyxforge-core`

#### `bond.rs`

| Test | What it verifies |
|------|-----------------|
| `bond_id_is_deterministic` | `Bond::compute_id` returns the same digest given identical inputs |
| `bond_id_differs_by_field` | Changing any field of `GoalSpec` or `issuer` changes the ID |
| `goal_spec_operator_lt` | `ComparisonOp::LessThan` evaluates `4 < 5 → true`, `5 < 5 → false` |
| `goal_spec_operator_lte` | `LessThanOrEqual` evaluates boundary correctly |
| `goal_spec_operator_gt` | `GreaterThan` evaluates boundary correctly |
| `goal_spec_operator_gte` | `GreaterThanOrEqual` evaluates boundary correctly |
| `goal_spec_operator_eq` | `Equal` passes only on exact match |
| `deadline_in_past_is_invalid` | `GoalSpec::validate()` rejects a deadline before `Utc::now()` |
| `empty_title_is_invalid` | `GoalSpec::validate()` rejects a blank title |
| `empty_data_id_is_invalid` | `GoalSpec::validate()` rejects a blank data_id |

#### `types.rs`

| Test | What it verifies |
|------|-----------------|
| `amount_from_whole_roundtrips` | `Amount::from_whole(100).to_whole() == 100` |
| `amount_addition_does_not_overflow` | Adding two `Amount` values yields the correct sum |
| `amount_subtraction_saturates_at_zero` | Subtracting more than available returns zero, not underflow |
| `amount_multiplication` | `Amount::from_whole(10) * 5 == Amount::from_whole(50)` |
| `collateral_amount` | `total_supply × redemption_value` computes correctly for representative values |

---

### 3.2 `nyxforge-wallet`

#### `keys.rs`

| Test | What it verifies |
|------|-----------------|
| `generated_spend_key_is_canonical` | `spend_bytes[31] & 0xf0 == 0` for a freshly generated key |
| `generated_view_key_is_canonical` | Same constraint for the view key |
| `spend_key_recovery_roundtrips` | `WalletKeys::from_spend_key(keys.xmr_spend_key_hex)` produces the same DRK pubkey |
| `drk_derivation_is_deterministic` | Calling `generate()` twice from the same spend key bytes yields identical DRK keys |
| `stagenet_address_has_correct_prefix` | Stagenet XMR addresses start with `5` |
| `invalid_spend_key_hex_is_rejected` | `from_spend_key("not-hex")` returns `Err` |
| `truncated_spend_key_is_rejected` | `from_spend_key("0102")` (< 64 chars) returns `Err` |
| `all_zero_spend_key_is_rejected` | `from_spend_key("00…00")` returns `Err` (zero scalar is invalid) |
| `serialise_deserialise_roundtrip` | `WalletKeys::from_serde(keys.to_serde())` is identity |

---

### 3.3 `nyxforge-zk`

#### `note.rs`

| Test | What it verifies |
|------|-----------------|
| `commitment_is_deterministic` *(exists)* | `note.commitment() == note.commitment()` |
| `different_owners_produce_different_commitments` | Two notes identical except `owner` produce distinct commitments |
| `different_quantities_produce_different_commitments` | Quantity is bound into the commitment |
| `different_bond_ids_produce_different_commitments` | Bond ID is bound into the commitment |
| `nullifier_is_deterministic` | `note.nullifier(sk) == note.nullifier(sk)` for same secret key |
| `nullifier_differs_by_serial` | Two notes with the same owner but different serials produce different nullifiers |
| `wrong_key_produces_different_nullifier` | `note.nullifier(sk1) != note.nullifier(sk2)` |

#### `circuits.rs` (wired — Phase 1 complete)

| Test | What it verifies |
|------|-----------------|
| `mint_proof_verifies` | A well-formed MINT proof passes the verifier |
| `mint_proof_with_wrong_commitment_fails` | Mutating the commitment field causes verification failure |
| `transfer_proof_verifies` | A well-formed TRANSFER proof passes |
| `transfer_proof_conservation_violated_fails` | `note_new.quantity != note_old.quantity` causes failure |
| `burn_proof_verifies` | A well-formed BURN proof passes |
| `burn_proof_wrong_quorum_hash_fails` | Mismatched `Q_hash` causes failure |
| `burn_proof_payout_address_not_linked_to_note_owner` | Providing a different `payout_address` than the note's `owner` still produces a valid BURN proof (payout_address is independent) |
| `burn_proof_different_payout_addresses_produce_different_commitments` | Same note, same randomness, different `payout_address` → different `payout_commitment` |

---

### 3.4 `nyxforge-contract`

#### `bond_market.rs`

| Test | What it verifies |
|------|-----------------|
| `issue_bond_transitions_draft_to_active` *(exists)* | State goes `Draft → Active` on valid issue |
| `issue_bond_rejects_wrong_state` | Issuing an already-`Active` bond returns `Err` |
| `issue_bond_checks_collateral_amount` | Issuing with insufficient collateral returns `Err` |
| `oracle_accept_all_transitions_to_draft` | All oracles accepting moves `PendingOracleApproval → Draft` |
| `single_rejection_blocks_draft` | One rejection keeps the bond in `PendingOracleApproval` |
| `revise_oracles_clears_responses` | After `revise_oracles`, all prior accept/reject signals are gone |
| `cannot_issue_before_all_oracles_accept` | Attempting to issue a `PendingOracleApproval` bond returns `Err` |
| `finalise_marks_redeemable_when_goal_met` | Quorum `true` attestations → `Active → Redeemable` |
| `finalise_marks_expired_when_goal_not_met` | Quorum `false` attestations after deadline → `Active → Expired` |
| `cannot_redeem_expired_bond` | Burning a note on an `Expired` bond returns `Err` |
| `bond_id_mismatch_in_transfer_rejected` | A TRANSFER proof for wrong `bond_id` returns `Err` |
| `nullifier_double_spend_rejected` | Submitting the same nullifier twice returns `Err` on the second |

---

### 3.4b `nyxforge-core` — CollateralSpec (planned)

| Test | What it verifies |
|------|-----------------|
| `collateral_spec_drk_amount_matches_bond` | `CollateralSpec { currency: Drk, amount }` validates against `total_supply × redemption_value` |
| `collateral_spec_xmr_requires_dleq_mechanism` | XMR currency with `LockMechanism::DirectCustody` is rejected |
| `collateral_spec_unknown_currency_rejected` | Unknown currency ID returns validation error |
| `chain_address_drk_parses_pubkey` | `ChainAddress::Drk(hex)` parses a valid 32-byte hex pubkey |
| `chain_address_xmr_parses_stealth_address` | `ChainAddress::Xmr(addr)` parses a valid XMR stealth address |
| `chain_address_zano_parses_stealth_address` | `ChainAddress::Zano(addr)` parses a valid Zano stealth address |
| `chain_address_ar_parses_arweave_address` | `ChainAddress::Ar(addr)` parses a valid Arweave wallet address |
| `chain_address_ao_parses_ao_process_id` | `ChainAddress::Ao(id)` parses a valid AO Process ID |
| `chain_address_wrong_currency_rejected` | Using a BTC address as an XMR ChainAddress returns error |

---

### 3.5 `nyxforge-oracle`

#### `verifier.rs`

| Test | What it verifies |
|------|-----------------|
| `mock_source_returns_configured_value` *(exists)* | `MockDataSource.fetch` returns preset value |
| `mock_source_supports_correct_data_id` | `supports()` true/false for right/wrong data ID |
| `evaluate_goal_lt_passes` | `fetch() = 49_000`, threshold `50_000`, op `LessThan` → `true` |
| `evaluate_goal_lt_fails_at_boundary` | `fetch() = 50_000`, op `LessThan` → `false` |
| `evaluate_goal_lte_passes_at_boundary` | `fetch() = 50_000`, op `LessThanOrEqual` → `true` |
| `evaluate_goal_gt_passes` | `fetch() = 51_000`, op `GreaterThan`, threshold `50_000` → `true` |
| `evaluate_goal_eq_passes` | `fetch() = 42`, op `Equal`, threshold `42` → `true` |
| `evaluate_goal_eq_fails_off_by_one` | `fetch() = 43`, op `Equal`, threshold `42` → `false` |
| `attestation_signature_verifies` | Oracle signs an attestation; verifier accepts it with correct public key |
| `attestation_wrong_key_rejected` | Verification with a different public key returns `Err` |

---

### 3.5b Oracle — DLC/DLEQ attestation model (planned)

| Test | What it verifies |
|------|-----------------|
| `oracle_cannot_publish_both_scalars` | Publishing `s_met` and `s_fail` from the same oracle key is detected as fraud |
| `s_met_scalar_unlocks_payout_output` | DLEQ proof with `s_met` verifies against the pre-committed payout output public key |
| `s_fail_scalar_unlocks_refund_output` | DLEQ proof with `s_fail` verifies against the pre-committed refund output public key |
| `s_met_does_not_unlock_refund_output` | `s_met` scalar fails verification against the refund output key |
| `timelock_fallback_bypasses_oracle` | After deadline, `return_address` holder can sweep collateral without any oracle scalar |
| `two_phase_claim_requires_burn_proof` | Intermediate output cannot be swept without a valid BURN proof |
| `quorum_result_s_met_requires_threshold` | `s_met` is only published when `attestation_threshold` oracles have agreed |

---

### 3.6 `nyxforge-miner`

#### `hasher.rs`

| Test | What it verifies |
|------|-----------------|
| `target_check_passes_easy_target` *(exists)* | All-zero hash beats `ffffffff` target |
| `target_check_fails_hard_target` | All-zero hash does NOT beat `0000000000000000…` target |
| `target_check_boundary` | Hash `0000ffff…` beats target `0001…`; fails target `0000ff00…` |

#### `p2pool.rs`

| Test | What it verifies |
|------|-----------------|
| `stratum_subscribe_serialises` | `StratumMsg::Subscribe` encodes to valid JSON with method and params |
| `stratum_notify_deserialises` | A sample `mining.notify` JSON line deserialises into the correct struct |
| `stratum_submit_serialises` | `StratumMsg::Submit` contains nonce, job_id, and extra_nonce2 |
| `stratum_difficulty_deserialises` | `mining.set_difficulty` JSON line deserialises correctly |

---

### 3.7 `nyxforge-mcp`

#### `config.rs`

| Test | What it verifies |
|------|-----------------|
| `default_provider_returns_active_entry` | `active_provider()` returns the entry named in `default_provider` |
| `missing_default_returns_error` | `active_provider()` when `providers` is empty returns descriptive `Err` |
| `effective_model_anthropic` | Anthropic entry without explicit model returns `claude-opus-4-6` |
| `effective_model_openai` | OpenAI entry without explicit model returns `gpt-4o` |
| `effective_base_url_ollama` | Ollama entry without explicit URL returns `http://localhost:11434` |
| `config_roundtrips_via_json` | Serialise → deserialise → `assert_eq` |
| `add_and_remove_provider` | Add a provider; verify it appears; remove it; verify it's gone |

---

### 3.8 `nyxforge-node` (RPC unit tests with mocked state)

#### `rpc.rs`

These tests construct a `NodeState` in-memory (no P2P, no disk) and call the
RPC handlers directly.

| Test | What it verifies |
|------|-----------------|
| `bonds_list_empty` | `bonds.list` returns `[]` on fresh state |
| `bonds_issue_and_list` | Issue a bond; `bonds.list` returns it |
| `bonds_get_by_id` | Issue a bond; `bonds.get` returns the same bond |
| `bonds_get_unknown_id_returns_error` | `bonds.get` with a random hex ID returns JSON-RPC error |
| `wallet_create_returns_addresses` | `wallet.create` returns `xmr_address`, `drk_address`, `xmr_spend_key` |
| `wallet_import_roundtrips` | Create wallet; import its spend key; DRK address matches |
| `wallet_create_twice_returns_error` | Second `wallet.create` call returns `Err("wallet already exists")` |
| `invalid_method_returns_error` | Calling `no_such_method` returns JSON-RPC `-32601` |
| `malformed_json_returns_parse_error` | Sending `{garbage}` returns JSON-RPC `-32700` |

---

## 4. Layer 2 — WASM Integration Tests

All tests in this layer require a Chrome / Chromium binary.

```bash
wasm-pack test crates/nyxforge-web --headless --chrome
wasm-pack test crates/nyxforge-zk  --headless --chrome
```

### 4.1 `nyxforge-web` — browser initialisation

```rust
// crates/nyxforge-web/src/lib.rs
#[wasm_bindgen_test]
async fn init_does_not_panic() {
    nyxforge_web::init().await;   // must not throw
}

#[wasm_bindgen_test]
fn note_commitment_matches_native() {
    // pre-computed from native unit test
    let note = test_fixtures::dummy_note();
    let expected = hex!("…");
    assert_eq!(note.commitment(), expected);
}
```

| Test | What it verifies |
|------|-----------------|
| `init_does_not_panic` | `init()` completes without a JS exception |
| `note_commitment_matches_native` | WASM commitment output matches the native `#[test]` fixture |
| `nullifier_matches_native` | WASM nullifier output matches the native fixture |
| `wallet_keys_stored_in_local_storage` | After key gen, `localStorage` contains the expected JSON keys |
| `wallet_keys_retrieved_from_local_storage` | Keys written then read back are bit-for-bit equal |
| `rpc_client_fails_gracefully_without_node` | `RpcClient::call("bonds.list")` against a closed port returns a typed error, not a JS panic |

### 4.2 `nyxforge-zk` — proof generation in WASM

These are stubs until the Halo2 circuits are wired; mark them `#[ignore]` and
un-ignore as each circuit is implemented.

| Test | What it verifies |
|------|-----------------|
| `mint_proof_round_trip_wasm` *(ignored)* | `prove_mint` + `verify_mint` succeeds in WASM |
| `transfer_proof_round_trip_wasm` *(ignored)* | `prove_transfer` + `verify_transfer` succeeds |
| `burn_proof_round_trip_wasm` *(ignored)* | `prove_burn` + `verify_burn` succeeds |
| `proof_generation_completes_under_30s_wasm` | `performance.now()` delta < 30 000 ms per proof type |

---

## 5. Layer 3 — End-to-End Tests

All Playwright tests live under `tests/e2e/`.  A `globalSetup` script starts
the node (`./nyxforge --start`) and waits for `http://localhost:8888/health`
before any test runs.  A `globalTeardown` script calls `./nyxforge --stop`.

Run all:

```bash
npx playwright test
```

Run one suite:

```bash
npx playwright test tests/e2e/bond-lifecycle.spec.ts
```

---

### 5.1 Application bootstrap

**File:** `tests/e2e/bootstrap.spec.ts`

| Test | Steps | Expected |
|------|-------|----------|
| WASM module loads | Navigate to `localhost:8080` | No console errors; page title "NyxForge" visible |
| Node health endpoint | `GET localhost:8888/health` | 200 OK |
| RPC responds | `POST localhost:8888/rpc` `{"method":"bonds.list"}` | `{"result": []}` (empty list) |
| WASM hydrates | Wait for `.bond-browser` selector | Element present within 10 s |

---

### 5.2 Wallet

**File:** `tests/e2e/wallet.spec.ts`

| Test | Steps | Expected |
|------|-------|----------|
| Create wallet via CLI | `nyxforge-cli wallet create` | Output contains `xmr:` and `drk:` addresses; no errors |
| Addresses are stable | Run `nyxforge-cli wallet addresses` | Same addresses printed |
| Import spend key | Copy spend key from create output; `nyxforge --wallet-key <hex>` restart | Node starts; wallet has same DRK address |
| Invalid spend key rejected | `nyxforge --wallet-key deadbeef` (too short) | Process exits non-zero with error message |

---

### 5.3 Bond — create path

**File:** `tests/e2e/bond-create.spec.ts`

This suite drives the CLI interactively using `node-pty` / expect-style
input, as the bond wizard uses `dialoguer`.

| Test | Steps | Expected |
|------|-------|----------|
| Bond create completes | Provide all wizard fields with valid values | `Bond issued: <64-char hex ID>` printed |
| Bond appears in list | `nyxforge-cli bond list` after create | Table row with correct title and `Active` state |
| Bond get returns JSON | `nyxforge-cli bond get <ID>` | Valid JSON with matching `id`, `goal.title`, `state` |
| Wizard cancelled | Ctrl-C after title prompt | No bond created; `bond list` unchanged |
| Below-floor price rejected | Enter `floor_price > redemption_value` | Wizard prints validation error; wizard re-prompts |
| Past deadline rejected | Enter deadline `2000-01-01` | Wizard prints validation error |
| Invalid data_id in prod mode | Enter `bogus.data.id` | Wizard prints "no oracle supports this data ID" |

---

### 5.4 Bond — proposal workflow

**File:** `tests/e2e/bond-proposal.spec.ts`

| Test | Steps | Expected |
|------|-------|----------|
| Propose creates Proposed bond | `nyxforge-cli bond propose` → complete wizard | State = `Proposed` |
| Comment appears | `bond comment <ID>` → enter text; `bond comments <ID>` | Text appears in output |
| Multiple comments ordered | Post two comments; `bond comments` | Chronological order |
| Submit advances state | `bond submit <ID>` | State = `PendingOracleApproval` |
| Cannot submit twice | `bond submit <ID>` again | Error: already in `PendingOracleApproval` |

---

### 5.5 Bond — oracle approval workflow

**File:** `tests/e2e/bond-oracle.spec.ts`

These tests use the node's `--allow-unverifiable` flag and a single-oracle
bond (wallet's own DRK key) for speed.

| Test | Steps | Expected |
|------|-------|----------|
| Oracle accept advances to Draft | `bond oracle-accept <ID>` | State = `Draft` |
| Oracle reject blocks Draft | `bond oracle-reject <ID>` | State stays `PendingOracleApproval`; rejection reason recorded |
| Status shows accept/reject | `bond oracle-status <ID>` after reject | Row shows `✘` and reason text |
| Revise oracles clears responses | `bond revise-oracles <ID>` with new key list | All prior responses cleared; state back in `PendingOracleApproval` |
| Issue after Draft transitions to Active | `bond issue <ID>` after all accept | State = `Active` |
| Issue before Draft is rejected | `bond issue <ID>` from `PendingOracleApproval` | Error: not yet approved |

---

### 5.6 Bond — settlement path

**File:** `tests/e2e/bond-settlement.spec.ts`

These tests inject a mock oracle attestation directly via RPC to avoid
real-world data dependencies.

| Test | Steps | Expected |
|------|-------|----------|
| Goal met → Redeemable | Post `attestation_threshold` `goal_met: true` attestations | Bond state = `Redeemable` |
| Goal not met → Expired | Post quorum `goal_met: false` attestations after deadline | Bond state = `Expired` |
| Redeem via CLI | `bond redeem <ID>` on `Redeemable` bond | Payout note created; DRK balance increases |
| Cannot redeem Expired | `bond redeem <ID>` on `Expired` bond | Error: bond expired |
| Issuer reclaims on Expired | `bond reclaim <ID>` on `Expired` | Collateral returned; bond state = `Settled` |
| Double-spend rejected | Submit same BurnProof twice | Second submission returns nullifier-already-spent error |
| Two-phase claim completes | Post `s_met`; holder generates BURN proof with `payout_address`; sweep intermediate | Payout delivered to `payout_address` |
| Wrong `payout_address` in burn proof rejected | Post valid BURN proof but with mismatched `payout_commitment` | Proof verification fails |
| Issuer cannot claim payout output | Attempt to sweep payout output using `return_address` key | Script/DLEQ rejects — wrong key |

---

### 5.7 MCP / AI assistance

**File:** `tests/e2e/mcp.spec.ts`

These tests use Ollama with a small local model to avoid API key requirements
in CI.  Skip with `--grep-invert mcp` if Ollama is not available.

| Test | Steps | Expected |
|------|-------|----------|
| MCP health check | `nyxforge-cli mcp status` | `nyxforge-mcp running` message |
| Add Ollama provider | `nyxforge-cli mcp add test-ollama` → choose Ollama | Provider appears in `mcp providers` list |
| Bond explore returns suggestions | `nyxforge-cli bond explore` → enter description | Similar bonds table and AI draft printed |
| Bond explore prefills wizard | Choose "Create bond from AI draft" | Wizard opens with pre-filled fields |
| Remove provider | `nyxforge-cli mcp remove test-ollama` | Provider absent from `mcp providers` |
| MCP not running gives clear error | Stop `nyxforge-mcp`; `nyxforge-cli bond explore` | Error: "Is it running? Start with: nyxforge-mcp" |

---

### 5.8 Mining

**File:** `tests/e2e/mining.spec.ts`

These tests run against a mock Stratum server (`tests/fixtures/mock-stratum-server.js`)
to avoid requiring a real P2Pool instance.

| Test | Steps | Expected |
|------|-------|----------|
| Miner starts via RPC | `miner.start { threads: 1 }` RPC | `miner.status` returns `running: true` |
| Miner stops via RPC | `miner.stop` after start | `miner.status` returns `running: false` |
| Miner connects to Stratum | Start mock server; start miner | Mock server receives `mining.subscribe` within 5 s |
| `--mine-on-start` flag | Restart node with flag | `miner.status` returns `running: true` immediately |
| Thread count respected | `miner.start { threads: 4 }` | Status shows `threads: 4` |

---

### 5.9 Error handling

**File:** `tests/e2e/error-handling.spec.ts`

| Test | Steps | Expected |
|------|-------|----------|
| Node not running | CLI command without started node | Descriptive error + suggestion to run `./nyxforge --start` |
| No wallet | `bond create` without `wallet.create` | Error: "No wallet found. Create one first…" |
| Unknown bond ID | `bond get aaaa…` (valid hex, absent) | Error: "bond not found" |
| Invalid bond ID format | `bond get not-hex` | Error: "invalid bond ID: expected 32-byte hex" |
| Malformed RPC | POST `{garbage}` to RPC port | JSON-RPC `-32700` parse error response |
| MCP provider API key wrong | Configure Anthropic with bad key; run `bond explore` | Error from upstream provider, not a panic |

---

## 6. Layer 4 — Performance and Binary Audits

### 6.1 ZK proof benchmarks

**File:** `crates/nyxforge-zk/benches/zk_proofs.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_mint(c: &mut Criterion) {
    let note = test_fixtures::dummy_note();
    c.bench_function("mint_prove", |b| b.iter(|| prove_mint(&note)));
}
```

| Benchmark | Target (laptop, single thread) |
|-----------|-------------------------------|
| `mint_prove` | < 10 s |
| `mint_verify` | < 1 s |
| `transfer_prove` | < 15 s |
| `transfer_verify` | < 1 s |
| `burn_prove` | < 15 s |
| `burn_verify` | < 1 s |

Run: `cargo bench -p nyxforge-zk`

These targets are advisory until the Halo2 circuits are wired.  Revisit once
real proof sizes are known.

### 6.2 WASM binary size

The `nyxforge-web` WASM module must load quickly in a browser.

| Metric | Target |
|--------|--------|
| Raw `.wasm` after `wasm-pack build` | < 5 MB |
| After `wasm-opt -Oz` | < 2 MB |
| Gzipped transfer size | < 700 KB |

Check:

```bash
wasm-pack build crates/nyxforge-web --release
ls -lh crates/nyxforge-web/pkg/*.wasm

wasm-opt -Oz crates/nyxforge-web/pkg/nyxforge_web_bg.wasm \
    -o /tmp/nyxforge_web_bg.opt.wasm
ls -lh /tmp/nyxforge_web_bg.opt.wasm

# Find what's contributing to binary size
twiggy top /tmp/nyxforge_web_bg.opt.wasm
```

Run in CI with:

```bash
SIZE=$(stat -f%z /tmp/nyxforge_web_bg.opt.wasm)
[ "$SIZE" -lt 2097152 ] || (echo "WASM binary too large: $SIZE bytes" && exit 1)
```

### 6.3 Memory profiling

Run the browser UI under Chrome DevTools memory snapshots:

1. Start the app (`./nyxforge --start`).
2. Open Chrome DevTools → Memory → Heap Snapshot → take baseline.
3. Create 10 bonds via the UI.
4. Force GC.
5. Take second snapshot.
6. Compare: retained WASM memory must not grow unboundedly.

Known areas to watch:
- Bond note list (unbounded append without pagination)
- P2P gossip message buffer
- RPC response deserialization (large bond lists)

### 6.4 P2P gossip throughput

**Script:** `tests/perf/gossip_flood.sh`

Start three local nodes on loopback, flood 1,000 bond announcements from node A,
and verify all 1,000 appear on node C within 30 seconds.

```bash
./tests/perf/gossip_flood.sh --nodes 3 --messages 1000 --timeout 30
```

This is a smoke test, not a precise benchmark.  Failure indicates a regression
in the gossip pipeline.

### 6.5 Miner hashrate floor

Start the miner with 1 thread against the mock Stratum server.  Assert that
`miner.status.hashrate_hps > 100` after a 10-second warm-up.  This catches
accidental removal of the inner hash loop.

---

## 7. CI Matrix

```
┌─────────────────────────────────────────────────────────────────────┐
│  trigger: push / pull_request                                       │
│                                                                     │
│  job: unit-tests                                                    │
│    runs-on: ubuntu-latest, macos-latest                             │
│    steps:                                                           │
│      cargo test --workspace                                         │
│                                                                     │
│  job: wasm-tests                                                    │
│    runs-on: ubuntu-latest                                           │
│    steps:                                                           │
│      install wasm-pack, chromium                                    │
│      wasm-pack test crates/nyxforge-web --headless --chrome         │
│      wasm-pack test crates/nyxforge-zk  --headless --chrome         │
│                                                                     │
│  job: e2e-tests                                                     │
│    runs-on: ubuntu-latest                                           │
│    steps:                                                           │
│      cargo build --workspace                                        │
│      scripts/install-playwright.sh                                  │
│      ./nyxforge --start &                                           │
│      npx playwright test --reporter=line                            │
│      ./nyxforge --stop                                              │
│                                                                     │
│  job: binary-audit  (on: push to master)                            │
│    runs-on: ubuntu-latest                                           │
│    steps:                                                           │
│      wasm-pack build crates/nyxforge-web --release                  │
│      wasm-opt -Oz … → check size < 2 MB                            │
└─────────────────────────────────────────────────────────────────────┘
```

Jobs run in parallel; `e2e-tests` depends on `unit-tests`.

---

## 8. Coverage Targets

| Crate | Statement coverage target |
|-------|--------------------------|
| `nyxforge-core` | ≥ 90% |
| `nyxforge-wallet` | ≥ 90% |
| `nyxforge-zk` | ≥ 80% |
| `nyxforge-contract` | ≥ 85% |
| `nyxforge-oracle` | ≥ 85% |
| `nyxforge-miner` | ≥ 75% |
| `nyxforge-mcp` | ≥ 80% |
| `nyxforge-node` (RPC, state) | ≥ 75% |
| `nyxforge-cli` (commands) | ≥ 70% |
| `nyxforge-web` (WASM) | ≥ 60% |

Measure with `cargo llvm-cov --workspace --html`.

---

## 9. Known Gaps and Future Work

| Gap | Priority | Notes |
|-----|----------|-------|
| ZK circuit proof tests are `#[ignore]` | High | Un-ignore as Halo2 wiring is completed (Phase 1) |
| No integration test for P2P gossip across real nodes | High | Requires multi-process test harness |
| No test for oracle slashing | Medium | Requires contract-level slash logic implementation |
| No test for DAO override path | Medium | `dao_override_allowed = true` path untested |
| No fuzz testing on RPC deserialization | Medium | Add `cargo-fuzz` targets for JSON-RPC input parsing |
| No test for `ClaimExpiredCollateral` | Medium | Implement after contract expiry path is wired |
| E2E mining tests require mock Stratum server | Low | Script to be written (`tests/fixtures/mock-stratum-server.js`) |
| No test for wallet passphrase encryption | Low | Deferred until at-rest encryption is implemented |
| Property-based tests for `GoalMetric` evaluation | Low | Add `proptest` to `nyxforge-core` dev-dependencies |
| Browser memory profiling is manual | Low | Automate with Playwright `page.evaluate` CDP hooks |
| CollateralSpec / CollateralManager / ChainAddress types not yet implemented | High | Phase 5 — XMR first |
| No test for DLEQ adaptor signature construction (XMR collateral) | High | Phase 5 — requires xmr-btc-swap DLEQ library |
| No test for oracle DLEQ pre-setup (two pre-committed XMR outputs) | High | Phase 5 — oracle panel nonce commitment protocol |
| No test for two-phase XMR claim (s_met → intermediate → BURN sweep) | High | Phase 5 — requires CollateralManager |
| No test for XMR timelock fallback (refund without oracle) | High | Phase 5 — requires CollateralManager |
| payout_address / return_address types are [u8; 32] — must become ChainAddress | Medium | Phase 5 — first variant is XMR stealth address |
| No test for DLC construction (BTC/ZEC collateral) | Low | Phase 6 — deferred |
| No test for EVM escrow contract (ETH collateral) | Low | Phase 6 — deferred |
| No test for Warp/SmartWeave escrow contract (AR collateral) | Low | Phase 6 — deferred |
| No test for AO Process actor escrow (AO collateral) | Low | Phase 6 — deferred |
| No test for Zano DLEQ collateral | Low | Phase 6 — deferred (same mechanism as XMR) |
