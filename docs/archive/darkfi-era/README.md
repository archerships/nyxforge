# Archive: DarkFi-era specifications

> Status: HISTORICAL / SUPERSEDED
> Archived: 2026-04-25

These specifications describe the **original NyxForge design**: a DarkFi L1
integration with DRK-token collateral, ZK-note ownership, nullifiers, and a P2P
order book. That design was superseded when the MVP pivoted to bearer `.bounty`
files with DLEQ/PTLC adaptor-signature collateral and **no dependency on DarkFi
mainnet**.

The current build target is [`../00_MVP.md`](../00_MVP.md).

## Contents

| File | Original subject |
| :--- | :--- |
| `architecture.md` | DarkFi L1 integration points, crate/subsystem layout |
| `bond-lifecycle.md` | DRK-escrow bond lifecycle, ZK payout notes |
| `oracle-spec.md` | Oracle network over the DarkFi design |
| `privacy-design.md` | AO log privacy, DarkFi P2P gossip, DRK/NYX key derivation |
| `roadmap.md` | Original phases (DarkFi L1 was Phase 8) |
| `user-manual.md` | `drk` wallet commands, DRK-denominated staking |
| `zk-design.md` | Halo2/zkVM circuits and the DRK payout note commitment |

DarkFi-era **code** is archived separately at
[`../../../src/archive/darkfi-era/`](../../../src/archive/darkfi-era/).

Kept for historical reference. Do not re-integrate without a deliberate
architecture decision.
