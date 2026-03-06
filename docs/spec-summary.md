# NyxForge — Product Summary

## What It Is

NyxForge is a decentralised, anonymous market for **social policy bonds** — financial
instruments that pay out only when a measurable real-world goal is achieved.
Anyone can issue a bond, fund a goal, trade positions, or operate as an oracle.
All ownership and transactions are private by default, enforced by zero-knowledge
proofs on the DarkFi L1 chain.

---

## The Problem

Existing mechanisms for funding social outcomes are broken in three ways:

1. **Misaligned incentives.** Governments and NGOs pay for *activity* (programmes,
   reports, headcount), not *results*. Money flows whether goals are met or not.

2. **No price signal.** There is no continuous, market-priced estimate of whether a
   goal will be achieved. Interventions are chosen by committees, not by aggregated
   distributed knowledge.

3. **Surveillance.** Any on-chain attempt to improve these systems has required
   publishing donor identities, holdings, and transaction histories — chilling
   participation from privacy-conscious actors, dissidents, and jurisdictions with
   hostile governments.

---

## The Solution

A bond series is defined by a **GoalSpec**: a data source, a comparison operator,
a numeric threshold, and a deadline. The issuer locks collateral equal to
`total_supply × redemption_value`. Bond notes are tradeable. When oracle operators
attest that the goal was met, holders redeem notes for the full payout. Oracles are
cryptographically constrained attestors — they cannot steal or redirect collateral.
If the goal is not met by the deadline, the issuer reclaims collateral and notes
expire worthless.

The secondary market price reflects the crowd's live probability estimate that
the goal will be achieved — a continuous, unincentivised signal that neither a
government nor a foundation can produce.

---

## Key Features

| Feature | Description |
|---|---|
| **AI-assisted bond design** | Describe a goal in plain language; the MCP AI layer finds similar existing bonds and drafts a new spec |
| **Community proposal review** | Bonds start in a *Proposed* state; anyone can post questions or improvements before the bond goes live |
| **Oracle approval workflow** | Named oracle operators must explicitly accept responsibility for judging a bond before it is issued |
| **Anonymous trading** | ZK transfer proofs (Ristretto / Halo2) hide sender, receiver, quantity, and price |
| **P2P, no custodian** | Every user runs a local node; there is no server, no company, no account |
| **Built-in XMR miner** | RandomX / P2Pool miner; mining earns XMR for collateral funding |
| **Coin-agnostic collateral** | Issuers lock collateral in DRK, XMR, Zano, BTC, ETH, AR (Arweave), AO (AO Computer), or other currencies; chain-specific locking via DLC / DLEQ / smart contract / AO Process |
| **Oracle trust enforcement** | Oracles are attestors, not custodians; DLC/DLEQ adaptor signatures make fund theft cryptographically impossible |
| **Flexible payout address** | The redemption destination is set by the current holder at burn time — not locked at bond issuance |
| **AI-provider agnostic** | MCP server supports Anthropic, OpenAI, Ollama, or any OpenAI-compatible endpoint |

---

## Architecture (one line each)

| Component | Role |
|---|---|
| `nyxforge-node` | Local P2P node; JSON-RPC bridge to browser; libp2p gossip |
| `nyxforge-contract` | DarkFi WASM smart contracts for bond issuance, trading, settlement |
| `nyxforge-zk` | ZK circuits: MINT / TRANSFER / BURN (Halo2) |
| `nyxforge-oracle` | Oracle daemon; pluggable data adapters; attests goal outcomes |
| `nyxforge-wallet` | XMR + DRK key management; stagenet/mainnet wallet |
| `nyxforge-miner` | RandomX CPU miner; Stratum v1 (P2Pool) |
| `nyxforge-cli` | Interactive bond wizard; full lifecycle management |
| `nyxforge-mcp` | AI provider bridge (MCP protocol); provider management REST API |
| `nyxforge-web` | Browser WASM frontend (Flutter) |

---

## Who It's For

- **Impact investors** who want outcome-linked exposure, not programme risk
- **Philanthropists and foundations** seeking verifiable ROI on social spending
- **Speculators** willing to price the probability of social change
- **Oracle operators** who provide reliable data and earn fees for doing so
- **Privacy-conscious participants** in any of the above roles

---

## Current Status

Working prototype (testnet / stagenet):

- [x] Full bond lifecycle: Proposed → Oracle Approval → Draft → Active → Redeemable → Settled
- [x] CLI bond wizard with AI assistance (MCP)
- [x] Oracle registration, approval workflow, attestation
- [x] XMR wallet (stagenet), mining (RandomX / P2Pool)
- [x] Node JSON-RPC, P2P gossip scaffold
- [x] ZK proofs wired (Halo2 PLONK; MINT/TRANSFER/BURN; 176 tests passing; 6 slow round-trip tests ignored)
- [ ] Browser WASM UI beyond splash/navigation
- [ ] DarkFi L1 integration (contract deployment)
- [ ] Oracle data adapter library (beyond mock + HTTP-JSON)
- [ ] CollateralSpec / CollateralManager / ChainAddress plugin architecture (Phase 5)
- [ ] XMR collateral: DLEQ oracle setup, two-phase claim, timelock fallback (Phase 5)
- [ ] Zano / BTC / ETH / AR / AO collateral plugins (Phase 6 — deferred)
