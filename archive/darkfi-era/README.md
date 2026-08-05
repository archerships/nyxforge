# Archive: DarkFi Era Code
> Status: HISTORICAL / SUPERSEDED
> Archived: 2026-04-25
> Superseded by: doc/00_MVP.md (bearer-file architecture, DLEQ/PTLC collateral)

These files implement the DarkFi L1 integration and DRK token wallet that was
the original NyxForge collateral model (March 2026). The design was superseded
when the MVP pivoted to XMR/ZEC/BTC/ETH bearer-file approach with DLEQ/PTLC
adaptor signatures and no dependency on DarkFi mainnet.

## Contents

nyxforge-wallet-drk/   -- DRK anonymous note wallet (DarkFi SDK integration)
nyxforge-miner-darkfi.rs -- DarkFi merge-mining module

## Why preserved

Kept for historical reference and potential reuse if DRK collateral is
revisited post-MVP (the roadmap originally planned DarkFi L1 for Phase 8).
Do not re-integrate without a deliberate architecture decision.
