# NyxForge

> Anonymous, permissionless social policy bond marketplace.
> **Bearer-file architecture** -- no custom blockchain, no ZK ownership circuits, no trusted custodian.
> **XMR-first.** Written in Rust.

---

## What NyxForge is

[Social policy bonds](https://en.wikipedia.org/wiki/Social_policy_bond) pay out only when a
measurable real-world goal is achieved -- reduced homelessness, clean air targets,
literacy rates. Traditional SPB schemes require trusted institutions to issue and
settle the bounty. NyxForge removes the trusted party.

A bounty is a **self-contained bearer instrument**: a `.bounty` file backed by crypto
collateral locked with DLEQ/PTLC adaptor signatures or smart-contract escrow. Anyone
holding the file and its secret scalar can verify it and redeem it. Possession is
ownership.

## Status

`doc/00_MVP.md` (**MVP spec v1.0, April 2026**) is the **current build target** and
takes precedence over every other design document.

| Document | Role |
| :--- | :--- |
| [`docs/00_MVP.md`](docs/00_MVP.md) | Current build target: bearer files, DLEQ/PTLC collateral |
| [`docs/architecture.md`](docs/architecture.md) | Current architecture: components, collateral rails, lifecycle |
| [`docs/00_CORE.md`](docs/00_CORE.md) | v2.0 aspirational spec: ZK notes, order book, P2P network |
| [`docs/archive/darkfi-era/`](docs/archive/darkfi-era/) | Superseded DarkFi L1 / DRK-token / ZK-note design (historical) |
| `crates/nyxforge-zk`, `-contract`, `-node` | Preserved v2 code; **not** on the MVP critical path |

## How it works

- **Bearer file.** One SQLite `.bounty` file per bounty, containing the goal, terms,
  collateral details, judge panel, evidence, and verification logic. No P2P network,
  no ZK ownership circuits, no nullifier set.
- **Trustless collateral.** XMR (DLEQ), ZEC shielded Sapling (DLEQ), BTC Taproot
  (PTLC), TARI (PTLC), ETH (escrow contract), NYX (ledger escrow). The judge
  attests to the outcome but **cannot steal the collateral**: payout requires
  completing a pre-signed adaptor transaction with `s_met`, and the issuer reclaims
  after expiry with `s_fail`.
- **Judges.** A human panel for qualitative goals, an HTTP-JSON oracle for
  quantitative ones. A quorum of attestations moves the bounty to REDEEMABLE.
- **Trading.** Bilateral and off-chain -- the file plus its scalar, exchanged over
  Tor or Signal. No order book is needed for the MVP.
- **Archival format.** SQLite (Library of Congress standard) plus the NyxEnvelope
  for non-bounty files, chosen for 200-year durability.

## Currencies: XMR first

**XMR is the default reference implementation and the launch target** -- the MVP
success criterion is the first live bounty on Monero mainnet.

The MVP supports XMR, ZEC (Sapling), BTC (Taproot), ETH, TARI, and NYX from genesis.
**NYX** is NyxForge's own unit, included in the MVP to support testing, reward
accounting, and early-supporter incentives; NYX-denominated judge fees are allowed
when the bounty's collateral currency is NYX. NYX is **not** a fundraise: the MVP
funding model is grants (Gitcoin, Octant, Monero CCS, ZCash Foundation) plus a
milestone-based development bounty -- no token sale, no DAO, no corporate entity.

## Out of scope for the MVP

Deferred to **v2**: ZK MINT/TRANSFER/BURN ownership circuits, libp2p P2P network,
order book / DEX, Flutter browser UI, nullifier set, Merkle membership proofs, judge
slashing, goal-text encryption.
Deferred to **v3**: DarkFi L1 integration, full post-quantum verifiers, ZEC Orchard
locking. Bounties with deadlines beyond 10 years are blocked on the cryptographically
relevant quantum computer (CRQC) timeline.

## Repository layout

```
crates/           Rust workspace: nyxforge-{core,zk,contract,node,oracle,web,wallet,miner,cli,mcp,test-fixtures}
ui/               Flutter browser UI (v2)
docs/             Specifications and research (see the Status table above)
archive/          Archived DarkFi-era code (historical; do not re-integrate without a decision)
scripts/          Tooling. Storyboard/mockup generators live in scripts/storyboards/
tests/            CLI behavioural test suite (see tests/README.md)
mock/             Mockoon stub API used by the UI work
```

MVP-critical work touches a `.bounty`/bounty crate, parts of `nyxforge-cli`, and a
judge crate. Every other crate is preserved but must not be modified during MVP
development.

## Building

```bash
cargo build            # Rust workspace
cargo test
```

## Documentation

- `docs/00_MVP.md` -- the current spec: file schema, lifecycle, judge protocol, CLI
- `docs/file-format-spec.md` -- `.bounty` and NyxEnvelope formats
- `docs/background-notes/` -- oracle, privacy, and cryptographic research briefs
- `docs/storyboards/` -- UI flows and rendered mockups
- `docs/research/` -- supporting research

## Contributing

Read `docs/00_MVP.md` before proposing changes. Design changes belong in the spec,
not in code comments.

## License

BSD 3-Clause. See [LICENSE](LICENSE).
