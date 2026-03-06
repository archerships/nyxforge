# NyxForge — Long Now Oracle Design

> Version 0.1 — March 2026
> Subject: Architecting Decentralized Oracles for Multi-Decade and Centennial Resolution

---

## 1. The Problem of Temporal Decay

Traditional decentralized oracles (e.g., Chainlink, Pyth) are designed for low-latency, high-frequency data (price feeds). They are fundamentally unsuitable for **Social Policy Bonds**, which may require resolution in 10, 50, or 100 years. Over these timescales, three forms of decay occur:

1.  **Institutional Decay:** The data source (e.g., `climate.gov`, `who.int`) or the oracle company itself ceases to exist.
2.  **Technical Decay:** The API standard (REST/JSON), the network protocol (IPv4/v6), or the blockchain hosting the contract becomes obsolete.
3.  **Cryptographic Decay:** Current signature schemes (Ed25519/ECDSA) are compromised by quantum computing or algorithmic breakthroughs.

---

## 2. The "Shielded AO" Perpetual Architecture

NyxForge solves for longevity by shifting from a **"Push"** model (where an oracle sends data) to a **"Holographic Archive"** model (where the contract evaluates permanent evidence).

### 2.1 Pillar 1: Perpetual Evidence Archiving (Arweave)
Instead of a single "Resolution Event," NyxForge incentivizes a continuous stream of evidence.
*   **Archivists:** A new participant role rewarded via the **NYX Fair Launch**.
*   **The Action:** Archivists permanently pin cryptographically signed reports, news, and raw data to the Arweave Permaweb.
*   **The Log:** The AO process for a bond becomes a "curated archive." By the year 2100, the "truth" is not fetched from the web; it is computed from 75 years of immutable evidence stored on-chain.

### 2.2 Pillar 2: The Delphic Fallback Pattern
To avoid "Link Rot," NyxForge uses a tiered resolution hierarchy:

| Tier | Mechanism | Trigger |
| :--- | :--- | :--- |
| **Tier 1: ZK-Attestation** | Automated verification of a ZK-proof from a known data provider (e.g., NOAA). | Primary source is active and matches the `GoalSpec`. |
| **Tier 2: Index Substitution** | A DAO vote updates the `data_id` to a modern equivalent. | Primary source is inactive for >2 years. |
| **Tier 3: Intersubjective Truth** | Escalation to a decentralized human jury (Kleros-style). | All automated data fails or is disputed. |

*Note: Human consensus (the legal system) is the only "oracle" that has successfully survived for thousands of years.*

---

## 3. Economic Sustainability: The Yield Endowment

An oracle will only monitor a bond for a century if it is profitable to do so.

*   **The Endowment:** When a Long Now bond is issued, a percentage (e.g., 5%) of the **XMR/ZANO collateral** is diverted into a **Perpetual Yield Vault**.
*   **The Bounty:** This vault generates a continuous return (via Serai or Zano DeFi). Every year, a "Maintenance Bounty" is paid to any node that successfully submits the year's evidence to the AO process.
*   **Result:** The bond is a self-funding entity that pays for its own monitoring across generations.

---

## 4. Post-Quantum Readiness (Agile Crypto)

To survive a century, the AO process must be "Cryptographically Agile."

*   **Logic Slots:** The `nyxforge-contract` includes "Verifier Slots." 
*   **The Upgrade:** If Ed25519 is compromised in 2045, the Treasury DAO can vote to add a **Lattice-based Signature Verifier** to the contract. 
*   **Persistence:** Existing bond notes remain valid, but new proofs must use the upgraded, quantum-resistant standards.

---

## 5. Summary of the "Long Now" Stack

| Layer | Technology | Longevity Strategy |
| :--- | :--- | :--- |
| **Storage** | Arweave | 200-year endowment-backed permanence. |
| **Compute** | AO (WASM) | Deterministic replay from permanent logs. |
| **Data** | zkTLS / DECO | Proof of session integrity without API cooperation. |
| **Resolution** | Human Jury | Subjective fallback for institutional collapse. |
| **Incentive** | XMR/ZANO Yield | Continuous payment for multi-generational nodes. |

---

## 6. Practical Implementation Steps

1.  **Define `EvidenceBundle`:** A struct in `nyxforge-core` that can hold heterogeneous data (IPFS CIDs, ZK-proofs, Arweave TxIDs).
2.  **Implement `MaintenanceBounty`:** Logic in `nyxforge-contract` to trigger annual payouts from the yield vault.
3.  **Audit `no-std` Rust:** Ensure the core logic is strictly platform-agnostic to survive the eventual death of current OS architectures.
