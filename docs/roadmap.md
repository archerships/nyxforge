# NyxForge — Roadmap to Mainnet

> Written March 2026.  Living document — update as phases complete.

---

## Overview

NyxForge's path to mainnet has one external dependency that cannot be controlled:
**DarkFi L1 mainnet**.  As of March 2026, DarkFi is on testnet with no public
ETA for mainnet.  Conservative estimate: 1–3 years.

The roadmap is therefore structured so that NyxForge becomes a **fully
functional, ZK-private, P2P bond market** before DarkFi mainnet exists, using
NyxForge's own P2P network for consensus in the interim.  When DarkFi mainnet
launches, a migration adds L1 settlement finality without changing the
application.

```
Phase 0 → 1 → 2 → 3 → 4 → 4.5      (NyxForge-controlled work)
                            ↓
                  Phase 5: NyxForge Network launch (P2P consensus, real ZK)
                            ↓
                  Phase 6: DarkFi Testnet integration (parallel)
                            ↓
                  Phase 7: Security audit
                            ↓
                  Phase 8: DarkFi Mainnet migration
```

Phases 1–5 are independent of DarkFi.  Phase 6 can begin as soon as Phase 1
is complete (ZK proofs real).  Phase 8 is gated on DarkFi, not on us.

---

## Current State (Phase 0 — Complete)

| Component | Status |
|---|---|
| Bond lifecycle state machine (Proposed → Settled) | Done |
| CLI bond wizard (all lifecycle commands) | Done |
| Oracle registration, approval, attestation | Done |
| XMR wallet (stagenet), key derivation, recovery | Done |
| RandomX miner with P2Pool Stratum client | Done |
| Node JSON-RPC + libp2p gossip scaffold | Done |
| MCP server (Anthropic / OpenAI / Ollama) | Done |
| AI-assisted bond exploration (`bond explore`) | Done |
| Community proposal review (propose / comment / submit) | Done |
| Unit test suite (139 tests, 0 failures) | Done |
| ZK proof generation & verification | **Stubbed — always returns true** |
| Browser WASM UI | **Scaffold only — splash + navigation** |
| Order book / DEX | **Not started** |
| DarkFi L1 deployment | **Blocked on DarkFi mainnet** |

---

## Phase 1 — Real ZK Proofs

**Goal:** Replace all stubbed ZK verifiers with real Halo2 proof generation and
verification.  After this phase, every bond note operation (mint, transfer, burn)
is cryptographically enforced — no longer simulated.

**Why first:** Everything else depends on correct proofs.  The order book, the
browser UI, and the DarkFi integration all assume real proofs.  Doing this first
means every subsequent phase is built on solid ground.

### Deliverables

#### 1.1 MINT circuit
- Implement `prove_mint(note) → MintProof` in `nyxforge-zk` using Halo2
- Implement `verify_mint(proof, commitment) → bool`
- Inputs: `bond_id`, `quantity`, `owner_pk`, `randomness`
- Public output: `commitment`
- Wire into `bonds.issue` RPC: node calls `prove_mint` before minting

#### 1.2 TRANSFER circuit
- Implement `prove_transfer(note_old, note_new, sk) → TransferProof`
- Implement `verify_transfer(proof) → bool`
- Enforce conservation: `note_old.quantity == note_new.quantity`
- Public inputs: old nullifier, new commitment
- Wire into the order book trade settlement path

#### 1.3 BURN circuit
- Implement `prove_burn(note, sk, quorum_hash) → BurnProof`
- Implement `verify_burn(proof, bond_id) → bool`
- Public inputs: nullifier, bond_id, quorum_result_hash
- Wire into `bond redeem` CLI command and settlement contract

#### 1.4 WASM proof generation
- `wasm-pack build crates/nyxforge-zk` compiles circuits to WASM
- Browser can call `prove_mint`, `prove_transfer`, `prove_burn` locally
- Proof generation never sends private data over the network

#### 1.5 Test un-ignoring
- Un-ignore all `#[ignore]`-tagged ZK tests in `nyxforge-zk`
- Add `wasm-pack test --headless --chrome` to CI matrix
- Golden vectors updated with real Halo2 proof hashes

#### 1.6 Wallet passphrase encryption
- Encrypt `wallet.json` at rest with Argon2id-derived key
- Prompt for passphrase on node start (or `--passphrase` env var)
- Recovery from spend key works with or without passphrase

**Success criterion:** `cargo test -p nyxforge-zk` passes all ZK tests
including previously-ignored proof round-trips.  `nyxforge-contract` rejects a
tampered note commitment.

---

## Phase 2 — Browser UI

**Goal:** The Flutter WASM frontend becomes a fully functional bond market
interface.  All CLI capabilities are available in the browser.

### Deliverables

#### 2.1 Bond browser
- List all bonds with filters: state, deadline, data_id, search
- Bond detail page: goal spec, oracle list, current state, price history chart
- Link to comment thread for Proposed bonds

#### 2.2 Bond creation wizard (UI)
- Port CLI wizard to Flutter form pages
- AI-assisted `bond explore` flow integrated (calls MCP server)
- Shows estimated collateral required in DRK and XMR

#### 2.3 Wallet UI
- Display XMR address, DRK address, spend key (blurred by default)
- Show bond note balances (quantities by bond)
- Show DRK balance
- Key import / recovery from spend key

#### 2.4 Portfolio and positions
- Table of held bond notes (bond title, quantity, purchase price if recorded,
  current market price, unrealised P&L)
- Redeemable bonds flagged with "Redeem" action

#### 2.5 Order entry and order book
- Bid / ask entry form for any active bond
- Live order book display (best N bids and asks)
- Last trade price and 24h price chart

#### 2.6 Mining UI
- Thread count selector
- Start / stop miner button
- Live hashrate and share count display
- XMR balance (from monerod RPC)

#### 2.7 Node status
- Peer count, sync status
- MCP server status
- Oracle daemon status (if running)

#### 2.8 De-googled production build
- `scripts/build-web.sh` passes: no CDN URLs, CanvasKit self-hosted
- WASM bundle < 5 MB raw, < 2 MB after `wasm-opt -Oz`

**Success criterion:** All CLI commands have a UI equivalent.  Playwright E2E
suite (Layer 3 in test plan) covers bond create, proposal, oracle, and
settlement flows in browser.

---

## Phase 3 — Order Book and Trading

**Goal:** Anonymous peer-to-peer bond note trading with ZK transfer proofs.

### Deliverables

#### 3.1 Order book contract
- `order_book.rs` in `nyxforge-contract`:
  - `PlaceOrder { bond_id, side, price, quantity, commitment, expiry }`
  - `CancelOrder { order_id, nullifier }`
  - `MatchOrders { bid_id, ask_id, transfer_proof_bond, transfer_proof_drk }`
- Orders stored in `orders.db` (sled); gossiped to all peers

#### 3.2 Atomic swap settlement
- Matched trade requires two TRANSFER proofs simultaneously:
  - Seller: bond note → buyer's key
  - Buyer: DRK note → seller's key
- Both nullifiers checked before commitment to state
- Atomic: either both succeed or neither does

#### 3.3 Order matching engine
- Local matching: each node independently matches the best bid/ask
- First node to broadcast a valid `MatchOrders` message wins the race
- Conflict resolution: higher fee / earlier timestamp breaks ties (TBD)

#### 3.4 Price history
- Node records `Trade { bond_id, price, quantity, timestamp }` on each match
- RPC exposes `market.history { bond_id, from, to }` for chart data

#### 3.5 Market order (Phase 3 stretch goal)
- IOC: match immediately against best N orders; cancel remainder
- GTC: persistent; cancelled by user or expiry

**Success criterion:** Two local nodes can trade a bond note anonymously.
The TRANSFER proof verifier rejects a tampered quantity.  Playwright E2E
`bond-settlement.spec.ts` passes (real proofs, real trade).

---

## Phase 4 — Oracle Ecosystem

**Goal:** Real-world data adapters; operational oracle tooling; slashing.

### Deliverables

#### 4.1 HTTP-JSON adapter
- `DataSource` implementation: fetch a URL, extract value via JSONPath
- Config: `{ url, json_path, transform: scale|parse_float }`
- Used for: HUD Point-in-Time homeless count, NOAA climate data, WHO stats,
  any REST API returning numeric data

#### 4.2 IPFS adapter
- Fetch a pinned document by CID
- Parse numeric value from JSON or CSV content
- Useful for academic / NGO data published to IPFS

#### 4.3 Oracle CLI improvements
- `oracle list-bonds` — bonds awaiting this oracle's response
- `oracle auto-accept --data-ids <csv>` — accept any bond whose data_id matches
- `oracle review <bond_id>` — interactive accept/reject with inline GoalSpec display
- `oracle dashboard` — live view of active bonds, attestation count, fee accrued

#### 4.4 Stake management
- `oracle stake deposit <amount>` — lock DRK stake
- `oracle stake withdraw` — unlock (after challenge window on all active bonds)
- `oracle stake status` — show staked amount and at-risk bonds

#### 4.5 Slashing implementation
- `nyxforge-contract`: `slash_oracle { bond_id, oracle_key, evidence }` transaction
- Burns `slash_fraction × staked_DRK`; returns remainder to oracle
- Evidence: quorum attestation opposing the fraudulent one

#### 4.6 Dispute resolution
- Challenge window: configurable per bond (default 72 hours after finalisation)
- During window: any peer can submit `ChallengeAttestation { bond_id, evidence }`
- After window: `FinaliseVerification` commits the result permanently
- DAO vote path (for bonds with `dao_override_allowed = true`): majority of
  staked DRK holders can override the oracle quorum result

#### 4.7 Oracle operator documentation
- `docs/oracle-operator-guide.md`
- Setup walkthrough: install, configure data adapter, register with bond issuer
- Stake requirements, fee model, slashing risks

**Success criterion:** An oracle running the HTTP-JSON adapter correctly
evaluates a live HUD homeless-count bond.  A fraudulent attestation triggers
slashing.  The Playwright `bond-oracle.spec.ts` suite passes with the real
slashing path.

---

## Phase 4.5 — Privacy Hardening

**Goal:** Ensure the AO/Arweave permanent ledger reveals zero information about
bond subjects, goal criteria, or the timing of real-world events.  This phase
implements two orthogonal defenses documented in `docs/privacy-design.md` §§5–6.

### Deliverables

#### 4.5.1 Encrypted Goal Text & Bond View Keys

Goal specifications (title, description, `data_id`, threshold) are sensitive
metadata about the bond subject.  Publishing them in plaintext on a permanent
public ledger is unacceptable.

- Add `GoalVisibility` enum to `nyxforge-core`:
  ```rust
  pub enum GoalVisibility {
      Private,  // ChaCha20-Poly1305 encrypted; default
      World,    // Plaintext; issuer opts in explicitly
  }
  ```
- Add `visibility` and `ciphertext: Option<Vec<u8>>` fields to `GoalSpec`.
- Implement `GoalSpec::encrypt(view_key)` / `GoalSpec::decrypt(view_key)`.
- View key derivation: `bond_view_key = blake3(issuer_spend_key ‖ bond_id)`
  — one key per bond, independent, shareable without exposing spend key.
- CLI: `bond view-key <bond_id>` — prints the view key for sharing with
  bondholders, oracles, or auditors.
- CLI: `bond show <bond_id> --view-key <hex>` — decrypts and displays goals.
- Browser UI: view key entry field on bond detail page.
- Halo2 MINT circuit: commit to *plaintext* goal hash inside proof; the ZK
  system enforces that the on-chain ciphertext corresponds to valid goals.
- Oracle key exchange: out-of-band view key delivery over DarkFi encrypted DM.

**Success criterion:** A bond issued with `Private` goals shows only ciphertext
on the AO ledger.  Holder with view key decrypts correctly.  Oracle without
view key cannot read goal text.

#### 4.5.2 Oracle Push Attestation Model

Replace the AO Cron polling model with a push model: oracles monitor data
sources continuously and submit attestations to AO the moment a goal condition
is observed (see `docs/ao-strategy.md` §2.4 for rationale).

- Implement `OracleNode::monitor_bonds(bonds, attestation_tx)` in
  `crates/nyxforge-oracle/src/oracle.rs`:
  - Takes a `Vec<Bond>` and `tokio::sync::mpsc::Sender<OracleAttestation>`.
  - Spawns one monitoring task per bond; each task polls all goals on the
    bond's configured `poll_interval_secs`.
  - When ALL goals are met (AND semantics), sends attestation on the channel.
  - Respects `poll_lead_secs`: monitoring starts before the deadline.
  - Tasks are cancelled automatically when the bond leaves `Active` state.
- AO Cron retained only for lifecycle events (expiry, slashing window, emission).

**Success criterion:** Integration test: mock data source transitions a bond's
value across the threshold; `monitor_bonds` fires exactly one attestation
within one poll interval.  No attestations fired when only a subset of goals
are met.

#### 4.5.3 Noise Bond Anonymity Set

An oracle attestation appearing on the AO ledger is a timing signal even when
goal text is encrypted — an observer can correlate the timestamp to real-world
events.  Noise bonds eliminate this by creating a constant-rate stream of
indistinguishable attestations.

- `NoiseBondConfig` in oracle config:
  ```rust
  pub struct NoiseBondConfig {
      pub emission_rate_per_hour: u32,    // e.g. 12 (one per 5 min)
      pub window_size_secs: u64,          // e.g. 300
      pub registry_key: [u8; 32],         // shared only with legitimate holders
  }
  ```
- `OracleNode::generate_noise_attestations(config)`: returns dummy
  `OracleAttestation` values signed with the oracle key, referencing registered
  noise bond IDs; submitted at each window boundary.
- Real attestations are held in a queue and released at the next noise window
  boundary, limiting timing resolution to `window_size_secs`.
- Noise bond IDs registered in a side-channel registry (encrypted with
  `registry_key`); legitimate holders filter them; external observers cannot.
- Protocol documentation: `docs/noise-bond-spec.md`.

**Success criterion:** With 12 noise attestations/hour and one real attestation,
an observer watching the AO ledger cannot determine which attestation is real
at better than 1-in-12 odds (uniform distribution across the window).

---

## Phase 5 — NyxForge Network Launch (Pre-DarkFi Mainnet)

**Goal:** A live, real-money-capable bond market using NyxForge P2P consensus
for finality, before DarkFi L1 exists.  Explicitly disclosed as pre-mainnet.

### What changes from DarkFi L1 settlement

| Property | NyxForge Network | DarkFi Mainnet |
|---|---|---|
| ZK proofs | Real Halo2 | Real Halo2 |
| Bond note privacy | Full (ZK commitments) | Full (ZK commitments) |
| Trade privacy | Full (ZK transfers) | Full (ZK transfers) |
| Settlement finality | NyxForge node supermajority | DarkFi L1 immutability |
| Censorship resistance | High (P2P, no central server) | Maximal (L1-enforced) |
| Collateral currency | XMR (held in multi-sig or timelock) | DRK (native L1) |

The honest framing: NyxForge Network is a federated, ZK-private system.
Finality relies on the NyxForge P2P network's honest majority.  Users accept
this tradeoff in exchange for using the system years before DarkFi mainnet.

### Deliverables

#### 5.1 Collateral mechanism (pre-DarkFi)
- Collateral is locked in XMR using a time-locked or multi-sig scheme
  (simplest viable: 2-of-3 multi-sig between issuer + two well-known oracle
  operators acting as collateral custodians)
- Redemption: oracles countersign a release transaction to the redeemer's
  address on Monero mainnet
- This is centralised-ish and disclosed; replace with DRK in Phase 8

#### 5.2 Bootstrapping infrastructure
- 3 bootstrap nodes with known peer IDs and stable public IPs
- DNS seed entry (`_nyxforge._tcp.dark.example`) listing bootstrap peers
- `nyxforge.toml` default config points to bootstrap peers

#### 5.3 Network monitoring dashboard
- Public-facing read-only web UI (static HTML/JS, no Flutter required)
- Shows: active bonds, total collateral, oracle count, recent trades
- Data served by a bootstrap node's public RPC (read-only subset)

#### 5.4 Launch checklist
- [ ] Security audit complete (Phase 7)
- [ ] ZK proofs real (Phase 1)
- [ ] Browser UI complete (Phase 2)
- [ ] Trading complete (Phase 3)
- [ ] Oracle ecosystem complete (Phase 4)
- [ ] Wallet passphrase encryption complete
- [ ] 30-day soak on testnet (multi-node, real oracle data)
- [ ] User documentation published
- [ ] Oracle operator onboarding: ≥ 5 operators pre-registered

---

## Phase 6 — DarkFi Testnet Integration (Parallel)

**Can begin as soon as Phase 1 is complete.**  Runs in parallel with Phases 2–5.

### Deliverables

#### 6.1 Contract WASM build
- `nyxforge-contract` compiled to `wasm32-unknown-unknown` with DarkFi SDK
- Deployed to DarkFi testnet via `drk deploy`
- Contract address stored in config

#### 6.2 Replace in-process state with on-chain reads/writes
- `nyxforge-node` calls `darkfid` JSON-RPC for bond state instead of local sled
- `bonds.issue` submits a DarkFi transaction instead of updating local DB
- Nullifier set reads from DarkFi chain instead of local `nullifiers.db`

#### 6.3 Wire ZK proofs through DarkFi zkVM
- MINT / TRANSFER / BURN proofs submitted as DarkFi transactions
- Verifier is DarkFi's on-chain zkVM (ZKAS / Halo2)
- Local proof generation unchanged; submission path changes

#### 6.4 DRK wallet integration
- Replace stub DRK balance with real `darkfid` wallet queries
- `wallet.drk_balance` reads from darkfid
- Bond collateral deposits and redemption payouts are real DRK transactions

#### 6.5 Testnet soak
- Run full bond lifecycle end-to-end on DarkFi testnet with real DRK (test
  funds available from DarkFi faucet)
- Automated regression: nightly CI run that creates, trades, and settles a bond
  on testnet using mock oracle

---

## Phase 7 — Security Audit

**Gate:** Must complete before Phase 5 (network launch) or Phase 8 (mainnet).

### Scope

| Area | Risk | Audit depth |
|---|---|---|
| Halo2 ZK circuits (MINT / TRANSFER / BURN) | Critical — soundness failure breaks all privacy | External auditor with Halo2 expertise |
| `nyxforge-contract` (bond lifecycle, nullifier set, collateral) | Critical — fund safety | External auditor |
| P2P gossip / Sybil resistance | High — network manipulation | External auditor |
| Oracle slashing logic | High — economic attack surface | External auditor |
| `nyxforge-node` RPC (injection, auth, DoS) | Medium | Internal + external |
| Flutter WASM frontend (key storage, XSS) | Medium | Internal + external |
| Wallet key management (derivation, storage) | High | External auditor |

### Preparatory steps (before engaging auditor)
- [ ] `cargo audit` clean (no known-vulnerable dependencies)
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo llvm-cov --workspace` ≥ targets per test plan (§8)
- [ ] `cargo fuzz` targets for RPC JSON deserialization and ZK input parsing
- [ ] Property tests (`proptest`) for `GoalMetric` evaluation
- [ ] Threat model document (`docs/threat-model.md`)

### Budget estimate
- ZK circuit audit: $40k–$80k (specialised; Trail of Bits, ZKCS, or similar)
- Contract + P2P audit: $30k–$60k
- Total: $70k–$140k

---

## Phase 8 — DarkFi Mainnet Migration

**Gated on DarkFi mainnet launch (external dependency).**

### Deliverables

#### 8.1 Contract deployment
- Deploy `nyxforge-contract` WASM to DarkFi mainnet
- Contract address published in `nyxforge.toml` and documentation

#### 8.2 Collateral migration
- NyxForge Network bonds: issuer and oracles co-sign release of XMR collateral
- Announce migration window (90 days)
- New bonds issued only on DarkFi mainnet

#### 8.3 Transport privacy
- Switch P2P transport from libp2p/Noise to DarkFi's native P2P (Tor-routed,
  QUIC/Nym)
- IP addresses no longer visible to peers

#### 8.4 DRK as bond currency
- Remove XMR multi-sig collateral mechanism
- All new bonds denominated and collateralised in DRK
- XMR mining revenue → DRK conversion path (via DarkFi atomic swap or DEX)

#### 8.5 Final documentation
- Multi-language docs (English first; translations by community)
- Oracle operator onboarding programme
- Bond issuer guide with worked examples (homelessness, climate, public health)

---

## Dependency Map

```
Phase 1 (ZK proofs)
  ├── required by Phase 2 (UI proof generation in browser)
  ├── required by Phase 3 (trade settlement proofs)
  ├── required by Phase 6 (DarkFi testnet)
  └── required by Phase 7 (audit scope)

Phase 2 (Browser UI)
  └── required by Phase 5 (network launch)

Phase 3 (Order book)
  └── required by Phase 5 (network launch)

Phase 4 (Oracle ecosystem)
  └── required by Phase 4.5 (privacy hardening)

Phase 4.5 (Privacy hardening)
  ├── requires Phase 4 (oracle push model builds on oracle infrastructure)
  └── required by Phase 5 (network launch)

Phase 5 (NyxForge Network launch)
  ├── requires Phases 1–4.5 complete
  └── requires Phase 7 (security audit) complete

Phase 6 (DarkFi testnet)
  ├── requires Phase 1 complete
  └── runs in parallel with Phases 2–5

Phase 7 (Security audit)
  ├── requires Phase 1 complete (real proofs to audit)
  └── runs in parallel with Phases 2–5 (ideally starts mid-Phase 3)

Phase 8 (DarkFi mainnet)
  ├── requires Phase 5 (NyxForge Network launched)
  ├── requires Phase 6 (DarkFi testnet integration tested)
  └── gated on DarkFi L1 mainnet launch (external)
```

---

## Milestone Summary

| Milestone | What ships | Gate |
|---|---|---|
| **M1: Real ZK** | Phase 1 complete. All proofs are real. Stubs removed. | Internal |
| **M2: Full UI** | Phase 2 complete. Browser-only user can do everything CLI user can. | Internal |
| **M3: Live Trading** | Phase 3 complete. Anonymous ZK bond trading between nodes. | Internal |
| **M4: Oracle Ecosystem** | Phase 4 complete. Real data adapters. Slashing live. | Internal |
| **M4.5: Privacy Hardening** | Phase 4.5 complete. Goal text encrypted. Push oracles live. Noise bonds operational. | Internal |
| **M5: Testnet soak** | Phases 1–4.5 running on multi-node testnet for 30 days, stable. | Internal |
| **M6: Security audit** | Phase 7 complete. All critical findings resolved. | External auditor |
| **M7: NyxForge Network** | Phase 5. Live real-money P2P bond market (pre-DarkFi). | M5 + M6 |
| **M8: DarkFi Testnet** | Phase 6 complete. Full stack on DarkFi testnet. | M1 + DarkFi testnet |
| **M9: Mainnet** | Phase 8. DarkFi mainnet, L1-settled bonds. | M7 + M8 + DarkFi mainnet |

---

## Open Questions

| Question | Affects | Resolution needed by |
|---|---|---|
| XMR multi-sig collateral mechanism: 2-of-3 with oracle custodians, or Monero atomic swap into a timelock? | Phase 5 | Before M7 |
| Oracle fee model: per-bond flat fee, percentage of collateral, or staking yield? | Phase 4 | M4 |
| Protocol fee level and governance: 0.1%? Who controls the treasury DAO? | Phase 3 | M3 |
| Challenge window adjudication: pure quorum override, or DAO vote? | Phase 4 | M4 |
| DarkFi L1 custom contract capability: can third parties deploy WASM contracts on testnet? Needs contact with DarkFi team. | Phase 6 | Before M8 |
| Security audit budget: source of funding? | Phase 7 | Before M6 |
