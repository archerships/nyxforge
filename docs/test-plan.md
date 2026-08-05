# NyxForge Test Plan

Status: current MVP test specification as of 2026-04-25.

This document defines the test contract for the active NyxForge MVP. The
current build target is `doc/00_MVP.md`: bearer `.bounty` files, bounty/judge
terminology, plugin-aware UI/CLI surfaces, a Flutter frontend with Rust/WASM
backend logic, multi-currency collateral metadata, and eventual XMR stagenet
redemption.

Older v2 work on ZK notes, P2P networking, browser proof generation, and full
DarkFi-era assumptions is preserved in the codebase but is not the active MVP
test target unless explicitly called out as deferred.

## 1. Test Philosophy

- Test user-visible behavior and protocol invariants, not private
  implementation details.
- Keep the mock API, CLI examples, generated mockups, and `.bounty` schema in
  sync with `doc/00_MVP.md`.
- Use deterministic fixtures for cryptographic, address, and proof-like data.
- Mock at process and RPC boundaries until the real backend replaces stubs.
- Run the smallest test layer that can catch the likely failure, then broaden
  when a shared interface changes.
- Every code change should include or update a relevant test unless the change
  is documentation-only.

## 2. Current Test Layers

| Layer | Tool | Scope | When to run |
| :--- | :--- | :--- | :--- |
| Mechanical consistency | `python3 bin/consistency-check` | Spec, mock data, CLI command shape | Before commits touching spec/mock/CLI |
| CLI behavior | `pytest test/test_*.py` or `python -m unittest` | `nyx` command behavior and JSON output | Every CLI/mock change |
| Mockup/generator checks | shell + `rg` | `bin/` generators and generated HTML | Every UI mockup change |
| Rust crate tests | `cargo test -p <crate>` | Real Rust implementation units | Every Rust code change |
| Rust workspace tests | `cargo test --workspace` | Shared Rust interfaces | Before broad refactors/merge |
| Flutter/Rust-WASM app smoke tests | `flutter test`, browser runner, Playwright | Static app bundle against mock state | Every user-facing app change |
| Flutter/widget tests | `flutter test` / Widgetbook | Optional shell/mobile wrapper UI | When wrapper code changes |
| E2E mock API | Mockoon + CLI + Flutter/Rust-WASM app | Full lifecycle against fake node | Pre-merge when flows change |
| E2E stagenet | Monero stagenet + real backend | Full bounty collateral lifecycle | Phase 5 gate |
| Performance/audit | Criterion, size tools | Hot cryptographic/UI paths | Pre-release or perf changes |

## 3. Canonical Commands

Run from `~/av/prj/nyxforge` unless noted.

```sh
python3 bin/consistency-check
```

```sh
pytest test
```

If `pytest` is unavailable, the suite uses `unittest` style tests:

```sh
python3 -m unittest discover -s test -p 'test_*.py'
```

Rust tests run from `src/`:

```sh
cd src
cargo test -p nyxforge-core
cargo test -p nyxforge-wallet
cargo test -p nyxforge-contract
cargo test --workspace
```

Mockup generator checks use the changed generator directly, for example:

```sh
bin/bounty-market > doc/04_STORYBOARDS/out/bounty-market-mockup.html
rg -n "oracle|bond|--bond|\\.bond|bounty-wallet|oracle-registry" doc/04_STORYBOARDS/out bin
```

## 4. Mechanical Consistency Check

`bin/consistency-check` validates structural invariants between:

- `mock/gen_bonds.js`
- `mock/nyxforge.mockoon.json`
- `src/crates/nyxforge-cli/src/commands/bond.rs`
- `doc/00_MVP.md`

Current checks include:

- every generated lifecycle state appears in Mockoon list/get responses
- Mockoon examples include required fields such as `serial`, `deadline`,
  `expiry`, `grace_days`, `amount`, `currency`, `lock_mechanism`, `quorum`,
  and `terms`
- generated mock data emits required fields such as `holder_pubkey` and
  `alg_epoch`
- proof/mock endpoints exist where expected
- CLI command variants include expected lifecycle commands
- stale MVP field names such as `unit_serial`, `collateral_xmr`, and `fee_xmr`
  do not reappear

Known naming caveat: the checker still references some legacy internal paths and
method names such as `bond.rs` and `bonds.*`. That reflects deferred code/RPC
renaming noted in `doc/00_MVP.md`; user-facing CLI flags should still use
`--bounty`.

## 5. Active Python CLI Test Suite

The active executable suite lives under `test/`. It exercises the current CLI
surface through the `nyx` binary. Set `NYX_BIN` to test a non-default binary.

Shared helper: `test/conftest.py`.

### 5.1 Bounty lifecycle

File: `test/test_bounty.py`

Must verify:

- `nyx bounty create` produces a SQLite `.bounty` file
- created files have a SQLite header
- subject, collateral, maturity/deadline, and 10-year hard cap validations fire
- JSON output includes the bounty identifier
- inspect output shows subject, collateral, and valid DLEQ status
- inspect JSON includes canonical keys such as `bond_id`/current internal ID,
  `subject`, `maturity`, `collateral`, `currency`, and `state`
- verify passes on a fresh file and fails on tampering
- list shows created bounty files and handles empty directories
- transfer changes the bearer key

### 5.2 Claim and evidence flow

File: `test/test_claim.py`

Must verify:

- claim filing validates bounty/claim identifiers and claim type
- valid claim types are accepted
- evidence attachment validates claim ID and file existence
- evidence metadata exposes hashes/statuses
- claim status and list JSON are parseable
- withdrawal validates claim state and supports dry run

Evidence policy: tests should cover both embedded evidence and external
references (`ipfs://`, `ar://`, `https://`) with stored SHA-256 hashes as the
schema matures.

### 5.3 Judge workflow

File: `test/test_judge.py`

Must verify:

- judges can list claims and filter by status
- review exposes evidence items and hashes
- decide validates claim ID, verdict flag, and verdict value
- approve/deny paths return a verdict transaction or equivalent receipt
- appeal rejects invalid or undecided claims
- judge registration returns a judge key and supports dry run
- judge history returns parseable JSON

Legacy note: `test/test_oracle.py` still covers old oracle registry commands.
It should be renamed or replaced as part of the deferred code/RPC terminology
cleanup.

### 5.4 Key management

File: `test/test_key.py`

Must verify:

- key generation requires label and passphrase
- `.nfkey`/keystore files are created through the canonical storage path
- JSON output includes fingerprint
- generated keys use `ed25519` and `alg_epoch = 0`
- weak passphrases warn
- list can filter by usage
- import/export validate supported formats
- rotation requires old fingerprint, supports dry run, and creates a new key

### 5.5 Wallet, NYX, TARI, and mining controls

File: `test/test_wallet.py`

Must verify:

- balances are non-negative and filterable by coin
- receive works for XMR, NYX, and TARI
- unknown coins are rejected
- send validates destination, amount, negative/zero values, and insufficient
  balance
- dry-run send returns fee information
- history is filterable and bounded by limit
- mining controls validate coin list, status, dry run, and stop behavior

NYX and TARI are MVP scope for testing, reward accounting, and early supporter
flows. Tests should not treat them as v2-only.

### 5.6 Market, DEX, auction, swap

Files:

- `test/test_dex.py`
- `test/test_auction.py`
- `test/test_swap.py`

Must verify:

- market/DEX pairs include NYX/XMR where relevant
- list endpoints return parseable arrays with required keys
- create/take/bid paths reject missing, zero, below-minimum, or invalid values
- dry-run commands return previews without mutating state
- cancellation, settlement, dispute, and history commands validate ownership and
  state transitions

These surfaces may be mock or preview implementations during MVP, but their CLI
contracts should remain stable once shown in UI mockups.

### 5.7 NGO, tax, node, config

Files:

- `test/test_ngo.py`
- `test/test_tax.py`
- `test/test_node.py`
- `test/test_config.py`

Must verify:

- NGO donation, conversion, treasury, receipt, and config commands validate
  inputs and produce expected files/JSON
- tax status, generation, receipts, exports, and ledger commands validate year,
  form, and output archive contents
- node status, peers, sync, and health return bounded, parseable values
- config get/set/reset/profile commands reject invalid network, URL, log level,
  and unknown profile/key operations

## 6. Mock API and Fixtures

Mock API work is the Phase 2 binding contract for the real backend.

Required lifecycle states:

- `DRAFT`
- `ACTIVE`
- `REDEEMABLE`
- `SETTLED`
- `EXPIRED`
- `RECLAIMED`

Core mocked methods should converge on current names:

- `bounties.list`
- `bounties.get`
- `bounties.status`
- `judge.status`
- `judge.attest`
- `dev.mock_attest`
- `dev.force_state`

Deferred compatibility methods such as `bonds.*` and `oracle.*` may remain while
the code rename is incomplete, but tests should make the intended canonical name
explicit whenever new behavior is added.

Fixture requirements:

- every mock bounty has `serial`, `holder_pubkey`, `alg_epoch`, `terms`, `amount`,
  `currency`, `lock_mechanism`, `deadline`, `expiry`, `grace_days`, and quorum
  metadata
- external evidence references must carry URI, media type, size when known, and
  SHA-256 digest
- embedded evidence must carry the same digest metadata

## 7. UI Mockup and Plugin Tests

Active UI mockups are generated from `bin/` scripts. The generator is the source
of truth; generated HTML is verification output.

For a mockup change:

1. edit the relevant `bin/<mockup-generator>`
2. regenerate only affected HTML under `doc/04_STORYBOARDS/out/`
3. update `doc/04_STORYBOARDS/out/index.html` if links/cards changed
4. run targeted stale-term searches
5. open changed HTML in a browser when layout or navigation changed

Required stale-term checks for active mockups and generators:

- no old user-facing `--bond` flag where `--bounty` is intended
- no active `bounty-wallet` or `oracle-registry` filenames/links
- no user-facing `oracle` where `judge` is intended
- no old `.bond` wording where `.bounty` is intended
- no stale bonus-multiplier or DRK/DarkFi wording

Plugin architecture checks:

- active mockup names match plugin registry names
- CLI suite mirrors plugin boundaries
- disabled or v2 preview plugins are labelled as such
- MVP plugins expose XMR, TARI, and NYX where the MVP spec requires them

## 8. Flutter + Rust/WASM App Tests

The Flutter frontend plus Rust/WASM backend module is the primary user-facing
development target.

Required checks:

- Flutter web static app bundle builds without CDN dependencies
- Rust backend module compiles to WebAssembly
- Dart-to-WASM interop calls handle success, validation failure, malformed
  input, and thrown/returned Rust errors
- app loads from local static files in Chromium and Firefox
- initial route renders without console errors
- plugin navigation renders enabled/disabled plugin states correctly
- mock bounty list/detail/claim/judge flows work without real backend
- `.bounty` inspect/verify path works through Rust/WASM core logic or local RPC
- offline/local-first mode handles missing network gracefully
- keyboard navigation and basic accessibility smoke checks pass
- generated bundle hashes are reproducible for identical inputs

Suggested commands once scripts exist:

```sh
src/scripts/build-web.sh
npx playwright test tests/e2e/wasm-app.spec.ts
```

## 9. Rust MVP Test Targets

The formal Rust suite should move toward these MVP crates and invariants.

### 8.1 Bounty/file crate

Must test:

- `.bounty` file creation, open, inspect, and schema migration
- SQLite table presence and required fields
- `bounty_id` determinism from canonical serialized bounty spec
- `serial` uniqueness within an issued set
- deadline/expiry validation, including the 10-year hard cap
- evidence embedded/reference validation and digest verification
- SQLCipher/encryption behavior when enabled
- corrupted or truncated files fail closed

### 8.2 Judge crate

Must test:

- judge registration and key validation
- accept/reject panel workflow
- evidence review and attestation signing
- quorum counting
- challenge window handling
- invalid signatures and wrong-key signatures fail
- duplicate/conflicting attestations are rejected or flagged

### 8.3 Collateral adapters

Must test:

- XMR DLEQ commitment construction and validation
- payout and reclaim paths select the correct adaptor secret
- wrong scalar cannot unlock the wrong path
- TARI/NYX reward or test-ledger paths are deterministic and auditable
- unsupported currency/lock-mechanism combinations are rejected
- address parsers reject wrong-network or wrong-currency addresses

### 8.4 CLI crate

Must test:

- user-facing flags use `--bounty`
- internal storage may use `bounty_id`
- JSON output schemas are stable
- dry-run never mutates state
- errors are descriptive and exit non-zero
- plugin command availability matches enabled plugins

## 10. End-to-End Gates

### 9.1 Mock lifecycle gate

Required before replacing Mockoon with real backend:

- create bounty through CLI or UI
- list and inspect it
- attach evidence
- judge accepts/reviews/attests
- quorum transitions state to `REDEEMABLE`
- transfer/claim/redeem Flutter/Rust-WASM app paths are navigable against mocks
- failure paths cover expired, rejected, and quorum-not-reached cases

### 9.2 Real backend gate

Required before Phase 5:

- `.bounty` files are real SQLite files, not fixture-only outputs
- CLI create/inspect/status/verify/list/transfer operate on real files
- judge attestations are signed and verified
- Mockoon contract has been replaced or matched by real RPC responses
- Python CLI tests and relevant Rust crate tests pass

### 9.3 Monero stagenet gate

Required MVP proof:

- issue a bounty with real XMR stagenet lock transaction
- judge quorum attests goal met
- holder redeems by combining adaptor data with `s_met`
- issuer can reclaim after expiry with the failure path
- double redemption/reclaim attempts fail
- transaction IDs and `.bounty` state transitions are recorded

## 11. Deferred v2 Test Plan

These remain valuable but are not MVP blockers:

- Halo2 MINT/TRANSFER/BURN proof benchmarks
- shielded note commitment/nullifier tests
- libp2p/P2P network propagation
- full Playwright browser E2E against a local node
- MCP provider quality tests beyond CLI availability
- old DarkFi/DRK-specific tests

When v2 work resumes, create a separate v2 section or branch update rather than
mixing those requirements into the MVP gate.

## 12. Coverage Targets

Minimum targets once the real Rust MVP crates exist:

| Area | Target |
| :--- | :--- |
| `.bounty` schema/read/write | 90% |
| judge signing/quorum logic | 90% |
| CLI command parsing/output | 85% |
| collateral adapter safety paths | 90% |
| mock API/fixture consistency | 100% mechanical checks |
| Flutter/Rust-WASM app navigation against mocks | critical flows covered |

Coverage is secondary to invariant coverage. Safety-critical negative tests are
required even if line coverage appears high.

## 13. Known Gaps

| Gap | Risk | Next action |
| :--- | :--- | :--- |
| Formal Rust MVP bounty crate tests are not yet implemented | High | Add tests as crate is created or renamed |
| `test/test_oracle.py` still uses legacy oracle naming | Medium | Rename or replace during judge/RPC cleanup |
| `bin/consistency-check` still references `bond.rs` and `bonds.*` | Medium | Update when code/RPC rename lands |
| Mock API path in MVP says `src/mock/...` while repo has `mock/...` | Low | Normalize doc path during mock cleanup |
| UI mockup tests are mostly `rg`/manual browser checks | Medium | Add Playwright smoke tests for generated mockups |
| Flutter/Rust-WASM app smoke-test scripts are not yet defined | High | Add static bundle build, Dart-WASM interop tests, and browser smoke test |
| Stagenet lifecycle is not automated | High | Add Phase 5 harness with isolated wallet/node state |
