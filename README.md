# NyxForge

> Anonymous, decentralised, peer-to-peer social policy bond market
> Built on [DarkFi](https://dark.fi) · Written in Rust · Runs on WASM

---

## What is NyxForge?

**Social Policy Bonds** are financial instruments that pay out only when a measurable
social or environmental goal is achieved (reduced homelessness, clean air targets,
literacy rates, etc.).  Traditional SPB schemes require trusted institutions to issue
and settle bonds.

**NyxForge removes that requirement.**  Anyone can:

- **Define** a goal with verifiable, on-chain criteria.
- **Issue** bonds backed by a DarkFi DAO treasury or individual collateral.
- **Trade** bonds anonymously on a ZK order-book DEX.
- **Verify** goal completion through a decentralised oracle network.
- **Redeem** bonds via anonymous ZK settlement — no KYC, no bank, no app store.

Everything runs as WASM in the browser.  There is no Apple/Google gatekeeping.

---

## Architecture

```
nyxforge/
├── crates/
│   ├── nyxforge-core        # Bond & market primitives, shared types
│   ├── nyxforge-zk          # ZK circuits: mint / transfer / burn / verify
│   ├── nyxforge-contract    # DarkFi WASM smart contracts
│   ├── nyxforge-node        # P2P node (libp2p gossip + DarkFi net)
│   ├── nyxforge-oracle      # Decentralised goal-verification oracle network
│   └── nyxforge-web         # Browser WASM frontend (wasm-bindgen)
└── docs/
    ├── architecture.md      # System design deep-dive
    ├── bond-lifecycle.md    # Bond state machine
    └── zk-design.md         # ZK circuit descriptions
```

## Bond Lifecycle

```
DRAFT ──issue──► ACTIVE ──trade──► ACTIVE (new holder)
                    │
               goal achieved?
                    │
               verify (oracle)
                    │
               REDEEMABLE ──redeem──► SETTLED
                    │
               deadline passed, goal unmet
                    │
               EXPIRED
```

## Getting Started

### Prerequisites

- Rust 1.79+ (install via rustup)
- wasm-pack (`cargo install wasm-pack`)
- A DarkFi testnet node (see [DarkFi docs](https://dark.fi/))

### Build

```bash
# Build all crates
cargo build --workspace

# Build WASM frontend
cd crates/nyxforge-web
wasm-pack build --target web

# Run a local node
cargo run -p nyxforge-node -- --testnet
```

### Run tests

```bash
cargo test --workspace
```

---

## Design Principles

1. **Privacy by default** — bond ownership and trades are ZK-anonymous; only
   goal verification results are public.
2. **No trusted third party** — oracles are a decentralised network with economic
   stake; settlement is trustless on-chain.
3. **Sovereign access** — WASM in browser; no native app install required.
4. **Outcome-driven incentives** — issuers set goals, markets price probability;
   profit motive aligns with social good.

---

## License

AGPL-3.0.  Contributions welcome.
