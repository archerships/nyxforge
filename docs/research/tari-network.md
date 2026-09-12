# Tari Network Research Brief (NyxForge)

**Status:** [ACTIVE RESEARCH: 2026-04-20]
**Objective:** Evaluate Tari as a potential Layer 1/2 substrate for NyxForge bond issuance and settlement.

---

## 1. Executive Summary
Tari is a high-performance, privacy-focused digital assets protocol designed to handle complex, programmable instruments. It leverages a dual-layer architecture (Minotari L1 + Ootle L2) and secures itself via merge-mining with the Monero (XMR) network.

## 2. Technical Architecture
### Layer 1: Minotari (XTM)
*   **Purpose:** Security, privacy, and base value transfer.
*   **Consensus:** Hybrid Proof-of-Work (PoW).
    *   **Merge-mining (RandomX):** Secured by the Monero network (zero marginal cost to XMR miners).
    *   **Standalone (SHA3x):** ASIC-friendly algorithm for independent security.
*   **Privacy:** Utilizes Mimblewimble for base layer confidentiality and scalability.

### Layer 2: Digital Assets Network / Ootle (XTR)
*   **Purpose:** Programmable smart contracts, digital assets, and high-throughput tapps.
*   **Consensus:** Sharded BFT (Byzantine Fault Tolerant) for performance (thousands of TPS).
*   **Programmability:** Uses Rust-based "Templates" compiled to WebAssembly (WASM).
*   **Economic Link (The Turbine):** XTR is minted by burning XTM 1:1, creating a deflationary sink for the L1 supply.

## 3. Tokenomics & Supply (April 2026)
*   **Initial Target Supply:** 21 Billion XTM.
*   **Circulating Supply:** ~4.52 Billion XTM (post-May 2025 launch).
*   **Emission Schedule:** Exponential decay (halving every ~3 years) with a perpetual **1% annual tail emission** (no hard cap, ensures long-term miner rewards).
*   **Allocation:** 70% Mining rewards, 30% Ecosystem/Backers/Contributors.

## 4. Governance & Network Evolution
*   **Mechanism:** Community-driven **Request for Comment (RFC)** system.
*   **RFC Lifecycle:** Draft -> Stable -> Implementation (Rust).
*   **Authority:** Technocratic/Meritocratic; major protocol upgrades require miner signaling (XMR/SHA3x miners).
*   **Documentation:** All design decisions are archived at [rfc.tari.com](https://rfc.tari.com).

## 5. NyxForge Integration Potential
### Alignment
*   **Privacy First:** Tari's Mimblewimble heritage aligns with NyxForge's anonymous social policy bond ethos.
*   **Censorship Resistance:** Permissionless Ootle templates allow for sovereign bond markets without gatekeepers.
*   **Settlement:** Minotari (L1) provides "hard money" security, while Ootle (L2) handles complex bond logic (matchmaking, listing).

### Next Research Steps
*   [ ] Analyze Ootle "Template" structure for `.bond` file compatibility.
*   [ ] Evaluate cross-chain oracle integration (Tellor/Chainlink) on the Ootle layer for bond settlement triggers.
*   [ ] Research "Agentic Payments" on Tari for automated bond payouts.
