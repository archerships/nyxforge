# NyxForge — AO Fair Launch & Persistence Strategy

> Strategic research on adopting the Arweave Computer (AO) architecture.
> Dated: March 2026

---

## 1. The NyxForge (NYX) Fair Launch Model

NyxForge will adopt the **AO Fair Launch** mechanism to ensure 100% of the NYX token supply is distributed to value-add participants, with no pre-mine or venture capital allocation.

### 1.1 The Five Pillars of Contribution

| Pillar | Contribution Type | Mining Mechanism |
| :--- | :--- | :--- |
| **1. Capital** | Bridging XMR/ZANO for bond collateral | **Proof of Liquidity:** Emission based on yield/time locked in AO treasury processes. |
| **2. Technical** | Code, security fixes, ZK circuit audits | **Proof of Quality:** Automated NYX minting via AO-integrated Git/Radicle pull requests. |
| **3. Soft Service** | Legal, marketing, lobbying, advocacy | **Proof of Value:** DAO-voted rewards based on on-chain Impact Reports. |
| **4. Oracles** | Providing verifiable real-world data | **Proof of Accuracy:** Staking-backed rewards for consensus-aligned attestations. |
| **5. Security** | Providing compute/storage for the node | **Proof of Security:** Rewards for running dedicated AO Compute Units (CUs). |

---

## 2. Permanent Persistence on AO

NyxForge will be implemented as a series of **Permanent Processes** on the Arweave Computer (AO).

### 2.1 Holographic State
*   **Immutability:** Every message and state transition is stored permanently on Arweave.
*   **Self-Healing:** If any node goes offline, the process state is reconstructed by any new Compute Unit (CU) by replaying the permanent message logs.
*   **Lifespan:** The NyxForge protocol exists for as long as the Arweave network exists (designed for 200+ years).

### 2.2 Unbounded Execution
*   **No Gas Limits:** Unlike EVM chains, AO has no protocol-enforced gas limits. High-compute tasks (e.g., massive ZK proof verification) can run as long as the user pays the CU marketplace.
*   **Hyper-Parallelism:** Each Social Policy Bond can operate as its own independent AO process, preventing network congestion.

### 2.3 Autonomous Lifecycle (Cron)
*   **Self-Triggering:** NyxForge uses **AO Cron messages** to "wake up" at scheduled intervals.
*   **Automation:** This enables truly autonomous bond expiry checking, NYX token emission, and fee sweeps without human intervention.
*   **Scope:** AO Cron is used for *lifecycle management* (expiry, slashing, emission schedules) — **not** for oracle attestation triggering. See §2.4.

### 2.4 Oracle Attestation Model — Push, Not Poll

NyxForge oracles use a **push model**: oracle nodes run as always-on processes that monitor real-world data sources continuously, and *push* a signed attestation to the AO bond process the moment a goal condition is observed.

**Why push over AO Cron pull:**

| Concern | AO Cron (Pull) | Oracle Push |
| :--- | :--- | :--- |
| **Timing linkability** | Cron fires on a public schedule — observers can bracket *when* a condition was met to the cron window. | Oracle chooses when to submit (further obscured by noise bonds). |
| **Latency** | Bounded below by the cron interval; a goal met 1 second after a cron tick waits a full interval. | Near-instant; oracle submits as soon as the condition clears. |
| **Cost** | Every cron tick costs compute even when nothing changed. | Compute only spent when an event actually occurs. |
| **Complexity** | Bond process must re-fetch and re-evaluate data inside the AO sandbox (requires trusted HTTP adapters). | Data fetch stays off-chain in the oracle node; AO only verifies the signed attestation. |

**Push flow:**

```
Oracle node                   AO Bond Process
    │                               │
    ├── poll data sources ──────────┤
    │   (every poll_interval_secs)  │
    │                               │
    ├── condition met ──────────────┤
    │   sign attestation            │
    │   (optionally delay to        │
    │    next noise window)         │
    │                               │
    └── send AO message ───────────►│ verify signature
                                    │ check nullifier
                                    │ update state → Redeemable
```

**AO Cron is retained for:**
- Bond expiry enforcement (deadline passed, goal not met → state → Expired)
- Oracle slash adjudication (dispute window elapsed)
- NYX emission schedule

---

## 3. Hardware & Philosophy Alignment

*   **Commodity Hardware:** AO processes are WASM-based and run on standard CPUs (Linux/macOS).
*   **No TEE/ASIC Dependency:** Aligns with the mandate to avoid hardware-based trust (Intel SGX) or specialized mining hardware (ASICs).
*   **Monero Integration:** AO acts as the "Settlement and Logic Layer," while Monero (XMR) and Zano provide the "Anonymity and Funding Layer" via bridge/atomic swap messages.
