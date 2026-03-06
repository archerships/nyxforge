# NyxForge — Oracle Specification

> Version 0.1 — March 2026
> Subject: Decentralized, Private, and Century-Scale Data Verification for Social Policy Bonds

---

## 1. Introduction

NyxForge requires a specialized oracle infrastructure to resolve **Social Policy Bonds (SPBs)**. Unlike standard DeFi price feeds, these oracles must handle **private web data**, **complex qualitative outcomes**, and **multi-century timeframes (up to 200 years)** while adhering to the mandate of **commodity hardware** and **anonymity**.

---

## 2. Oracle Architecture: The Three-Tier Stack

To ensure intersubjective truth and censorship resistance, NyxForge employs a tiered resolution hierarchy capable of surviving 200 years.

| Tier | Mechanism | Purpose |
| :--- | :--- | :--- |
| **Tier 1: Optimistic** | AI-driven proposals with bonded stakes. | Efficient resolution of high-confidence outcomes (95% of cases). |
| **Tier 2: Escalation** | Economic "Bond-Doubling" games (Reality.eth model). | Deters "griefing" and filters out frivolous disputes through capital-at-risk. |
| **Tier 3: Arbitration** | Decentralized Courts (Kleros 2.0 / UMA DVM). | Final judgment for complex, qualitative, or high-stakes disputes. |

---

## 3. The "Long Now" (Multi-Century) Model

For bonds maturing in 10, 50, 100, or 200 years, the oracle system shifts from "Data Fetching" to **"Evidence Archiving."**

### 3.1 Sovereign Infrastructure & Persistence
Oracle nodes do not rely on a central server. They run on a decentralized network of **commodity hardware** (home servers, laptops, or decentralized clouds like Akash/Flux).
*   **Rotation:** No single computer is expected to run for 200 years. Instead, the **NYX Fair Launch Emission** (100-200 year decay) creates a perpetual incentive for a rotating cast of global participants to spin up new nodes.
*   **Heartbeat:** If one operator goes offline, the unclaimed rewards attract a new operator to take over the monitoring task on modern hardware.

### 3.2 Perpetual Evidence Archiving
*   **Archivists:** Participants incentivized by the **NYX Fair Launch** to pin signed reports, news, and scientific data to the Arweave Permaweb.
*   **Holographic Replay:** In the year 2200, the AO process resolves the bond by replaying two centuries of immutable evidence stored in its own log.

### 3.3 The Delphic Pattern (Institutional Resilience)
*   **Substitution:** If a data source (e.g., NOAA) ceases to exist, the Treasury DAO votes to substitute it with a modern equivalent index.
*   **Fallback:** If all automated metrics fail, the bond defaults to a decentralized jury of humans—the only "oracle" that has survived for millennia (the legal model).

### 3.4 Cryptographic Agility
Contracts include "Verifier Slots" to allow upgrading from Ed25519/ECDSA to **Post-Quantum Cryptography** (Lattice-based) as standards evolve over the centuries.

---

## 4. Privacy & Zero-Knowledge Attestation

NyxForge oracles utilize **Zero-Knowledge (ZK)** technology to prove outcomes without exposing raw sensitive data.

### 3.1 zkTLS (DECO / TLS-Notary)
Used for bonds linked to legacy Web2 portals (e.g., government databases).
*   **Action:** A prover logs into a portal and generates a ZK-proof that a specific session value (e.g., "Unsheltered count = 450") existed.
*   **Result:** The oracle verifies the proof but never sees the user's login credentials or the full page content.

### 3.2 zkVM (RISC Zero / Succinct SP1)
Used for bonds requiring complex statistical verification.
*   **Action:** The "GoalMetric" logic is compiled into a zkVM image. The data provider runs this image locally on their private dataset.
*   **Result:** The provider broadcasts a ZK-proof of the final calculation result (e.g., "Recidivism rate dropped by 5.2%").

---

## 4. The "Long Now" (Century-Scale) Model

For bonds maturing in 10, 50, or 100 years, the oracle system shifts from "Data Fetching" to **"Evidence Archiving."**

### 4.1 Perpetual Evidence Archiving
*   **Archivists:** Participants incentivized by the **NYX Fair Launch** to pin signed reports, news, and scientific data to the Arweave Permaweb.
*   **Holographic Replay:** In the year 2100, the AO process resolves the bond by replaying a century of immutable evidence stored in its own log.

### 4.2 The Delphic Pattern (Institutional Resilience)
*   **Substitution:** If a data source (e.g., NOAA) ceases to exist, the Treasury DAO votes to substitute it with a modern equivalent index.
*   **Fallback:** If all automated metrics fail, the bond defaults to a decentralized jury of humans—the only "oracle" that has survived for millennia (the legal model).

### 4.3 Cryptographic Agility
Contracts include "Verifier Slots" to allow upgrading from Ed25519/ECDSA to **Post-Quantum Cryptography** (Lattice-based) as standards evolve over the century.

---

## 5. Economic Model & Payments

Oracles are compensated through a multi-modal payment system designed for sustainability.

### 5.1 External Infrastructure Fees
*   **0rbit:** Universal Web2 data fetching costs **~1 $0RBT per request** (<$0.05 USD).
*   **RedStone:** DeFi price feeds (for collateral valuation) use **micro-payments** in $RED or ETH.
*   **Message Capacity:** Oracle processes must hold **$AO tokens** to maintain message throughput (1 AO = 10 msgs/hr).

### 5.2 NyxForge Native Rewards
*   **Proof of Accuracy ($NYX):** Oracles earn a continuous emission of NYX tokens based on their attestation reputation and consensus alignment.
*   **Maintenance Bounties (XMR Yield):** For Long Now bonds, a portion of the bond's **Monero yield** is diverted into an endowment that pays annual bounties to archivists and monitors for up to 100 years.

---

## 6. Implementation Strategy

| Component | Responsibility | Technical Stack |
| :--- | :--- | :--- |
| **Market Value** | Assessing collateral worth every 5 mins. | RedStone Bolt / 0rbit |
| **Metric Fetching** | Retrieving social outcomes from Web2. | 0rbit / zkTLS |
| **Verification** | Proving complex metric logic. | zkVM (Rust) |
| **Dispute** | Resolving qualitative outcomes. | Kleros / UMA |

---

## 7. Security Mandate

1.  **Commodity Hardware:** All oracle and verifier logic must run on standard CPUs (WASM).
2.  **No TEEs:** Avoid hardware trust assumptions (SGX/TDX).
3.  **Holographic State:** All oracle inputs must be logged permanently to Arweave to allow for state reconstruction across decades.
