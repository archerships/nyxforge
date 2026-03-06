# NyxForge — Product Specification

> Version 0.1 — living document — March 2026

---

## Table of Contents

1. [Vision & Mission](#1-vision--mission)
2. [Problem Statement](#2-problem-statement)
3. [Target Users](#3-target-users)
4. [Product Overview](#4-product-overview)
5. [Core Concepts](#5-core-concepts)
6. [Functional Requirements](#6-functional-requirements)
7. [Non-Functional Requirements](#7-non-functional-requirements)
8. [Bond Lifecycle](#8-bond-lifecycle)
9. [Oracle Network](#9-oracle-network)
10. [Market Mechanics](#10-market-mechanics)
11. [Wallet & Key Management](#11-wallet--key-management)
12. [Mining Integration](#12-mining-integration)
13. [AI Assistance (MCP)](#13-ai-assistance-mcp)
14. [Privacy Model](#14-privacy-model)
15. [Technical Architecture](#15-technical-architecture)
16. [Non-Goals](#16-non-goals)
17. [Success Metrics](#17-success-metrics)
18. [Roadmap](#18-roadmap)
19. [Glossary](#19-glossary)

---

## 1. Vision & Mission

**Vision:** A world where the probability of any measurable social outcome has a
publicly traded market price — creating continuous, unincentivised pressure on
governments, companies, and individuals to produce results rather than activity.

**Mission:** Build the infrastructure that makes anonymous, permissionless,
outcome-linked social finance possible — without trusting any institution,
exposing any identity, or requiring permission from any authority.

NyxForge does for social outcomes what prediction markets did for forecasting
events: it aggregates distributed knowledge into a price signal, and it ties
capital flows directly to verified results rather than to promised intentions.

---

## 2. Problem Statement

### 2.1 Misaligned incentives

Governments and NGOs are funded for *activity*, not *results*.  Budgets are
allocated to programmes, headcount, and reports.  Money flows whether or not the
underlying goal is ever achieved.  There is no mechanism that automatically
redirects capital toward more effective interventions or penalises persistent
failure.

### 2.2 No price signal

No one can look up the market's best estimate of whether homelessness will fall
below 50,000 by 2030, or whether global CO₂ will return to 350 ppm by 2045.
Without a price, capital cannot efficiently allocate.  Philanthropists choose
programmes by reputation and relationships rather than by aggregated evidence.

### 2.3 Surveillance

Every existing on-chain experiment in social finance has required publishing
donor identities, holding amounts, and transaction histories.  This chills
participation from:

- Privacy-conscious individuals who do not want their charitable giving analysed
- Dissidents and activists in jurisdictions where funding certain goals is
  politically dangerous
- Institutional actors with legal constraints on public disclosure of positions

### 2.4 Trust requirements

Existing social impact bond programmes require trusting the issuer to lock
collateral, the oracle to measure results honestly, and the settlement agent to
pay out on time.  Each trust assumption is a failure point and a censorship
vector.

---

## 3. Target Users

### 3.1 Impact investors

Individuals and funds seeking outcome-linked financial exposure.  They want to
hold a position whose value increases as a measurable social goal approaches
completion, without taking on programme execution risk or trusting any single
implementing organisation.

### 3.2 Philanthropists and foundations

Donors who want verifiable return on social spending.  NyxForge lets them lock
collateral against an outcome and reclaim it if the goal is not met — converting
a grant into a conditional commitment.

### 3.3 Speculators

Participants willing to price the probability of social change.  Their trading
activity provides the price signal that makes the market useful to all other
participants.

### 3.4 Oracle operators

Technical actors who run data adapters, monitor real-world metrics, and post
signed attestations to the P2P network.  They earn fees proportional to their
stake and attestation accuracy.

### 3.5 Privacy-conscious participants

Any of the above who cannot or will not publish their identity, holdings, or
transaction history.  NyxForge is the only venue where any of these roles can
be played anonymously by default.

---

## 4. Product Overview

NyxForge is a decentralised, anonymous market for social policy bonds — financial
instruments that pay out only when a measurable real-world goal is achieved.

The core loop:

1. An **issuer** defines a goal with a verifiable metric (data ID, operator,
   threshold, deadline) and locks collateral equal to `total_supply ×
   redemption_value` in a supported collateral currency (DRK, XMR, BTC, ETH, or
   others).  The locking mechanism depends on the chain.
2. The collateral is locked by a smart contract on DarkFi L1.  Bond notes
   representing ownership of a share of that collateral are minted as ZK notes.
3. Notes trade on an anonymous order-book DEX.  The market price at any moment
   reflects the crowd's live estimate of the probability that the goal will be
   met before the deadline.
4. **Oracle** operators independently monitor the goal metric and post signed
   attestations.  Once a quorum of consistent attestations accumulates, the bond
   finalises.
5. If the goal is met, note holders submit ZK burn proofs and receive DRK
   payouts, anonymously.  If the deadline passes without the goal being met, the
   issuer reclaims the collateral and notes expire worthless.

No company, server, or custodian mediates any step.  Every user runs a local
node.

---

## 5. Core Concepts

### 5.1 Social policy bond

A financial instrument whose redemption is conditional on a measurable social or
environmental outcome being achieved by a specified date.  The instrument
converts vague philanthropic intent into a precise, verifiable commitment with
market-priced probability attached.

The term was coined by Ronnie Hoban (1988).  NyxForge implements the concept
natively on a ZK L1 chain with no trusted intermediary.

### 5.2 GoalSpec

The machine-readable definition of a bond's outcome target:

```
GoalSpec {
    title            : human-readable name
    description      : prose explanation
    metric: GoalMetric {
        data_id      : dot-separated data source identifier
        operator     : lt | lte | gt | gte | eq
        threshold    : Decimal
        aggregation  : Option<String>   // e.g. "annual_mean"
    }
    deadline         : DateTime<Utc>
    evidence_format  : Option<String>
}
```

The `data_id` is the contract between the issuer and the oracle operators.  It
identifies which data series will be fetched and how the result will be
interpreted.  Bond issuance fails (in production mode) if no registered oracle
has an adapter for the given `data_id`.

### 5.3 Bond note

An anonymous ZK commitment representing ownership of one or more units of a bond
series.  Bond notes behave like Zcash sapling notes: they are held privately in
a local wallet, transferred via ZK proof, and spent by revealing a nullifier.

```
Note {
    bond_id          : [u8; 32]
    quantity         : u64
    redemption_value : Amount
    owner            : PublicKey
    randomness       : Scalar
    serial           : Scalar
}

Commitment = Poseidon2(Poseidon2(bond_id, quantity), Poseidon2(owner_pk, randomness))
Nullifier  = Poseidon2(owner_secret, serial)
```

Note: `payout_address` is NOT part of the note — it is provided at redemption time (in `BurnWitness`).

### 5.4 Oracle attestation

A signed statement by a registered oracle operator asserting whether a bond's
goal was met at the time of evaluation.  Attestations are gossiped on the P2P
network.  Once `quorum` matching attestations accumulate, the bond finalises.

### 5.5 DRK

The native token of DarkFi L1.  Used as:
- Bond collateral
- Redemption payout currency
- Oracle stake
- Market trading currency

DRK is anonymous by construction (ZK notes, same model as bond notes).  DRK is used for trading, oracle staking, and fees.  Bond collateral may alternatively be locked in XMR, Zano, BTC, ETH, AR (Arweave), AO (AO Computer), or other supported currencies via the CollateralManager plugin system.

### 5.6 Supported collateral currencies

Bond collateral can be locked in any supported currency. The locking mechanism varies by chain:

| Currency | Lock mechanism | Privacy |
|---|---|---|
| DRK | DarkFi smart contract (native) | Full ZK privacy |
| XMR | DLEQ adaptor signatures | Full privacy (RingCT + stealth addresses) |
| Zano | DLEQ adaptor signatures | Full privacy (RingCT + stealth addresses) |
| BTC / ZEC | Taproot / Schnorr DLC + timelock | Pseudonymous |
| ETH / ERC-20 | EVM smart contract + oracle signatures | Pseudonymous |
| AR (Arweave) | Warp/SmartWeave contract escrow | Pseudonymous |
| AO (AO Computer) | AO Process (actor) escrow | Pseudonymous |

The issuer specifies the currency and amount in `CollateralSpec` at bond creation.  The `return_address` field specifies where collateral is returned if the goal is not met; the `payout_address` is specified by each holder at **redemption time** (in `BurnWitness`), not at issuance.

XMR is also integrated as a **mining revenue source**: the built-in RandomX / P2Pool miner earns XMR that can be converted to DRK or used directly as bond collateral.

### 5.7 Oracle trust model — attestors, not custodians

Oracles **attest to real-world outcomes**; they do not control fund routing.  Two outcomes are pre-committed at bond setup:

- **`s_met`**: oracle scalar proving "goal was achieved" — reveals the spending key for the payout collateral output
- **`s_fail`**: oracle scalar proving "goal failed / expired" — reveals the spending key for the refund output

Publishing *either* scalar resolves exactly one output.  Publishing *both* scalars would reveal the oracle's private key — this is **cryptographically enforced**, not trust-based.  Oracles cannot redirect funds to themselves because no output is addressed to any oracle key.

The enforcement mechanism varies by chain:
- **XMR / Zano**: DLEQ (Discrete Log Equality) proofs — same primitive as XMR-BTC atomic swaps
- **BTC / ZEC**: Discreet Log Contracts (DLC) with Schnorr/Taproot
- **ETH / ERC-20**: EVM smart contract enforces payout routing on-chain
- **AR (Arweave)**: Warp/SmartWeave contract verifies oracle signature and releases AR
- **AO (AO Computer)**: AO Process (actor) holds tokens; oracle message triggers release
- **DRK**: DarkFi smart contract; oracle posts signed quorum result

For chains requiring a two-phase claim (XMR, BTC):
1. Oracle publishes `s_met` → unlocks an intermediate collateral output
2. The current bond holder presents a **BURN proof** → sweeps the intermediate to their `payout_address` (specified at burn time)

Neither the oracle nor an outside observer can complete both phases.

---

## 6. Functional Requirements

### 6.1 Bond management

| ID | Requirement |
|----|-------------|
| F-B01 | Issuer can define a GoalSpec via CLI wizard or web UI |
| F-B02 | AI assistant (MCP) can identify similar existing bonds and draft a new GoalSpec |
| F-B03 | Bond can be published in `Proposed` state for community review before collateral is locked |
| F-B04 | Any network participant can post a comment on a `Proposed` bond |
| F-B05 | Issuer can read all comments on their bond |
| F-B06 | Bond can be submitted to listed oracles for acceptance before going live |
| F-B07 | Each oracle must explicitly accept or reject a bond before it moves to `Draft` |
| F-B08 | If any oracle rejects, issuer can revise the oracle list (clearing all prior responses) |
| F-B09 | Once all oracles accept (`Draft` state), issuer can lock collateral and activate the bond |
| F-B10 | Bond issuance (in production mode) must fail if the `data_id` cannot be verified against a registered oracle adapter |
| F-B11 | Bond details are readable by any node participant |
| F-B12 | Bond list is browsable by state, deadline, and data ID |

### 6.2 Trading

| ID | Requirement |
|----|-------------|
| F-T01 | Any wallet holder can place a bid or ask order for any active bond |
| F-T02 | Order matching produces a ZK transfer proof; no on-chain link between buyer and seller |
| F-T03 | Orders are gossiped on the P2P network; matching is done locally |
| F-T04 | Order book is visible to any node participant (bids, asks, last trade price) |
| F-T05 | Market price history is recorded and browsable |

### 6.3 Oracle

| ID | Requirement |
|----|-------------|
| F-O01 | Oracle operators register with a stake deposit and a DRK public key |
| F-O02 | Oracle nodes fetch data via pluggable adapters keyed to `data_id` |
| F-O03 | Oracle nodes post signed attestation scalars (`s_met` or `s_fail`) to the P2P network; these scalars act as adaptor signatures that unlock collateral for the appropriate party |
| F-O04 | Once `quorum` matching attestations accumulate, the contract finalises the bond |
| F-O05 | A configurable challenge window allows dispute before finalisation |
| F-O06 | Fraudulent oracle attestation triggers slashing of `slash_fraction` of staked DRK |
| F-O07 | Oracle fees are distributed proportionally after successful bond settlement |

### 6.4 Redemption & settlement

| ID | Requirement |
|----|-------------|
| F-R01 | Bond note holder generates a ZK burn proof to redeem a `Redeemable` bond |
| F-R02 | Burn proof reveals no information about the redeemer's identity |
| F-R03 | Payout is a fresh anonymous DRK note; no on-chain link to the burned bond note |
| F-R04 | If goal not met, issuer reclaims collateral via `ClaimExpiredCollateral` transaction |
| F-R05 | Expired bond notes cannot be redeemed |
| F-R06 | For chain-specific collateral, bond holder completes a two-phase claim: oracle's `s_met` attestation unlocks an intermediate output; holder's BURN proof sweeps it to their `payout_address` |
| F-R07 | The `payout_address` is specified by the current holder at redemption time (in `BurnWitness`), not locked at bond issuance |

### 6.5 Wallet & keys

| ID | Requirement |
|----|-------------|
| F-W01 | User can generate a new XMR + DRK keypair locally |
| F-W02 | User can recover the full wallet from an XMR spend key |
| F-W03 | DRK keys are derived deterministically from the XMR spend key |
| F-W04 | Wallet keys never leave the local device |
| F-W05 | Wallet supports stagenet and mainnet addresses |
| F-W06 | XMR wallet background scanner monitors the configured monerod instance |

### 6.6 Mining

| ID | Requirement |
|----|-------------|
| F-M01 | Node includes a built-in RandomX CPU miner |
| F-M02 | Miner connects to a Stratum v1 endpoint (P2Pool or compatible server) |
| F-M03 | Number of mining threads is configurable at launch |
| F-M04 | Mining can be started and stopped at runtime via RPC |
| F-M05 | Install scripts provide a one-command setup for stagenet mining (monerod + P2Pool) |

### 6.7 AI assistance (MCP)

| ID | Requirement |
|----|-------------|
| F-A01 | An MCP server provides AI assistance over JSON-RPC 2.0 over HTTP |
| F-A02 | The MCP server supports Anthropic, OpenAI, Ollama, and any OpenAI-compatible endpoint |
| F-A03 | Users can add, remove, and configure AI providers via CLI or REST API |
| F-A04 | The `bond_assist` tool accepts a plain-language goal description and existing bonds; returns similar bonds, a drafted GoalSpec, and analysis notes |
| F-A05 | The CLI prefers the MCP server for AI; falls back with a clear error if not running |

### 6.8 Node & P2P

| ID | Requirement |
|----|-------------|
| F-N01 | Node exposes a JSON-RPC API on localhost for local clients |
| F-N02 | Node participates in a libp2p gossip network for bond, order, trade, and attestation propagation |
| F-N03 | Node uses Kademlia DHT for peer discovery |
| F-N04 | All P2P transport is encrypted (Noise XX) |
| F-N05 | Node data is persisted to a local directory; re-runs are safe |

### 6.9 Collateral management

| ID | Requirement |
|----|-------------|
| F-C01 | Bond issuer specifies collateral currency and amount via `CollateralSpec` |
| F-C02 | Each supported currency has a `CollateralManager` plugin implementing `publish_attestation`, `claim_payout`, and `claim_refund` |
| F-C03 | For XMR/Zano collateral: oracle panel performs pre-issuance DLEQ nonce commitment; issuer sends XMR to the two pre-committed outputs |
| F-C04 | For BTC/ZEC collateral: oracle panel generates pre-committed DLC outcome transactions |
| F-C05 | For ETH/ERC-20 collateral: issuer deposits to a verifiable escrow contract address |
| F-C06 | For AR collateral: issuer deposits to a Warp/SmartWeave escrow contract; oracle signature triggers release |
| F-C07 | For AO collateral: issuer deposits to an AO Process actor; oracle message triggers release |
| F-C08 | Collateral is verifiable by any observer before bond activation (proof-of-lock) |
| F-C07 | Timelock fallback: if goal deadline passes without oracle attestation, `return_address` can sweep collateral without oracle cooperation |

---

## 7. Non-Functional Requirements

### 7.1 Privacy

- All bond ownership is anonymous by default.
- No user account, email, KYC, or IP binding is required at any level.
- Keys are generated and stored locally; they are never transmitted to any server.
- Trade counterparties are unlinkable on-chain.
- Redemption amounts are private.

### 7.2 Decentralisation

- Every user runs a full local node.
- There is no central relay, matching engine, oracle aggregator, or settlement server.
- Bootstrap peers are provided but the network operates without them once connected.

### 7.3 Censorship resistance

- No address can be blacklisted from issuing, trading, or redeeming.
- No bond topic is restricted; only verifiability of the goal metric is required.
- Oracle set is chosen by the issuer; any operator can run an oracle.

### 7.4 Correctness

- Collateral amounts are verified by smart contract before bond activation.
- Conservation of bond note quantity is enforced by ZK transfer proofs.
- Double-spend is prevented by the on-chain nullifier set.
- Oracle quorum requirements are enforced by contract, not by the oracle software.

### 7.5 Security

- ZK circuits use Halo2 (transparent, no toxic waste trusted setup).
- Oracle stake slashing deters fraudulent attestation.
- Challenge window allows dispute before finalisation.
- DAO override is opt-in per bond (default off).

### 7.6 Portability

- All user-facing software (node, CLI, WASM frontend) is compiled from Rust
  source with no proprietary dependencies.
- The system runs on Linux, macOS, and (eventually) Windows.
- The WASM frontend runs in any modern browser without installation.

### 7.7 Performance

- Bond creation (excluding collateral lock) completes in under 5 seconds on a
  modern laptop.
- ZK proof generation (MINT / TRANSFER / BURN) completes in under 30 seconds on
  a modern laptop CPU (single-threaded).
- P2P gossip propagation reaches all connected peers within 10 seconds under
  normal conditions.

---

## 8. Bond Lifecycle

The full state machine and lifecycle events are documented in
[bond-lifecycle.md](bond-lifecycle.md).

### State summary

| State | Description |
|-------|-------------|
| `Proposed` | Published for community review; no collateral locked, no trading |
| `PendingOracleApproval` | Submitted to listed oracles; awaiting acceptance |
| `Draft` | All oracles accepted; collateral not yet locked |
| `Active` | Collateral locked; notes minted; trading live |
| `Redeemable` | Oracle quorum reached; goal verified as met |
| `Expired` | Deadline passed; goal not met; notes worthless |
| `Settled` | All reachable notes redeemed |

For bonds using XMR, BTC, or other chain-specific collateral, a **Phase 0** pre-setup step occurs before issuance: the oracle panel generates DLEQ/DLC nonce commitments for the two pre-committed collateral outcomes. This happens off-chain before `bonds.issue` is called.

### Transitions

```
[create] → Proposed (optional)
         → PendingOracleApproval
         → Draft (all oracles accept)
         → Active (bonds.issue; collateral locked)
         → Redeemable (oracle quorum, goal met)
         → Settled
         → Expired (deadline passed, goal not met) → [issuer reclaims collateral]
```

Any rejection by an oracle while in `PendingOracleApproval` returns the bond to
the issuer for revision.  The issuer may revise the oracle list (via
`bonds.revise_oracles`), which clears all existing responses and requires all
listed oracles to re-accept from scratch.

---

## 9. Oracle Network

### 9.1 Role

Oracle operators are the link between on-chain bond contracts and real-world data. They are **attestors** — they sign statements about real-world outcomes — but they are **not custodians** of collateral. Fund routing is enforced cryptographically by the lock mechanism (DLC, DLEQ, or smart contract), not by oracle honesty. An oracle's only power is to choose which of the two pre-committed outcomes (`s_met` or `s_fail`) to publish. Publishing both is cryptographically impossible without revealing the oracle's private key.

### 9.2 Registration

An oracle operator must:

1. Hold a DRK keypair that they register with the bond issuer.
2. Stake at least `required_stake` DRK (per bond or globally, TBD).
3. Run the `nyxforge-oracle` daemon with a data adapter for the bond's `data_id`.
4. Explicitly accept each bond they are listed on (via `oracle-accept` command or
   the oracle daemon's auto-accept policy).

### 9.3 Attestation flow

1. The oracle daemon monitors active bonds and their deadlines.
2. At evaluation time (typically when the deadline is near or a new data release
   is available), the daemon fetches the data via the registered `DataSource` adapter.
3. It evaluates the `GoalMetric` predicate: `fetched_value OPERATOR threshold`.
4. It produces a signed attestation scalar (`s_met` or `s_fail`) using its DRK private key.
5. The attestation scalar is gossiped to the P2P network and — for chain-specific collateral — is used as the adaptor signature that unlocks the appropriate collateral output.
6. Other nodes accumulate attestations.  Once `attestation_threshold` matching
   attestations from the listed oracle set are recorded, any peer can call
   `FinaliseVerification`.
7. A `challenge_period_secs` window follows before the result is committed.

### 9.4 Slashing

A fraudulent attestation — one where the oracle attests `goal_met: true` but the
oracle consensus or a DAO override later determines the goal was not met, or vice
versa — results in `slash_fraction × staked_DRK` being burned from that oracle's
stake.  The remainder of the stake is returned.

The slash mechanism relies on DAO governance to adjudicate disputes after the
challenge window.  In bonds where `dao_override_allowed = false`, no post-hoc
override is possible.

### 9.5 Data adapters

Data adapters are Rust traits implementing:

```rust
trait DataSource {
    fn data_id(&self) -> &str;
    async fn fetch(&self) -> Result<Decimal>;
}
```

Built-in adapters (planned):
- HTTP-JSON: fetch a URL, extract a field with a JSONPath expression
- IPFS: retrieve a pinned document
- Mock: return a configurable test value

Custom adapters can be compiled into the oracle daemon or loaded as WASM plugins
(not yet implemented).

---

## 10. Market Mechanics

### 10.1 Dutch auction for initial bond sales

When a bond is activated (`bonds.issue`), it enters a **Dutch (descending-clock)
auction** for its initial sale period.  The ask price starts at `start_price`
and falls linearly to `reserve_price` over `duration_secs` seconds (configured
at issuance).  After the window closes the price stays permanently at
`reserve_price`.

```
price(t) = start_price - (start_price - reserve_price) × min(t, duration) / duration
```

The current ask price is available via the `bonds.auction_price` RPC endpoint.
Buyers call `bonds.buy` to purchase bonds at the live auction price.

**Supply tracking:** `bonds_remaining` is set to `total_supply` at activation
and decremented atomically on each purchase.  When `bonds_remaining` reaches
zero no further buys are accepted.

### 10.2 Price discovery (secondary market)

Bond notes trade on a local order-book DEX after the initial sale.  The market
price at any moment is the intersection of the best bid and ask.

The theoretical fair price is:

```
price ≈ redemption_value × P(goal met before deadline)
```

As the deadline approaches, `P` is constrained by available time and current
metric progress.  Price tracks the crowd's live estimate of the remaining
probability.  Secondary trading is independent of the Dutch auction price.

### 10.3 Order types

Phase 1 (initial implementation):

- **Limit order** — buy or sell at a specified price or better.

Phase 2 (planned):

- **Market order** — buy or sell immediately at the best available price.
- **Time-in-force** — GTC (Good 'til Cancelled), IOC (Immediate or Cancel), FOK
  (Fill or Kill).

### 10.4 Trade settlement

Every matched trade requires two ZK proofs generated by the respective parties:

- A `TRANSFER` proof from the seller: proves ownership of the bond note being sold
  and transfers it to the buyer.
- A payment `TRANSFER` proof from the buyer: proves ownership of the DRK being
  paid and transfers it to the seller.

Both proofs are submitted atomically.  Both nullifiers are checked for unspent
status.  If either nullifier has been spent, the transaction fails.

The on-chain state records the nullifiers and the new note commitments; no
information about buyer, seller, quantity, or price is recorded beyond what is
necessary to prevent double-spend.

### 10.5 Fees

A protocol-level fee (TBD, e.g. 0.1% of trade value) may be levied at the
contract level and directed to a protocol treasury DAO.  Fee parameters are set
by governance.

---

## 11. Wallet & Key Management

### 11.1 Key derivation

A NyxForge wallet consists of two key pairs:

**XMR keypair** (Monero-compatible)
- Spend key: a canonical Ed25519 scalar (32 bytes, `bytes[31] &= 0x0f`)
- View key: `keccak256(spend_key)` reduced to canonical form
- Address: derived from spend key and view key for the configured network
  (stagenet or mainnet)

**DRK keypair** (DarkFi-compatible)
- Secret: `blake3("nyxforge-drk:" ‖ spend_key_bytes)` — deterministic derivation
- Pubkey: Ed25519 public key of the DRK secret
- Re-derivable from the XMR spend key at any time

Because the DRK key is derived from the XMR spend key, the single XMR spend key
is the sole recovery secret for the entire wallet.

### 11.2 Storage

Keys are stored in the node's data directory (`node_data/`) as
encrypted-at-rest files.  The encryption key is derived from a user-supplied
passphrase (not yet implemented; currently stored in plaintext for development).

**Keys never leave the local machine.**  The browser WASM frontend generates and
holds keys in `localStorage`; the node binary holds keys in the data directory.
Neither communicates keys over any network interface.

### 11.3 Recovery

The XMR spend key is displayed (once) when a wallet is created.  Wallet recovery
from a spend key:

```bash
./nyxforge --wallet-key <64-hex-spend-key>
```

or via RPC:

```bash
nyxforge-cli wallet import --spend-key <hex>
```

### 11.4 Network selection

The wallet address format is network-dependent:

| Network | Address prefix | RPC port |
|---------|---------------|----------|
| Stagenet | `5…` | 38081 |
| Mainnet | `4…` | 18081 |

The node's `--testnet` flag and the wallet's `Network` enum must agree.  The
default in the current prototype is stagenet.

---

## 12. Mining Integration

### 12.1 Purpose

Mining is a built-in mechanism for users to earn XMR (Monero) without a
centralised exchange.  Earned XMR can be converted to DRK for bond collateral
and oracle staking.

### 12.2 Stack

```
nyxforge-node
  ├── nyxforge-miner (RandomX CPU miner)
  │     └── Stratum v1 TCP client
  └── connects to → P2Pool (local)
                      └── connects to → monerod (local, stagenet)
```

The NyxForge miner implements the Stratum v1 protocol and connects to a P2Pool
node.  P2Pool in turn participates in the Monero P2P mining network.  This means
the user mines as part of a decentralised pool without a centralised pool operator
that knows their identity or controlling their payout address.

### 12.3 Configuration

| Flag | Default | Description |
|------|---------|-------------|
| `--mine-on-start` | false | Begin mining immediately at node launch |
| `--mine-threads N` | 1 | CPU threads for RandomX |
| `--p2pool-url host:port` | `127.0.0.1:3333` | Stratum endpoint |
| `--xmr-node url` | `http://127.0.0.1:38081` | monerod JSON-RPC |

### 12.4 Install script

`scripts/install-stagenet-mining.sh` performs a one-command setup:

- Installs `monerod` via Homebrew
- Downloads the correct P2Pool binary from GitHub releases for the current
  architecture (arm64 / x86_64)
- Creates `~/.config/nyxforge/stagenet/config.env`
- Generates `scripts/stagenet-mining.sh` (management script)
- Optionally installs a launchd plist to auto-start monerod on login

After installation, `scripts/stagenet-mining.sh start --wallet <address>`
starts the full mining stack.

---

## 13. AI Assistance (MCP)

### 13.1 Purpose

Bond design is non-trivial: choosing the right data ID, operator, threshold,
aggregation method, and deadline requires domain knowledge.  The AI assistant
helps non-experts design well-formed bonds and surfaces existing bonds that
might overlap with their goal.

### 13.2 Architecture

NyxForge uses the **Model Context Protocol (MCP)** — JSON-RPC 2.0 over HTTP —
to communicate with AI providers.  This keeps the AI integration provider-agnostic
and allows users to run any compatible LLM locally or via a cloud API.

```
nyxforge-cli bond explore
    └── McpClient  →  nyxforge-mcp  →  AI provider API
                            ↑
                     provider config
                 (~/.config/nyxforge/mcp.json)
```

### 13.3 Supported providers

| Provider | Auth | Default model |
|----------|------|---------------|
| Anthropic | API key | `claude-opus-4-6` |
| OpenAI | API key | `gpt-4o` |
| Ollama | None (local) | `llama3` |
| Custom | Optional API key | User-defined |

### 13.4 MCP server endpoints

**JSON-RPC (POST /)**

| Method | Description |
|--------|-------------|
| `initialize` | MCP handshake; returns server capabilities |
| `tools/list` | Returns the list of available tools |
| `tools/call` | Invoke a tool (`bond_assist`) |

**REST (provider management)**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/providers` | List configured providers |
| POST | `/providers` | Add a provider |
| DELETE | `/providers/:name` | Remove a provider |
| PUT | `/providers/default` | Set the active provider |
| GET | `/health` | Liveness check |

### 13.5 `bond_assist` tool

Input:
```json
{
  "description": "I want to reduce US unsheltered homelessness to under 100,000 by 2030",
  "existing_bonds": [ { "id": "...", "title": "...", "goal": "..." }, ... ]
}
```

Output (`BondAssistance`):
```json
{
  "similar_bonds": [
    { "bond_id": "...", "title": "...", "similarity": "high", "explanation": "..." }
  ],
  "suggested_bond": {
    "title": "...",
    "description": "...",
    "data_id": "us.hud.pit_count.unsheltered",
    "operator": "lt",
    "threshold": "100000",
    "aggregation": "annual_point_in_time",
    "deadline": "2030-01-01"
  },
  "analysis": "..."
}
```

### 13.6 CLI integration

```bash
# Set up provider (one time)
nyxforge-cli mcp add claude
  Provider type: Anthropic
  API key: sk-ant-…

# Bond design session
nyxforge-cli bond explore
```

After exploring, the user can:
- **Create bond from AI draft** — pre-fills the wizard with the AI's suggestions
- **Back an existing bond** — view market instructions for a similar bond
- **Start fresh** — open the wizard with no pre-fills
- **Cancel**

---

## 14. Privacy Model

### 14.1 What is public

| Data | Visibility |
|------|-----------|
| Bond goal specs, deadlines, data IDs | Public to all nodes |
| Oracle attestation results | Public to all nodes |
| Nullifier set | Public (required for double-spend prevention) |
| Bond state transitions | Public |
| Order book (bids and asks) | Public |
| Collateral lock proofs (txid + DLEQ proof) | Public |

### 14.2 What is private

| Data | Visibility |
|------|-----------|
| Who holds which bonds | Private (ZK note commitments only) |
| Trade counterparties | Private (ZK transfer proofs) |
| Trade quantity and price | Private |
| Wallet balances | Private |
| Redemption amounts | Private (XMR collateral: fully hidden; BTC/ETH: pseudonymous) |
| Issuer identity (if desired) | Private (issuer DRK key not linked to real-world identity) |

### 14.3 ZK proof system

NyxForge uses DarkFi's Halo2-based zkVM for all privacy-preserving proofs.  The
three circuit types are described in detail in [zk-design.md](zk-design.md):

- **MINT** — prove a new note commitment is well-formed without revealing the owner
- **TRANSFER** — prove quantity-conserving ownership transfer
- **BURN** — prove ownership of a redeemable bond note and authorise payout

Halo2 is a transparent proof system (no trusted setup / toxic waste).  All
verifier keys are derived from the circuit; there is no ceremony to trust.

### 14.4 Network privacy

The current prototype uses libp2p with Noise XX encryption for P2P transport.
IP addresses of connecting peers are visible to direct peers.  Future integration
with DarkFi's native P2P (which routes over Tor by default) will address
transport-layer privacy.

---

## 15. Technical Architecture

The full architecture diagram, crate dependency graph, and data flow diagrams are
in [architecture.md](architecture.md).

### 15.1 Crate overview

| Crate | Type | Role |
|-------|------|------|
| `nyxforge-core` | Library | Shared types: Bond, GoalSpec, OracleAttestation, Note, Amount |
| `nyxforge-contract` | Library | Bond lifecycle state machine; ZK proof verification stubs |
| `nyxforge-zk` | Library | Halo2 ZK circuits: MINT, TRANSFER, BURN |
| `nyxforge-oracle` | Binary | Oracle daemon; data adapters; attestation signing |
| `nyxforge-node` | Binary | Local P2P node; JSON-RPC server; state; contract engine |
| `nyxforge-miner` | Library | RandomX CPU miner; Stratum v1 client |
| `nyxforge-wallet` | Library | XMR + DRK key management |
| `nyxforge-cli` | Binary | Interactive bond wizard; full lifecycle management |
| `nyxforge-mcp` | Binary | AI provider bridge (MCP protocol); provider REST API |
| `nyxforge-web` | WASM | Browser frontend (Flutter / wasm-pack) |

A future `nyxforge-collateral` crate (not yet created) will contain per-chain `CollateralManager` implementations.

### 15.2 RPC API (node, port 8888)

Key methods:

| Method | Description |
|--------|-------------|
| `wallet.create` | Generate new keypair; returns XMR address, DRK address, spend key |
| `wallet.import` | Import wallet from XMR spend key |
| `wallet.addresses` | Return current wallet addresses |
| `bonds.issue` | Submit a Bond for issuance |
| `bonds.list` | Return all known bonds |
| `bonds.get` | Return a single bond by hex ID |
| `bonds.propose` | Publish a bond in Proposed state |
| `bonds.comment` | Post a comment on a Proposed bond |
| `bonds.comments` | Retrieve comments on a bond |
| `bonds.submit_for_approval` | Advance bond to PendingOracleApproval |
| `bonds.oracle_accept` | Oracle accepts a bond |
| `bonds.oracle_reject` | Oracle rejects a bond |
| `bonds.revise_oracles` | Replace oracle list (clears responses) |
| `miner.start` | Start mining with optional thread count |
| `miner.stop` | Stop mining |
| `miner.status` | Return mining status and hashrate |

### 15.3 Data store

The node stores all state in a local directory (`node_data/` by default):

- `bonds.db` — all known bonds (sled key-value store)
- `orders.db` — open orders
- `nullifiers.db` — spent nullifier set
- `wallet.json` — encrypted wallet keys

### 15.4 DarkFi L1 integration

Current status: stubbed.  The contract engine calls `nyxforge-contract`
functions in-process.  Full integration requires:

1. Compiling `nyxforge-contract` to DarkFi WASM (`wasm32-unknown-unknown`).
2. Deploying to DarkFi testnet.
3. Replacing in-process state with on-chain state reads/writes via DarkFi SDK.
4. Wiring ZK proof verification to Halo2 verifier (currently stubbed to `true`).

---

## 16. Non-Goals

- **Fiat on-ramp / off-ramp** — NyxForge does not provide, facilitate, or
  integrate with any fiat currency exchange.
- **KYC / AML compliance features** — The system is explicitly designed to be
  anonymous.  No compliance tooling will be added to the core protocol.
- **Custodial wallet** — Keys are always held by the user.  NyxForge will never
  offer a hosted wallet service.
- **Centralised oracle** — There is no official oracle service run by the
  NyxForge developers.  Any participant can run an oracle.
- **Content moderation** — Bond goal specs are not reviewed or filtered by any
  central authority.  Only verifiability of the goal metric is required.
- **Mobile-native apps** — The initial target is desktop browser (WASM).
  Mobile support may follow but is not in scope.
- **Governance of real-world policy** — NyxForge provides a market for pricing
  outcomes and paying for results.  It does not participate in policy-making.

---

## 17. Success Metrics

### 17.1 Protocol health

| Metric | Target (12 months post-mainnet) |
|--------|--------------------------------|
| Active bonds | ≥ 50 |
| Unique oracle operators | ≥ 10 |
| Total collateral locked | ≥ 100,000 DRK |
| Bonds successfully settled (goal met) | ≥ 5 |
| Bonds expired (goal not met, collateral reclaimed) | Measurable (proves the mechanism works) |

### 17.2 Market quality

| Metric | Target |
|--------|--------|
| Average bid-ask spread | < 5% of redemption value |
| Median time to first trade after issuance | < 24 hours |
| Bond price correlation with independent probability estimates | > 0.7 |

### 17.3 Developer adoption

| Metric | Target |
|--------|--------|
| Oracle data adapters contributed | ≥ 5 external |
| AI provider integrations | ≥ 3 (Anthropic, OpenAI, Ollama live) |
| External node deployments | ≥ 20 |

---

## 18. Roadmap

### Phase 0 — Prototype (current)

- [x] Full bond lifecycle state machine (Proposed → Settled)
- [x] CLI bond wizard with all lifecycle commands
- [x] Oracle registration, acceptance workflow, attestation
- [x] XMR wallet (stagenet), key derivation, recovery
- [x] RandomX miner with P2Pool Stratum client
- [x] Node JSON-RPC with P2P scaffold (libp2p gossip)
- [x] MCP server with multi-provider AI support
- [x] AI-assisted bond exploration (`bond explore`)
- [x] Community proposal review (propose / comment / submit)
- [x] ZK proofs wired (Halo2 PLONK; MINT/TRANSFER/BURN; 176 tests passing)
- [ ] Browser WASM UI (Flutter scaffold, splash/navigation only)
- [ ] DarkFi L1 deployment

### Phase 1 — ZK integration (complete)

- [x] Wire MINT / TRANSFER / BURN circuits to real Halo2 proof generation and verification
- [x] Deploy contract to DarkFi testnet
- [x] WASM proof generation in browser
- [x] Full redemption flow end-to-end

### Phase 2 — Market

- Order book contract with atomic swap (ZK bond note + ZK DRK)
- Browser UI: bond browser, order entry, portfolio view
- Price history charts
- Order matching engine

### Phase 3 — Oracle ecosystem

- Oracle data adapter library (NOAA, HUD, WHO, custom HTTP-JSON)
- Oracle CLI / dashboard
- Stake management UI
- Dispute resolution (challenge window + DAO vote)

### Phase 4 — Mainnet

- DarkFi mainnet contract deployment
- Security audit (ZK circuits, contract, P2P)
- P2P transport privacy (Tor integration via DarkFi net)
- Multi-language documentation
- Oracle operator onboarding programme

### Phase 5 — Monero collateral

The initial non-DRK collateral implementation targets XMR only.  The plugin
interfaces (`CollateralManager`, `CollateralSpec`, `ChainAddress`) are designed
for extensibility so additional currencies can be added in Phase 6 without
architectural changes.

- `CollateralSpec` type on Bond struct (`currency`, `amount`, `lock_mechanism`)
- `CollateralManager` trait (extensible plugin interface)
- `ChainAddress` enum with XMR stealth address as the first concrete variant
- XMR collateral: oracle panel DLEQ nonce commitment pre-setup
- XMR collateral: two pre-committed Monero outputs (`s_met` payout, `s_fail` refund)
- XMR collateral: two-phase claim (oracle `s_met` → intermediate; holder BURN proof → payout)
- XMR collateral: timelock fallback (refund without oracle cooperation after deadline)
- Update `payout_address` / `return_address` types from `[u8; 32]` to `ChainAddress`

### Phase 6 — Additional collateral currencies (deferred)

Additional `CollateralManager` plugins, each following the Phase 5 plugin interface:

- Zano: DLEQ adaptor signatures (same mechanism as XMR)
- BTC / ZEC: Taproot / Schnorr DLC construction
- ETH / ERC-20: EVM escrow contract deployment
- AR (Arweave): Warp/SmartWeave escrow contract
- AO (AO Computer): AO Process actor escrow

---

## 19. Glossary

| Term | Definition |
|------|-----------|
| **Attestation** | A signed statement from an oracle operator asserting whether a bond's goal was met |
| **Bond note** | An anonymous ZK commitment representing ownership of bond units |
| **BurnProof** | A ZK proof authorising redemption of a bond note for DRK payout |
| **Challenge window** | A time period after oracle finalisation during which disputes can be raised |
| **Collateral** | Assets locked by the bond issuer (DRK, XMR, BTC, ETH, or other supported currency) equal to `total_supply × redemption_value` |
| **Commitment** | A cryptographic hash that hides the plaintext of a note while allowing verification |
| **data_id** | A dot-separated string identifying the data series used to measure a bond's goal |
| **DRK** | The native privacy token of DarkFi L1; used as bond currency and collateral |
| **GoalMetric** | The machine-readable predicate: `data_id OPERATOR threshold` evaluated against real-world data |
| **GoalSpec** | The full goal definition: title, description, metric, deadline, evidence format |
| **Halo2** | The ZK proof system used by DarkFi; transparent (no trusted setup) |
| **MCP** | Model Context Protocol — JSON-RPC 2.0 over HTTP for AI tool integration |
| **MintProof** | A ZK proof that a new note commitment is well-formed |
| **Nullifier** | A value derived from a note's secret; revealed on spend to prevent double-spend |
| **Oracle** | A participant who fetches real-world data and signs attestations for bonds.  Oracles are attestors, not custodians — they cannot steal or redirect funds. |
| **Oracle quorum** | The minimum number of consistent attestations required to finalise a bond |
| **P2Pool** | A decentralised Monero mining pool using Stratum v1; no central operator |
| **RandomX** | Monero's CPU-friendly proof-of-work algorithm |
| **Redemption value** | DRK paid per bond note unit when the goal is verified as met |
| **Slash fraction** | Fraction of an oracle's stake burned on fraudulent attestation |
| **Social policy bond** | A financial instrument that pays out only when a measurable social goal is achieved |
| **Stratum v1** | The mining protocol used between the NyxForge miner and P2Pool |
| **TransferProof** | A ZK proof that a bond note has been transferred without revealing sender or receiver |
| **Warp / SmartWeave** | A lazy-evaluation smart contract system running on the Arweave network; used to hold and release AR token collateral |
| **XMR** | Monero; privacy-preserving cryptocurrency used for mining, DRK funding, and direct bond collateral via DLEQ adaptor signatures |
| **Zano** | A privacy cryptocurrency with RingCT + stealth addresses; used as bond collateral via DLEQ adaptor signatures, same mechanism as XMR |
| **ZK note** | A zero-knowledge note: a commitment to a value owned by a private key |
| **AO** | AO Computer — a decentralized, message-passing compute system built on Arweave; AO tokens can be used as bond collateral locked in AO Process actors |
| **AO Process** | An actor-based program on the AO Computer network; used to hold and release AO token collateral in response to oracle messages |
| **AR** | Arweave — a decentralized permanent storage network; AR tokens can be used as bond collateral locked in Warp/SmartWeave contracts |
| **ChainAddress** | A currency-aware address type supporting DRK, XMR, Zano, BTC, ETH, AR, AO, and other collateral currencies |
| **CollateralManager** | A per-chain plugin implementing collateral locking, attestation publication, payout claim, and refund |
| **CollateralSpec** | The collateral configuration for a bond: currency, amount, and locking mechanism |
| **DLC** | Discreet Log Contract — a Bitcoin-compatible construct where oracle attestation scalars unlock pre-committed transaction outputs |
| **DLEQ** | Discrete Log Equality proof — used for XMR collateral; same primitive as XMR-BTC atomic swaps; allows oracle attestation to unlock Monero outputs without oracle custody |
| **payout_address** | The address where redemption proceeds are sent; set by the current bond holder at BURN time (in `BurnWitness`), not at bond issuance |
| **s_met / s_fail** | Pre-committed oracle attestation scalars: `s_met` unlocks the payout collateral output; `s_fail` unlocks the refund output |
| **Two-phase claim** | The redemption process for chain-specific collateral: (1) oracle's `s_met` unlocks intermediate output; (2) holder's BURN proof sweeps it to their `payout_address` |
