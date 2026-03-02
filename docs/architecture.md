# NyxForge — Architecture

## System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        Browser (WASM)                            │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  nyxforge-web  (wasm-pack / wasm-bindgen)               │     │
│  │  • key gen / wallet (localStorage)                      │     │
│  │  • bond browser / order entry UI                        │     │
│  │  • ZK proof generation (mint/transfer/burn)             │     │
│  └────────────────┬────────────────────────────────────────┘     │
└───────────────────│──────────────────────────────────────────────┘
                    │ HTTP JSON-RPC (localhost:8888)
┌───────────────────▼──────────────────────────────────────────────┐
│  nyxforge-node  (Rust binary, user-run)                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │  State   │  │  RPC Server  │  │  Contract    │               │
│  │ (bonds,  │  │  (axum HTTP) │  │  Engine      │               │
│  │  orders, │  └──────────────┘  │  (ZK verify) │               │
│  │  nullif) │                    └──────────────┘               │
│  └──────────┘                                                    │
│        │                                                         │
│  ┌─────▼──────────────────────────────────────────────────┐      │
│  │  P2P Swarm  (libp2p)                                   │      │
│  │  • Gossipsub  — bond/order/trade/oracle propagation    │      │
│  │  • Kademlia   — peer discovery                         │      │
│  │  • Noise XX   — encrypted transport                    │      │
│  └──────────────────────────────────────────────────────-─┘      │
└──────────────────────────────────────────────────────────────────┘
              │                              │
   ┌──────────▼──────────┐     ┌────────────▼────────────┐
   │  nyxforge-oracle    │     │  DarkFi L1 chain        │
   │  • data adapters    │     │  • ZK contract state    │
   │  • goal evaluation  │     │  • note tree / nullif   │
   │  • attestation sign │     │  • collateral escrow    │
   └─────────────────────┘     └─────────────────────────┘
```

## Crate Dependency Graph

```
nyxforge-web
  └── nyxforge-core
  └── nyxforge-zk
        └── nyxforge-core

nyxforge-node
  └── nyxforge-core
  └── nyxforge-contract
        └── nyxforge-core
        └── nyxforge-zk
  └── nyxforge-oracle
        └── nyxforge-core

nyxforge-oracle (binary)
  └── nyxforge-core
```

## Privacy Model

All bond ownership is represented as **ZK notes** (analogous to Zcash sapling notes):

```
Note {
    bond_id          // which series
    quantity         // units held
    redemption_value // cached
    owner            // public key (hidden in commitment)
    randomness       // blinding factor
    serial           // unique; hashed → nullifier
}

Commitment = Com(bond_id, qty, owner, randomness)   // public, on-chain
Nullifier  = PRF(owner_secret, serial)              // revealed on spend
```

Spending a note reveals its nullifier (preventing double-spend) but leaks
no information about who owned it, how long they held it, or what they paid.

## DarkFi Integration Points

| What                | DarkFi API                     |
|---------------------|--------------------------------|
| ZK proofs           | `darkfi_sdk::zk::Proof`        |
| Note tree           | `darkfi::blockchain::TxStore`  |
| P2P (alt to libp2p) | `darkfi::net::P2p`             |
| Contract execution  | `darkfi::runtime::Wasm`        |
| Collateral escrow   | DRK anonymous token contract   |

## Data Flow: Bond Issuance

```
1. Issuer defines GoalSpec + OracleSpec in the UI (browser/WASM)
2. nyxforge-web generates:
     - Bond struct (draft state)
     - Collateral note (locking DRK in escrow)
     - MintProof for the initial bond notes
3. Submits to nyxforge-node via JSON-RPC
4. Node calls process_issue_bond() from nyxforge-contract
5. Contract verifies fields + collateral proof
6. Node gossips the new Bond to the P2P network
7. All peers update their state; bond appears in market
```

## Data Flow: Anonymous Trade

```
Maker (seller):                Taker (buyer):
  PlaceOrder(ask)     →→→→→→→→  PlaceOrder(bid)
  (ZK ownership proof)          (ZK token balance proof)
                      ← match ←
  TransferProof(bond)  →→→→→→→→  receives new bond note
                      ← PaymentTransferProof ←
  receives DRK note
                      both nullifiers go on-chain
```

## Data Flow: Redemption

```
Oracle network monitors bond deadlines
  → fetches data (HTTP adapter, IPFS, etc.)
  → evaluates GoalMetric predicate
  → signs OracleAttestation
  → gossips to P2P network

Once quorum reached:
  FinaliseVerification → bond.state = REDEEMABLE

Bond holder:
  generates BurnProof (proves ownership, goal_met attestation hash)
  submits to node → contract issues payout note (anonymous DRK)
```
