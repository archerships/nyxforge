# NyxForge — Product Specification

> Version 0.2 — living document — April 2026
>
> Authority note: This file is the v2.0+ aspirational spec and long-form product
> narrative.  For the active build target (bearer .bounty files, SQLite schema,
> multi-term bounties, DLEQ/PTLC collateral), see `00_MVP.md`.  Where the two files
> conflict, `00_MVP.md` takes precedence for all current development decisions.

---

## Table of Contents

1. [Vision & Mission](#1-vision--mission)
2. [Problem Statement](#2-problem-statement)
3. [Target Users](#3-target-users)
4. [Product Overview](#4-product-overview)
5. [Core Concepts](#5-core-concepts)
6. [Functional Requirements](#6-functional-requirements)
7. [Non-Functional Requirements](#7-non-functional-requirements)
8. [Bounty Lifecycle](#8-bounty-lifecycle)
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
20. [v2.0 Architectural Extensions](#20-v20-architectural-extensions)
21. [.bounty Archive Format](#21-bounty-archive-format)

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

Existing social impact bounty programmes require trusting the issuer to lock
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
signed attestations.  They earn fees proportional to their stake and attestation
accuracy.

### 3.5 Privacy-conscious participants

Any of the above who cannot or will not publish their identity, holdings, or
transaction history.  NyxForge is the only venue where any of these roles can
be played anonymously by default.

---

## 4. Product Overview

NyxForge is a system for creating and trading **Social Policy Bonds** as
self-contained **bearer instruments**. A bounty is represented by a single
`.bounty` file (SQLite) that contains the goal, the collateral details, and the
evidence required for settlement.

The core loop:

1.  **Issuance:** An issuer defines a goal (GoalSpec) and a collateral amount.
    They select an oracle panel and lock the collateral using a currency-specific
    mechanism (**DLEQ/PTLC adaptor signatures** for XMR/BTC/ZEC, or a **Smart
    Contract Escrow** for ETH).
2.  **Possession as Ownership:** Possession of the `.bounty` file (and the
    corresponding secret scalar) constitutes ownership. There is no central
    registry, no P2P network, and no ZK ownership circuit in the MVP.
3.  **Bilateral Trading:** Bounties are traded directly between individuals (off-chain)
    by exchanging the `.bounty` file and re-encrypting the secret scalar to the
    buyer's public key.
4.  **Oracle Settlement:** Oracle operators monitor the goal. On achievement, they
    publish a settlement credential (**s_met** scalar or an **EIP-712 signature**).
5.  **Redemption:** The holder combines their secret with the oracle's credential
    to sweep the collateral directly from the locking address to their own wallet.

No company, server, or custodian mediates any step. Every user runs a local
node.

---

## 5. Core Concepts

### 5.1 Social policy bond

A financial instrument whose redemption is conditional on a measurable social or
environmental outcome being achieved by a specified date. The instrument
converts vague philanthropic intent into a precise, verifiable commitment with
market-priced probability attached.

The term was coined by Ronnie Hoban (1988). NyxForge implements the concept
using decentralized adaptor signatures and smart contracts.

### 5.2 GoalSpec

The machine-readable definition of a bounty's outcome target.  A bounty has one or
more terms; if there are multiple terms, `term_aggregation` specifies whether
all must be met (AND) or any one suffices (OR).

In the MVP SQLite schema (see `00_MVP.md` Section 4.1):

```sql
-- bounty_spec table
goal_type        TEXT    -- derived from terms: quantitative | qualitative | hybrid
term_aggregation TEXT    -- AND | OR; null when bounty has exactly one term

-- terms table (one row per term)
seq        INTEGER  -- 1-based display order
goal_type  TEXT     -- quantitative | qualitative | hybrid
criterion  TEXT     -- human-readable description of the condition
data_id    TEXT     -- e.g. usgov.cbo.federal_outlays_pct_gdp
operator   TEXT     -- lt | lte | gt | gte | eq
threshold  REAL
aggregation TEXT    -- e.g. annual_mean
```

### 5.3 Bearer Bounty (.bounty file)

The primary unit of ownership in NyxForge. It is an encrypted SQLite container
holding the full state of the bounty.

- **Archival stability:** SQLite is a Library of Congress recommended format for
  long-term data preservation.
- **Atomic state:** The file IS the bounty. It contains the goal, the history,
  the evidence BLOBs, and the settlement logic.

### 5.4 Oracle Attestation

A signed statement by a registered oracle operator asserting whether a bounty's
goal was met. For MVP, attestations result in the release of a **secret scalar
(s_met)** or a **smart contract signature**.

### 5.5 Collateral Currencies

NyxForge is coin-agnostic. Support includes:

- **XMR (Monero):** Default reference implementation using DLEQ adaptor signatures.
- **ZEC (Zcash):** Shielded Sapling outputs using Jubjub DLEQ.
- **BTC (Bitcoin):** Taproot outputs using secp256k1 PTLCs (BIP-340).
- **ETH (Ethereum):** EVM smart contract escrow (NyxForgeEscrow).

DRK (DarkFi) integration and ZK-note ownership models are deferred to v2.0+.

### 5.6 Supported collateral currencies

Bounty collateral can be locked in any supported currency. The locking mechanism varies by chain:

| Currency | Lock mechanism | Oracle Credential | Privacy |
|---|---|---|---|
| XMR | dleq_xmr | s_met scalar | Full (Stealth/RingCT) |
| ZEC | dleq_zec_sapling | s_met scalar | Full (Sapling) |
| BTC | ptlc_btc | s_met scalar | Pseudonymous (Taproot) |
| ETH | eth_escrow | EIP-712 Signature | Pseudonymous (EVM) |

The issuer specifies the currency and amount in `CollateralSpec` at bounty creation. The `return_address` field specifies where collateral is returned if the goal is not met.

### 5.7 Oracle trust model — attestors, not custodians

Oracles **attest to real-world outcomes**; they do not control fund routing. Two outcomes are pre-committed at bounty setup:

- **`s_met`**: oracle scalar proving "goal was achieved" — reveals the spending key for the payout collateral output
- **`s_fail`**: oracle scalar proving "goal failed / expired" — reveals the spending key for the refund output

Publishing *either* scalar resolves exactly one output. Publishing *both* scalars would reveal the oracle's private key — this is **cryptographically enforced**, not trust-based. Oracles cannot redirect funds to themselves because no output is addressed to any oracle key.

The enforcement mechanism varies by chain:
- **XMR / ZEC**: DLEQ (Discrete Log Equality) proofs.
- **BTC**: Point Time-Locked Contracts (PTLC) on secp256k1 Taproot.
- **ETH / ERC-20**: EVM smart contract enforces payout routing on-chain.

For chains requiring a two-phase claim (XMR, BTC):
1. Oracle publishes `s_met` → unlocks an intermediate collateral output.
2. The current bounty holder combines the oracle's scalar with their own secret to sweep the funds to their `payout_address`.

Neither the oracle nor an outside observer can complete both phases.

### 5.8 Transfer Protocol

#### The primary issuance problem

A naive implementation locks collateral and stores the secret scalar in the
`.bounty` file at issuance time, then "transfers" to the first buyer by
re-encrypting the scalar to the buyer's public key.  This is insecure: the
issuer held the scalar in plaintext during setup and can retain a copy.  If the
goal is later met, the issuer can sweep the collateral ahead of the buyer.

Re-encryption alone does not solve this for primary issuance.

#### Trustless primary issuance (MVP)

The MVP solves the problem by removing the issuer from scalar generation
entirely.  The buyer participates in bounty setup before collateral is locked:

1. Buyer generates a fresh keypair `(b, B)` where `B = b*G`.
2. Buyer sends `B` (pubkey only) to the issuer.
3. Issuer runs the creation wizard with the buyer's pubkey as the
   `holder_pubkey`.  The wizard derives the collateral lock address from `B`
   and the oracle adaptor point; it never generates or stores a plaintext
   scalar that the issuer could later use.
4. Collateral is locked.  The `.bounty` file is created with
   `scalar_encrypted = enc(t, B)` where `t` is the buyer's adaptor contribution
   (generated on the buyer's machine).
5. Issuer sends the `.bounty` file to the buyer.  The buyer verifies
   `lock_txid` on-chain and confirms `holder_pubkey == B`.
6. Buyer pays the issuer.

At no point does the issuer learn `b` or `t`.  The issuer cannot construct the
sweep even after `s_met` is published because the full spending key requires
`b + s_met` (discrete log of the lock address), and `b` is known only to the
buyer.

**Practical implication:** the buyer must provide their pubkey before the bounty
is issued.  The creation wizard requires a `--holder-pubkey` argument (or
prompts for it interactively) for any bounty that will be sold on the primary
market.  An issuer who creates a bounty for their own account first and re-sells
it later cannot offer trustless primary issuance without DLEQ re-keying (see
Phase 2 below).

#### Secondary transfer (re-encryption)

Once a buyer holds a bounty, any subsequent transfer is safe via re-encryption:

1. Seller runs `nyxforge-cli bounty transfer <file> <buyer_pubkey>`.
2. The CLI re-encrypts `scalar_encrypted` from the seller's key to the buyer's
   key and updates `holder_pubkey`.
3. Seller sends the updated file to the buyer and receives payment.
4. After transfer, the seller's decryption key no longer matches
   `holder_pubkey`.  The seller cannot decrypt `scalar_encrypted` and cannot
   complete the two-phase sweep.

The re-encryption is enforced by the file format: `scalar_encrypted` is an
ECIES ciphertext bound to `holder_pubkey`.  Any transfer attempt that does not
produce a valid ciphertext for the new pubkey will be rejected by `bounty verify`.

#### DLEQ re-keying (Phase 2)

Phase 2 will implement blind re-keying via DLEQ proofs, allowing an issuer who
created a bounty without a specific buyer in mind to subsequently transfer to a
buyer without the buyer needing to trust that the issuer deleted their scalar
copy.  The issuer proves the re-key was performed correctly (discrete log
relationship between old and new lock address) without learning the buyer's
private key.  This enables "blind issuance" workflows where bounties are created
speculatively and sold into the secondary market.

---

## 6. Functional Requirements (MVP)

### 6.1 Bounty management

| ID | Requirement |
|----|-------------|
| F-B01 | Issuer can define a GoalSpec via interactive CLI wizard |
| F-B02 | AI assistant (MCP) can draft a GoalSpec from natural language |
| F-B03 | Bounty can be published in `Proposed` state for community review |
| F-B04 | Any participant with a `.bounty` file can post comments or attach evidence |
| F-B05 | Oracles can review and accept a bounty before it moves to `Draft` |
| F-B06 | Bounty issuance fails if the `data_id` has no registered oracle adapter |

### 6.2 Trading (Bilateral)

| ID | Requirement |
|----|-------------|
| F-T01 | Holder can generate a signed Listing Record (`bounty prove`) to prove ownership |
| F-T02 | Holder can transfer a bounty by re-encrypting the secret scalar to a recipient |
| F-T03 | Trading is off-chain (file exchange over Signal/Tor) |
| F-T04 | No shared order book or P2P network required for MVP |
| F-T05 | Creation wizard accepts an optional `--holder-pubkey` to enable trustless primary issuance (issuer never holds the plaintext scalar) |
| F-T06 | `bounty verify` rejects a `.bounty` file whose `scalar_encrypted` ciphertext does not match `holder_pubkey` |

### 6.3 Oracle

| ID | Requirement |
|----|-------------|
| F-O01 | Oracles sign attestations and publish settlement credentials (scalars or signatures) |
| F-O02 | Qualitative oracles can attach evidence BLOBs (PDF/Video) to the `.bounty` file |
| F-O03 | Quantitative oracles fetch data via pluggable HTTP-JSON adapters |
| F-O04 | Once `quorum` of attestations accumulate, the bounty becomes `Redeemable` |

### 6.4 Redemption & settlement

| ID | Requirement |
|----|-------------|
| F-R01 | Holder redeems by combining oracle's credential with their own secret |
| F-R02 | Payout is sent to the address provided at redemption time |
| F-R03 | Issuer reclaims collateral via `s_fail` if the goal is not met by the deadline |
| F-R04 | Timelock fallback allows issuer reclaim without oracle cooperation after expiry |

---

## 7. Non-Functional Requirements

### 7.1 Privacy

- All bounty ownership is anonymous by default (possession-based).
- No user account or KYC required.
- Keys generated and stored locally.
- On-chain anonymity provided by the collateral chain (XMR/ZEC).

### 7.2 Archival Stability

- 200-year mandate via SQLite and embedded WASM verifiers.
- Offline-first: redemption can occur in air-gapped environments.

---

## 8. Bounty Lifecycle

### State summary

| State | Description |
|-------|-------------|
| `DRAFT` | File created, collateral not yet locked |
| `ACTIVE` | Collateral locked; oracles registered; trading open |
| `REDEEMABLE` | Oracle quorum reached; settlement credential available |
| `SETTLED` | Collateral swept by holder; file is spent |
| `EXPIRED` | Deadline passed; goal not met; s_fail available |
| `RECLAIMED` | Collateral returned to issuer |

### Transitions

```
[create] → DRAFT
         → ACTIVE (bounty issue; collateral locked)
         → REDEEMABLE (oracle quorum, goal met)
         → SETTLED
         → EXPIRED (deadline passed, goal not met) → [issuer reclaims]
         → RECLAIMED
```

Any rejection by an oracle while in `PendingOracleApproval` returns the bounty to
the issuer for revision.  The issuer may revise the oracle list (via
`bounties.revise_oracles`), which clears all existing responses and requires all
listed oracles to re-accept from scratch.

---

## 9. Oracle Network

### 9.1 Role

Oracle operators are the link between on-chain bounty contracts and real-world data. They are **attestors** — they sign statements about real-world outcomes — but they are **not custodians** of collateral. Fund routing is enforced cryptographically by the lock mechanism (DLC, DLEQ, or smart contract), not by oracle honesty. An oracle's only power is to choose which of the two pre-committed outcomes (`s_met` or `s_fail`) to publish. Publishing both is cryptographically impossible without revealing the oracle's private key.

### 9.2 Registration

An oracle operator must:

1. Hold a DRK keypair that they register with the bounty issuer.
2. Stake at least `required_stake` DRK (per bounty or globally, TBD).
3. Run the `nyxforge-oracle` daemon with a data adapter for the bounty's `data_id`.
4. Explicitly accept each bounty they are listed on (via `oracle-accept` command or
   the oracle daemon's auto-accept policy).

### 9.3 Attestation flow

1. The oracle daemon monitors active bounties and their deadlines.
2. At evaluation time (typically when the deadline is near or a new data release
   is available), the daemon fetches the data via the registered `DataSource` adapter.
3. For each quantitative term it evaluates: `fetched_value OPERATOR threshold`.
   For qualitative terms, human panel review is required.  The overall bounty
   outcome applies `term_aggregation` (AND/OR) across all term results.
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
challenge window.  In bounties where `dao_override_allowed = false`, no post-hoc
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

## 10. Market Mechanics (MVP)

### 10.1 Bilateral Trading

In the MVP, there is no central order book or automated matching engine.
Trading is **bilateral and off-chain**.

1.  **Discovery:** Sellers share **Listing Records** (produced by `bounty prove`)
    on social media, forums, or P2P messaging (Tor/Signal).
2.  **Negotiation:** Buyer and Seller agree on a price and payment method.
3.  **Settlement:**
    - Seller sends the `.bounty` file to the Buyer.
    - Buyer verifies the bounty's status and ownership proof.
    - Seller re-encrypts the secret scalar (s_met) to the Buyer's public key
      and sends the updated file.
    - Buyer pays the Seller via a separate on-chain transaction.

### 10.2 Listing Records

A Listing Record allows a holder to prove ownership without revealing the
secret scalar. It contains:
- Bounty ID and Metadata summary.
- Current holder's public key.
- A signature over an exchange-provided nonce.

### 10.3 Price Discovery

Price is discovered purely through social consensus and individual negotiation.
The market price of a bounty reflects the crowd's estimate of the probability
of goal achievement: `Price ≈ Redemption_Value × P(goal_met)`.

DEX integration and automated order books are deferred to v2.0+.

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
centralised exchange.  Earned XMR can be converted to DRK for bounty collateral
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

Bounty design is non-trivial: choosing the right data ID, operator, threshold,
aggregation method, and deadline requires domain knowledge.  The AI assistant
helps non-experts design well-formed bounties and surfaces existing bounties that
might overlap with their goal.

### 13.2 Architecture

NyxForge uses the **Model Context Protocol (MCP)** — JSON-RPC 2.0 over HTTP —
to communicate with AI providers.  This keeps the AI integration provider-agnostic
and allows users to run any compatible LLM locally or via a cloud API.

```
nyxforge-cli bounty explore
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
| `tools/call` | Invoke a tool (`bounty_assist`) |

**REST (provider management)**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/providers` | List configured providers |
| POST | `/providers` | Add a provider |
| DELETE | `/providers/:name` | Remove a provider |
| PUT | `/providers/default` | Set the active provider |
| GET | `/health` | Liveness check |

### 13.5 `bounty_assist` tool

Input:
```json
{
  "description": "I want to reduce US unsheltered homelessness to under 100,000 by 2030",
  "existing_bounties": [ { "id": "...", "title": "...", "goal": "..." }, ... ]
}
```

Output (`BountyAssistance`):
```json
{
  "similar_bounties": [
    { "bounty_id": "...", "title": "...", "similarity": "high", "explanation": "..." }
  ],
  "suggested_bounty": {
    "title": "...",
    "description": "...",
    "terms": [
      {
        "goal_type": "quantitative",
        "criterion": "US unsheltered homelessness falls below 100,000",
        "data_id": "us.hud.pit_count.unsheltered",
        "operator": "lt",
        "threshold": "100000",
        "aggregation": "annual_point_in_time"
      }
    ],
    "term_aggregation": null,
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

# Bounty design session
nyxforge-cli bounty explore
```

After exploring, the user can:
- **Create bounty from AI draft** — pre-fills the wizard with the AI's suggestions
- **Back an existing bounty** — view market instructions for a similar bounty
- **Start fresh** — open the wizard with no pre-fills
- **Cancel**

---

## 14. Privacy Model (Bearer Model)

### 14.1 What is public

| Data | Visibility |
|------|-----------|
| Bounty GoalSpecs, deadlines, data IDs | Public (shared in files) |
| Oracle attestation results | Public (when published) |
| On-chain collateral locks | Public (txid on XMR/BTC/ETH) |

### 14.2 What is private

| Data | Visibility |
|------|-----------|
| Ownership | Private (possession-based; not on-chain) |
| Trade history | Private (bilateral; no public order book) |
| Redemption payouts | Private (shielded by XMR/ZEC) |
| Issuer identity (if desired) | Private (issuer not linked to real-world identity) |

### 14.3 ZK-Note Ownership (v2.0+)

The use of ZK-circuits (MINT/TRANSFER/BURN) to track ownership on a shared
ledger is deferred to v2.0. The MVP uses the **Bearer Model** where the file
itself is the proof of stake.

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

| Crate | Role |
|-------|------|
| `nyxforge-bounty` | **(NEW)** .bounty SQLite schema, DLEQ primitives, file I/O |
| `nyxforge-cli` | Interactive bounty wizard; inspect/status/transfer/redeem |
| `nyxforge-oracle` | Oracle accept/attest workflow; quantitative adapters |
| `nyxforge-mcp` | AI provider bridge (Model Context Protocol) |
| `nyxforge-core` | Shared types (deferred v2.0 types preserved) |
| `nyxforge-zk` | ZK circuits (deferred to v2.0) |
| `nyxforge-contract` | Solidity escrow for ETH; settlement logic |
| `nyxforge-node` | Local P2P node; JSON-RPC server; state |
| `nyxforge-miner` | RandomX CPU miner; Stratum v1 client |
| `nyxforge-wallet` | XMR key management |

### 15.2 Integration Status

Current status: building the bearer-bounty prototype.

1.  **Phase 0-3:** UI/UX development using Penpot, Mockoon, and Flutter.
2.  **Phase 4:** Implementing the `nyxforge-bounty` crate with rusqlite.
3.  **Phase 5:** Monero stagenet integration for DLEQ collateral.
4.  **Phase 6-7:** Wiring the UI and launching the first live bounty.

DarkFi L1 integration and ZK ownership proofs are moved to the v2.0/v3.0 roadmap.

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
- **Content moderation** — Bounty goal specs are not reviewed or filtered by any
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
| Active bounties | ≥ 50 |
| Unique oracle operators | ≥ 10 |
| Total collateral locked | ≥ 100,000 DRK |
| Bounties successfully settled (goal met) | ≥ 5 |
| Bounties expired (goal not met, collateral reclaimed) | Measurable (proves the mechanism works) |

### 17.2 Market quality

| Metric | Target |
|--------|--------|
| Average bid-ask spread | < 5% of redemption value |
| Median time to first trade after issuance | < 24 hours |
| Bounty price correlation with independent probability estimates | > 0.7 |

### 17.3 Developer adoption

| Metric | Target |
|--------|--------|
| Oracle data adapters contributed | ≥ 5 external |
| AI provider integrations | ≥ 3 (Anthropic, OpenAI, Ollama live) |
| External node deployments | ≥ 20 |

---

## 18. Roadmap

### Phase 0 — User Flows

- [x] Full bounty lifecycle diagrams (Proposed → Settled) committed to `doc/04_STORYBOARDS/`.
- [x] Identifying all edge cases: expired bounty, rejected oracle, quorum not reached.

### Phase 1 — Screen Designs (Penpot)

- [ ] Design bounty list, creation wizard, and detail screens.
- [ ] Component library: bounty card, state badges, oracle status rows.

### Phase 2 — Mock API (Mockoon)

- [x] JSON-RPC mock returning fake bounties for all 6 lifecycle states.
- [x] Mock server used for Flutter and CLI front-end development.

### Phase 3 — Flutter UI (Widgetbook)

- [ ] Build all screens in isolation via Widgetbook.
- [ ] Compost UI with navigation and state management against Mockoon.

### Phase 4 — nyxforge-bounty Crate

- [ ] SQLite integration (rusqlite) for `.bounty` files.
- [ ] DLEQ adaptor signature library for Monero/Zcash.
- [ ] Quantitative oracle adapters (HTTP-JSON).

### Phase 5 — Collateral & Redemption

- [ ] XMR DLEQ setup at bounty issuance.
- [ ] `bounty redeem`: combine adaptor + s_met → complete sweep tx.
- [ ] Stagenet end-to-end testing (issue → attest → redeem).

### Phase 6 — Integration & Finalization

- [ ] Wire Flutter UI to real Rust backend.
- [ ] Handle async on-chain broadcast states.

### Phase 7 — Mainnet Launch

- [ ] Issue the first real bounty on Monero mainnet.
- [ ] Grant applications (Gitcoin, Octant, CCS).

---

## 19. Glossary

| Term | Definition |
|------|-----------|
| **Bearer Bounty** | A self-contained `.bounty` file (SQLite) where possession equals ownership. |
| **s_met / s_fail** | Pre-committed oracle attestation scalars. `s_met` unlocks the payout collateral; `s_fail` unlocks the refund. |
| **Adaptor Signature** | A cryptographic primitive used for DLEQ/PTLC atomic swaps. |
| **GoalSpec** | The machine-readable definition of a bounty's outcome: one or more terms (each with a criterion, optional data_id/operator/threshold), plus a term_aggregation operator (AND/OR). |
| **Listing Record** | A signed proof of ownership used to list a bounty for trade without revealing the secret scalar. |
| **Oracle Panel** | The set of oracles selected by the issuer to monitor and settle a bounty. |
| **Bilateral Trading** | Direct, off-chain exchange of `.bounty` files between individuals. |
| **Redemption Value** | The amount of collateral paid per bounty unit on goal achievement. |
| **DLEQ** | Discrete Log Equality proof; used to unlock Monero/Zcash collateral trustlessly. |
| **PTLC** | Point Time-Locked Contract; used for Bitcoin Taproot collateral. |
| **XMR** | Monero; the primary reference currency for NyxForge MVP. |
| **Trustless Primary Issuance** | A bounty creation flow where the buyer provides their public key before collateral is locked, ensuring the issuer never holds the plaintext scalar and cannot sweep on goal achievement. |
| **DLEQ Re-keying** | A Phase 2 protocol allowing an issuer to cryptographically transfer control of an existing bounty lock to a buyer's key without the buyer trusting that the issuer deleted their copy. |

---

## 20. v2.0 Architectural Extensions

> Absorbed from: `05_TECH/horesh-spb-spec-v2.md` (April 2026)

### 20.1 DarkFi Actor Model

The system is modeled as independent Actors communicating via asynchronous messages.

- Each Bounty is an Actor Process (DarkFi WASM module or AO Process).
- State transitions are verified using Halo2 ZK-circuits. Every transaction reveals
  a nullifier and creates a new commitment in the global shielded set.
- All bounty interactions (creation, funding, redemption) produce a standardized
  ZK-proof, ensuring a large, unfractured anonymity set.

### 20.2 Maintenance Bounties and Stability Dividends

Unlike standard SPBs that pay once, a Maintenance Bounty pays periodic Stability
Dividends to bondholders as long as a metric remains within a target range (e.g.
"Annual Mean CO2 < 350ppm"). This makes century-scale bounties economically viable
because holders receive yield while waiting.

### 20.3 XMR Yield Endowment

Bounty collateral is locked in a Yield-Bearing Escrow (RandomX/P2Pool mining rewards).

- Principal: the base collateral (e.g. 10,000 XMR).
- Endowment: ongoing yield from mining.
- Flow: yield funds (1) oracle fees, (2) maintenance bounties, (3) Stability Dividends.

### 20.4 Oracle Succession (The Relay)

Oracles are stake-holding Actors, not named identities.

- Current oracles can deputize successors by signing a transition proof.
- If an oracle set disappears (no heartbeat for 10 years), a consensus of
  bondholders can vote to install a new oracle set.

### 20.5 GoalSpec v2 (Rust)

```rust
struct GoalSpecV2 {
    title: String,
    description: String,
    mode: AdjudicationMode,    // Automated | Subjective | Hybrid
    metric_vk: [u8; 32],       // Halo2 verifier key
    inception: Timestamp,
    expiry: DateTime<Utc>,     // can be 2200-01-01+
    check_interval: Duration,
    collateral: CollateralSpec {
        currency: Currency,    // XMR, DRK, AR
        amount: u128,
        use_yield_as_endowment: bool,
    },
    oracles: Vec<OracleKey>,
    quorum_threshold: u8,
    dispute_period: Duration,
}
```

### 20.6 200-Year Security Mandate

1. Post-Quantum Slots: all signature verifiers must have a migration path to
   lattice-based signatures.
2. Stateless Verification: all evidence for a bounty's resolution must be
   self-contained so the bounty can be resolved in a clean-room environment
   without relying on live Web2 APIs.

---

## 21. .bounty Archive Format

> Absorbed from: `05_TECH/proposed-alteration-hipp-archival.md` (April 2026)
> Philosophy: D. Richard Hipp (SQLite)

### 21.1 Problem

The prior spec relied on Arweave (`ar://`) and HTTPS URLs for evidence storage.
Over 200+ years, gateways disappear, domains expire, and funding models shift.
A bounty linked to an external URL is a brick waiting to happen.

### 21.2 The SQLite Container

Every Social Policy Bond is maintained as a single-file SQLite database with
the extension `.bounty`. The file IS the bounty -- it contains the goal, the
history, the evidence, and the proofs.

- Archival stability: SQLite is a Library of Congress recommended format for
  long-term data preservation.
- Atomic state: all state transitions are appended to the same file.

### 21.3 Embedded Evidence (BLOB Mandate)

External URLs are prohibited for critical adjudication data.

- Schema includes an `evidence_blobs` table.
- Proof-of-life videos, zkTLS JSON packets, and scientific PDFs are stored as
  binary BLOBs directly inside the `.bounty` file.
- Verification logic hashes the local BLOB against the GoalSpec requirement.

### 21.4 Self-Verifying Logic (WASM Bundling)

Each `.bounty` file bundles the WASM bytecode of the Halo2 verifier circuit
required to settle the bounty. A maintainer in the 23rd century needs only a
standard WASM runtime to execute `settle()` using the evidence in the same file.

### 21.5 P2P Replication

Nodes (bondholders, issuers, oracles) pin and mirror the full `.bounty` files
they are invested in. New evidence and state transitions are broadcast as
delta-updates (SQLite WAL segments) to the gossip network.

### 21.6 Security

- SQLCipher (AES-256) encrypts the `.bounty` container; only holders of the
  `BountyViewKey` can read evidence.
- Offline-first: payouts can be prepared and proven in a fully air-gapped
  environment using only the `.bounty` file and a local wallet.
