> **STATUS: HISTORICAL / SUPERSEDED — 2026-04-25**
> This document describes the original DarkFi L1 / DRK-token / ZK-note architecture
> that was the pre-MVP NyxForge design (March 2026). It was superseded by the
> bearer-file approach specified in `doc/00_MVP.md`: SQLite .bounty files,
> DLEQ/PTLC adaptor signatures, XMR/ZEC/BTC/ETH collateral, no DarkFi dependency.
> Preserved for historical reference only. Do not use as an authority source.


# NyxForge — Development Plan to v1

> Written: March 2026.  Living document.
> Definition of v1: **NyxForge Network Launch** (roadmap Phase 5) — a live,
> real-money-capable bond market running on NyxForge's own P2P network,
> explicitly pre-DarkFi-mainnet, with ZK privacy enforced and full browser UI.

---

## Part 1 — Two Decisions That Must Be Made First

These are blocking questions that affect nearly every subsequent work package.
They cannot be resolved by writing more code; they require a choice.

### Decision 1: What is payout currency before DarkFi mainnet?

Every spec says the BURN proof "commits to a new DRK payout note."  DarkFi is
on testnet with no mainnet ETA.  For the NyxForge Network launch (Phase 5) to
handle real money, you need to decide:

**Option A (recommended):** Payout is XMR.  The payout commitment in the BURN
proof is a commitment to a Monero stealth address + amount.  Oracles co-sign
a Monero release transaction alongside verifying the BURN proof.  Collateral
and payout use the same currency — simple, auditable, no new protocol needed.

**Option B:** Payout is an IOU note in the NyxForge P2P network, redeemable
for XMR when the issuer co-signs a release.  More flexible but adds a trust
layer and complicates the UX.

**Option C:** Wait for DarkFi testnet, use DRK test tokens for Phase 5.
Pushes Phase 5 to whenever DarkFi testnet is stable enough.

**Recommendation:** Option A.  Every other design decision downstream (BURN
circuit payout commitment, settlement contract, UI payout flow) depends on
knowing the answer.  Pick it, write it down, and close the question.

---

### Decision 2: Does the TRANSFER circuit need a Merkle membership proof?

The current TRANSFER circuit proves *knowledge of a valid note and its
nullifier* but does not prove the note was ever recorded in the commitment
tree.  In theory, a malicious actor who learns the Poseidon parameters could
construct a fake TRANSFER for a note that was never minted.

In practice, the node software refuses to record a nullifier it hasn't seen
a commitment for — so the off-circuit check partially mitigates this.  But
it is not cryptographically enforced.

**Option A (recommended for v1):** Defer the Merkle proof.  Document the
limitation explicitly.  Before Phase 5 launches, publish a security disclosure
that the commitment tree membership check is node-enforced, not
circuit-enforced.  Fix it in Phase 6 when DarkFi handles the note tree.
This avoids 4-6 weeks of Merkle gadget work that becomes moot at DarkFi
integration.

**Option B:** Add the Merkle proof now.  Requires an incremental Merkle tree
gadget in Halo2, adding depth-20 Poseidon hash gates to the TRANSFER circuit.
Significant work; blocks everything else while you build it.

**Recommendation:** Option A.  The node-level enforcement catches the attack
in the pre-DarkFi phase.  Document it clearly.  Move on.

---

## Part 2 — Current State (Actual, as of March 2026)

| Component | Status | Notes |
|---|---|---|
| MINT circuit (real Halo2) | **Done** | Poseidon2 commitment scheme |
| TRANSFER circuit (real Halo2) | **Done** | No Merkle proof (see Decision 2) |
| BURN circuit (real Halo2) | **Done** | oracle_attest_pk constraint (Option A) |
| ZK oracle attestation (Option A) | **Done** | oracle generates per-bond key |
| Oracle push model | **Done** | monitor_bonds() |
| Bond lifecycle state machine | **Done** | Proposed → Settled |
| CLI bond wizard | **Done** | full lifecycle |
| Oracle approval workflow | **Done** | accept/reject with attest_key stub |
| XMR wallet (stagenet) | **Done** | key derivation, recovery |
| RandomX miner / P2Pool | **Done** | |
| Node JSON-RPC + libp2p scaffold | **Done** | |
| MCP server | **Done** | |
| 214 unit/integration tests | **Done** | 6 slow proof tests still ignored |
| WASM proof generation | **Not done** | wasm-pack target untested |
| Wallet passphrase encryption | **Not done** | wallet.json unencrypted at rest |
| Goal text encryption | **Not done** | spec written, not built |
| Oracle key registration on bond | **Not done** | oracle_attest_pks always [] |
| HTTP data adapter (real) | **Partial** | DataSource trait + scaffold |
| Oracle slashing | **Not done** | |
| Order book / trading | **Not done** | |
| Browser UI | **Scaffold only** | splash + nav |
| Collateral mechanism | **Not done** | |
| P2P network consensus rules | **Not done** | |

---

## Part 3 — Explicit Descopes

The following are valid long-term goals that are **cut from v1**.  Do not
let them consume design energy until after Phase 5 launches.

| Item | Why Cut | When to Revisit |
|---|---|---|
| Noise bonds (Phase 4.5.3) | Superseded by Option A — oracle attestations never appear on AO at all; there is nothing to add noise to.  The timing signal is now BURN nullifiers, not attestations.  Re-evaluate if noise-BURN is needed post-launch. | After Phase 5 |
| Kleros / UMA arbitration (oracle-spec.md Tier 3) | No integration path exists yet; the roadmap's slashing model is sufficient for v1 | Phase 6+ |
| zkTLS / DECO / TLS-Notary | Interesting but requires substantial external dependency; HTTP-JSON adapter is sufficient for v1 data sources | Phase 6+ |
| zkVM (RISC Zero / Succinct SP1) | Same as zkTLS | Phase 6+ |
| Long Now endowment / yield vault | Century-scale bonds can be issued once the basic lifecycle works; the endowment mechanics are not needed for v1 | Phase 6+ |
| Post-quantum verifier slots | No practical threat until at least 2035; add when standards stabilise | Phase 7+ |
| Cake Wallet URI / RetoSwap atomic swap / Vexl | These improve UX for collateral funding but are not required for v1 | Phase 5 stretch |
| Coin-agnostic collateral (BTC/ETH/AR/AO plugins) | XMR only for v1; plugin architecture deferred | Phase 6+ |
| Multi-language docs | English first | Post-launch |
| NYX token / fair launch | Important for sustainability but decoupled from the bond market mechanics | Phase 5 launch or shortly after |

---

## Part 4 — Work Packages

Packages are ordered by dependency.  Each package lists its blocking
predecessors.  Estimated effort is in developer-days for a single
experienced Rust/Flutter developer.

---

### W1 — Spec Alignment  *(3 days)*
*Blocks: everything else.  No code, just writing.*

The following docs contain stale or conflicting information that will
confuse anyone reading them.  Fix them before writing more code so you
have a clear target.

**Docs to update:**

- **`docs/zk-design.md`** — Replace all references to PedersenCommit with
  Poseidon2.  Replace `quorum_result_hash` with `oracle_attest_pk`.  Add the
  Decision 2 note (Merkle proof deferred, reason documented).  Remove the
  `C_old == PedersenCommit(note_old)` constraint from the TRANSFER spec.

- **`docs/bond-lifecycle.md`** — Redemption section: replace the old
  quorum / FinaliseVerification → REDEEMABLE flow with the Option A flow
  (oracle shares attest_key privately → bondholder constructs BURN proof →
  submits to contract from Active state).  Update the state machine diagram.

- **`docs/architecture.md`** — Data Flow: Redemption section: replace old
  oracle gossip / quorum / REDEEMABLE flow with Option A flow.

- **`docs/privacy-design.md`** — §3.3 BURN: replace "oracle quorum hash"
  with oracle_attest_pk.  §6 Noise Bonds: add a note that with Option A,
  attestations never appear on AO, so the timing linkability problem the
  noise bonds were solving no longer exists in the same form.  Mark §6 as
  "under re-evaluation."

- **`docs/oracle-spec.md`** — Fix duplicate §3/§4 headers.  Add a clear
  "current implementation" section at the top that describes the actual
  Option A model.  Mark the Tier 1/2/3 section as "long-term vision, not
  Phase 5 scope."

- **`docs/spec-summary.md`** — Update test count (214, not 176).  Remove
  "Coin-agnostic collateral" from the key features list (replace with
  "XMR collateral, DRK payout" or whatever Decision 1 resolves to).

- **`docs/roadmap.md`** — Mark Phase 1 as ~65% done.  Update Phase 4.5.3
  (noise bonds) status.  Add oracle key registration as Phase 1.5.

**New docs to write:**

- **`docs/decisions.md`** — Record the two decisions above (payout currency,
  Merkle proof deferral) with rationale.  Use this file for all future
  architectural decisions so reasoning is preserved.

- **`docs/security-limitations.md`** — One page listing known security
  tradeoffs that are accepted for v1 (Merkle proof not circuit-enforced,
  pre-mainnet P2P consensus, XMR multi-sig for collateral).  This becomes
  the basis for the security audit scope document.

---

### W2 — Oracle Key Registration Protocol  *(4 days)*
*Blocks: W5, end-to-end BURN proof.*
*Depends on: nothing (design decision in W1 suffices).*

Currently `oracle_attest_pks` is always `[]` on every bond.  The circuit
exists; the registration flow does not.

**What to build:**

1. **`OracleNode::generate_attest_key(bond_id)`** — already implemented
   in oracle.rs.  Now surface it.

2. **Oracle accept RPC** (`oracle.accept_bond`): when an oracle accepts a
   bond, it calls `generate_attest_key(bond_id)`, stores the
   `OracleAttestKey` locally (encrypted at rest with the oracle's secret),
   and returns `oracle_attest_pk` in the accept response.

3. **Bond approval handler** in `nyxforge-node/src/rpc.rs`: when all oracles
   have accepted, collect their `oracle_attest_pk` values and set
   `bond.oracle.oracle_attest_pks`.  Gossip the updated bond.

4. **Bondholder BURN flow**: the CLI `bond redeem` command must retrieve the
   `oracle_attest_key` from the oracle (via encrypted P2P message or manual
   out-of-band delivery), supply it as the `BurnWitness.oracle_attest_key`,
   and construct the proof.

5. **Key delivery protocol** (minimal for v1): a simple request-response RPC
   between bondholder and oracle node.  The oracle verifies the requester
   owns notes for the bond (via a signed challenge) before releasing the
   attest_key.  Full DarkFi encrypted-DM delivery is a Phase 6 upgrade.

6. **Tests**: integration test covering the full flow — bond approved with
   registered PKs, attest_key delivered to bondholder, BURN proof generated
   and verified by settlement contract.

---

### W3 — Complete Phase 1 ZK  *(5 days)*
*Blocks: W7 (browser proof generation).*
*Depends on: W1 (spec alignment).*

**3a — Wallet passphrase encryption** *(1 day)*
- Argon2id KDF over user passphrase → 256-bit key
- Encrypt `wallet.json` at rest (ChaCha20-Poly1305 + random nonce)
- Prompt on node start; accept `NYXFORGE_PASSPHRASE` env var
- Recovery from spend key still works without passphrase

**3b — Un-ignore slow proof tests** *(2 days)*
- The 6 `#[ignore]` proof roundtrip tests are currently skipped everywhere
- Run them, measure times, decide: either they're fast enough to always run
  (in which case remove the ignore), or create a `cargo test --ignored`
  nightly CI target
- Add `cargo test --ignored -p nyxforge-zk` to CI as a separate slow job

**3c — WASM proof generation** *(2 days)*
- `wasm-pack build crates/nyxforge-zk --target web`
- Expose `prove_mint`, `prove_transfer`, `prove_burn` as `#[wasm_bindgen]`
  exported functions
- Add `wasm-pack test --headless --firefox` to CI
- Prove that private witness data never crosses the RPC boundary (proof
  generated in browser, only the proof bytes sent to the node)

---

### W4 — Goal Text Encryption  *(4 days)*
*Blocks: W7 (browser bond creation with private goals).*
*Depends on: W1.*

Implements Phase 4.5.1 from the roadmap.

**4a — Core types** *(1 day)*
- `GoalVisibility` enum (`Private` / `World`) in `nyxforge-core`
- `visibility` and `encrypted_goal: Option<Vec<u8>>` fields on `GoalSpec`
- `GoalSpec::encrypt(view_key: &[u8; 32])` / `::decrypt(view_key)` using
  ChaCha20-Poly1305 (use the `chacha20poly1305` crate)
- View key derivation: `blake3(issuer_spend_key ‖ bond_id)` — already
  described in privacy-design.md

**4b — CLI** *(1 day)*
- `bond view-key <bond_id>` — derives and prints view key
- `bond show <bond_id> --view-key <hex>` — decrypts and displays goals
- Bond creation wizard: ask "Make goal text private? [Y/n]"
- Default: private

**4c — Oracle key exchange** *(1 day)*
- When issuer shares view key with oracle during approval (alongside the
  attest_key registration in W2), oracle stores it encrypted locally
- Oracle `evaluate()` decrypts goal spec before fetching data if
  `visibility == Private`

**4d — Tests** *(1 day)*
- Round-trip: encrypt → store → decrypt with correct key → decrypt fails
  with wrong key
- Issuer bond with private goals: AO record shows only ciphertext
- Oracle without view key cannot read goal text

---

### W5 — Oracle Ecosystem  *(8 days)*
*Blocks: W8 (a live network needs real oracles).*
*Depends on: W2 (attest_key registration), W1.*

This is a subset of Phase 4 — only what v1 requires.

**5a — HTTP-JSON data adapter** *(2 days)*
- Concrete `HttpJsonSource` implementing `DataSource`:
  - Config: `{ url: String, json_path: String, transform: Option<Scale|ParseFloat> }`
  - Fetches URL via `reqwest`, extracts value via `jsonpath_lib` or simple
    key navigation, converts to `Decimal`
- Register in `OracleNode` config via TOML
- Works for: HUD homeless count, NOAA CO2, WHO stats, any REST JSON API
- Integration test: mock HTTP server returns JSON, oracle evaluates correctly

**5b — Slashing implementation** *(3 days)*
- `slash_oracle { bond_id, oracle_key, evidence }` in `nyxforge-contract`
- Evidence is an Ed25519 signature from the oracle over a false attestation,
  plus the contradicting BURN proof showing the oracle_attest_pk was valid
- Burns `slash_fraction × staked_DRK` (or XMR equivalent pre-mainnet)
- Note: with Option A, the slashing evidence model changes.  An oracle that
  hands out an attest_key for a goal that wasn't met can be slashed if the
  bondholder publishes the attest_key (which they received privately) together
  with signed confirmation that the goal data didn't meet the threshold.
  This needs to be specced in `docs/decisions.md` as part of W1.

**5c — Stake management** *(2 days)*
- `oracle stake deposit <amount>` — lock XMR in multi-sig (or stub for pre-mainnet)
- `oracle stake withdraw` — unlock after all active bonds in challenge window
- `oracle stake status` — show staked amount, at-risk bonds
- `oracle dashboard` — live terminal view (bond list, attestation count, fees)

**5d — Challenge window and dispute** *(1 day)*
- Enforce `challenge_period_secs` before `FinaliseVerification` commits
- Allow `ChallengeAttestation` during window
- After window: state transition is permanent

---

### W6 — Order Book and Trading  *(14 days)*
*Blocks: W8 (network launch requires live trading).*
*Depends on: W3 (real ZK transfer proofs needed for settlement).*

This is Phase 3.  It is the largest single unbuilt component.

**6a — Order book contract** *(3 days)*
- `order_book.rs` in `nyxforge-contract`:
  - `PlaceOrder { bond_id, side, price, quantity, commitment, expiry }`
  - `CancelOrder { order_id, nullifier }`
  - `MatchOrders { bid_id, ask_id, transfer_proof_bond, transfer_proof_drk }`
- Orders stored in `orders.db` (sled or rocksdb); gossiped to all peers
- On-chain order book: orders are public (price/qty visible); identity hidden
  via ZK commitment to ownership

**6b — Atomic swap settlement** *(4 days)*
- A matched trade requires two simultaneous TRANSFER proofs:
  - Seller → buyer: bond note transfer
  - Buyer → seller: DRK/XMR payment transfer
- Both nullifiers must be unspent; accept both or reject both
- Settlement contract records new commitments, marks nullifiers spent

**6c — Order matching engine** *(3 days)*
- Price-time priority: best bid meets best ask
- Local matching: each node tries to match; first valid broadcast wins
- Conflict resolution: canonical ordering by block height + message hash
- `market.orders { bond_id }` RPC — list current order book
- `market.history { bond_id, from, to }` RPC — trade price history

**6d — Primary market (note distribution)** *(2 days)*
The roadmap has issuance (MINT) and secondary trading (TRANSFER) but no
spec for how the issuer gets notes to initial buyers.  Minimal v1 approach:
- Issuer holds all minted notes initially
- Issuer places ask orders at desired price; buyers place bids
- Standard order book handles initial distribution
- No Dutch auction or special primary market mechanics needed for v1

**6e — Market order support** *(1 day)*
- IOC (immediate-or-cancel): fill against best N orders; cancel rest
- GTC (good-till-cancelled): persistent until cancelled or expired

**6f — Integration tests** *(1 day)*
- Two local nodes trade a bond note anonymously
- TRANSFER proof verifier rejects tampered quantity
- Atomic swap: both legs succeed or neither does

---

### W7 — Browser UI  *(14 days)*
*Blocks: W8 (launch requires full browser UI).*
*Depends on: W3 (WASM proofs), W4 (goal encryption UI).*

This is Phase 2.  The scaffold exists; everything else is unbuilt.

**7a — Bond browser** *(3 days)*
- List all bonds with filters: state, deadline, data_id, keyword search
- Bond detail page:
  - Goal spec (decrypted if view key provided)
  - Oracle list and acceptance status
  - Current state and deadline
  - Price history chart (line, last 30 days of trades)
- View key entry field on detail page (decrypts private goals inline)

**7b — Bond creation wizard** *(2 days)*
- Port CLI wizard to Flutter multi-step form
- AI-assisted `bond explore` flow integrated (calls MCP server)
- Shows required collateral in XMR
- Goal visibility toggle (private / world)
- Multi-criterion support (the Vec<GoalSpec> we already have)

**7c — Wallet UI** *(2 days)*
- XMR address, spend key (blurred, toggle to reveal)
- Bond note balances by bond series
- XMR balance (from monerod)
- Key import / recovery from spend key

**7d — Portfolio and positions** *(2 days)*
- Table: bond title, quantity, current market price, unrealised P&L
- Redeemable bonds flagged with "Redeem" action button
- Redemption flow: fetch attest_key from oracle → generate BURN proof
  in-browser (WASM) → submit to node

**7e — Order entry and order book** *(2 days)*
- Bid / ask entry form for any active bond
- Live order book display (best 10 bids and asks)
- Last trade price, 24h volume
- "Trade" button: generates TRANSFER proof in-browser

**7f — Mining and node status** *(1 day)*
- Thread count selector, start/stop
- Live hashrate, share count, XMR balance
- Peer count, sync status, oracle daemon status

**7g — De-googled production build** *(1 day)*
- `scripts/build-web.sh`: no CDN URLs, CanvasKit self-hosted
- WASM bundle < 5 MB, < 2 MB after `wasm-opt -Oz`
- Playwright E2E suite: bond create → oracle approve → issue → redeem

**7h — Accessibility and mobile** *(1 day)*
- Readable on mobile (Flutter responsive layout)
- No JavaScript-only dependencies; works with CanvasKit
- Basic keyboard navigation

---

### W8 — Collateral Mechanism and Network Launch  *(10 days)*
*Blocks: Phase 5.*
*Depends on: W2–W7 complete, Decision 1 resolved.*

**8a — Collateral spec** *(2 days, writing + design)*
Write `docs/collateral-mechanism.md`:
- For v1: XMR held in 2-of-3 multi-sig between issuer + two designated
  oracle operators who act as collateral custodians
- Release on goal met: oracles co-sign an XMR transaction to the redeemer's
  stealth address when a valid BURN proof is presented to them
- Release on expiry: issuer submits expiry proof after deadline; oracles
  countersign return to issuer
- This is centralised-ish and disclosed; replace with DRK DLC in Phase 6
- Explicit security disclaimer: custodian oracle collusion could steal funds;
  mitigation is reputation + stake slashing

**8b — XMR multi-sig integration** *(4 days)*
- Monero 2-of-3 multi-sig key setup in `nyxforge-wallet`
- Bond issuance: generate multi-sig address, wait for XMR deposit confirmation
- Bond activation: advance to Active state once XMR confirmed at required depth
- Redemption: oracle co-signs XMR release to payout address from BURN proof
- Expiry: oracle co-signs XMR return to issuer

**8c — Bootstrap infrastructure** *(2 days)*
- 3 bootstrap nodes with known peer IDs and stable IPs
- DNS seed entry listing bootstrap peers
- `nyxforge.toml` default config points to bootstrap peers
- Bootstrap node `nyxforge-oracle` pre-configured with HTTP-JSON adapters
  for common data sources (HUD, NOAA, WHO)

**8d — Network monitoring dashboard** *(1 day)*
- Public-facing static HTML/JS (no Flutter required)
- Active bonds, total collateral, oracle count, recent trades
- Served from a bootstrap node's public RPC (read-only)

**8e — Launch checklist** *(1 day)*
- cargo audit clean
- cargo clippy -D warnings clean
- 30-day soak on testnet (multi-node, real oracle data, real XMR stagenet)
- oracle operator onboarding: ≥ 3 operators pre-registered
- user documentation published

---

### W9 — Security Audit Prep  *(parallel track, 5 days total)*
*Can start after W3 and W6 complete.*
*Required before Phase 5 launch.*

- `cargo audit` — resolve any vulnerable deps
- `cargo clippy --all-targets -- -D warnings` — clean
- `cargo fuzz` targets for:
  - RPC JSON deserialization (invalid inputs)
  - ZK circuit input parsing (malformed witnesses)
- `proptest` property tests for `GoalMetric` evaluation
- `docs/threat-model.md` (one page: assets, adversaries, mitigations)
- Engage external auditor after W6 (order book) is stable
- Estimated audit cost: $70k–$140k (see roadmap Phase 7)

---

## Part 5 — Sequence and Dependencies

```
W1 Spec alignment (3d)
  │
  ├─► W2 Oracle key registration (4d)
  │     │
  │     ├─► W5 Oracle ecosystem (8d)
  │     │
  │     └─► [end-to-end BURN test]
  │
  ├─► W3 Complete Phase 1 ZK (5d)
  │     │
  │     └─► W7 Browser UI (14d) ──────────────────────────────────────┐
  │                                                                    │
  ├─► W4 Goal text encryption (4d)                                     │
  │     │                                                              │
  │     └─► W7                                                         │
  │                                                                    │
  └─► W6 Order book and trading (14d) ─────────────────────────────────┤
        │                                                              │
        └─► W9 Security audit prep (parallel after W6)                │
                                                                       │
                             W8 Collateral + launch (10d) ◄───────────┘
                             (requires W2–W7 all done)
```

W1 first, always.  W2 and W3/W4 can run in parallel after W1.  W6 and
W7 can run in parallel after their respective deps.  W8 is last.

---

## Part 6 — Rough Timeline

Assuming one full-time developer with occasional help:

| Package | Duration | Cumulative |
|---|---|---|
| W1 Spec alignment | 3 days | week 1 |
| W2 Oracle key registration | 4 days | week 2 |
| W3 Phase 1 ZK completion | 5 days | week 3 |
| W4 Goal text encryption | 4 days | week 4 |
| W5 Oracle ecosystem | 8 days | weeks 5–6 |
| W6 Order book | 14 days | weeks 7–9 |
| W7 Browser UI | 14 days | weeks 10–12 (parallel with W6) |
| W9 Audit prep | 5 days | week 13 (parallel) |
| W8 Collateral + launch | 10 days | weeks 14–15 |
| External security audit | 4–8 weeks | weeks 16–23 |
| Audit finding remediation | 2–3 weeks | weeks 24–26 |
| 30-day testnet soak | 4 weeks | weeks 27–30 |

**Rough estimate: 7–8 months to Phase 5 launch** (assuming no DarkFi dependency,
no major architectural changes triggered by the audit, and the two decisions
above are resolved quickly).

This is without external funding or team expansion.  With a second developer
splitting W6 and W7, the critical path compresses to ~4–5 months.

---

## Part 7 — Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Decision 1 (payout currency) not resolved quickly | High | Blocks W8, W7 redemption flow, W5 slashing model | Schedule a decision session; force a choice; document it |
| DarkFi testnet API changes break Phase 6 integration | High | Phase 6 slips | Phase 6 is parallel and not on v1 critical path |
| Security audit finds critical ZK circuit soundness bug | Medium | Delays launch, expensive rework | Invest in W9 audit prep; consider interim code review from ZK specialist |
| TRANSFER Merkle gap disclosed by external researcher before audit | Low-Medium | Reputational; need emergency patch | Publish `docs/security-limitations.md` proactively; this controls the narrative |
| Oracle collusion steals collateral (v1 XMR multi-sig) | Low | Fund loss | Disclose the trust model explicitly; limit bond sizes until DLC integration |
| Performance: Halo2 BURN proof generation in browser is too slow | Medium | Breaks W7 redemption UX | Profile early (W3c); if > 30s in browser, move proof generation to background worker or offer CLI-only redemption for v1 |
| P2P consensus divergence (two nodes disagree on bond state) | Medium | Double-spend or stuck bond | Define canonical conflict resolution rules before W8 |

---

## Part 8 — Deferred to Post-v1 Roadmap

These items should be added to the roadmap after Phase 5 launches:

| Item | Phase |
|---|---|
| DarkFi testnet → mainnet migration | Phase 6/8 |
| Noise-BURN anonymity set (if re-evaluation concludes it's needed) | Phase 6 |
| DLC/DLEQ XMR collateral (removes multi-sig trust) | Phase 6 |
| TRANSFER Merkle circuit membership proof | Phase 6 |
| Multi-currency collateral plugins (BTC, ETH, AR, AO, Zano) | Phase 6+ |
| Kleros / UMA oracle dispute tier | Phase 7+ |
| zkTLS / DECO data attestation | Phase 7+ |
| Long Now endowment mechanics | Phase 7+ |
| Post-quantum verifier slots | Phase 8+ |
| NYX token fair launch | Phase 5 launch or shortly after |
| Cake Wallet / RetoSwap / Vexl liquidity integration | Phase 5 stretch |
