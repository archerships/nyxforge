# NyxForge — Monero-Based Funding Strategy

> Strategic plan for sustainable, decentralized funding of NyxForge development.
> Dated: March 2026

---

## 1. The "Dev-as-Miner" Model (Proof of Quality)

NyxForge rejects traditional venture capital in favor of a **"Proof of Quality"** mechanism within its AO Fair Launch. Technical contributions are treated as a form of "mining" the native **NYX token**.

### 1.1 Automated Bounty Emission
*   **Mechanism:** Direct minting of NYX to developers whose code is merged into the core repository.
*   **Verification:** Pull requests are linked to an AO-integrated Git process. Once a PR is merged (verified by a multisig of lead maintainers or a community vote), a scheduled amount of NYX is emitted.
*   **Incentive:** Developers earn "equity" in the system they are building, aligning long-term incentives without a centralized payroll.

---

## 2. Yield-Direction (The Treasury Engine)

NyxForge can leverage the inherent yield-generating capabilities of the Monero ecosystem to fund its ongoing operations (e.g., audits, infrastructure, soft services).

### 2.1 The "Nyx-Vault" Model
*   **Mechanism:** Users bridge **XMR (Monero)** or **ZANO** into a dedicated **NyxForge Yield Vault** on the Arweave AO network.
*   **Strategy:** The vault automatically provides this liquidity to the **Serai DEX** (XMR/BTC pools) or **Shade Protocol** (sXMR/SILK pools).
*   **Funding Split:** 
    *   **80% of the yield** is returned to the user (the capital provider).
    *   **20% of the yield** is directed to the **NyxForge Treasury DAO**.
*   **User Reward:** Users are further compensated for this 20% "donation" with a higher emission rate of **NYX tokens** via the AO Fair Launch.

---

## 3. Merge-Mining for Development

As a native feature of the NyxForge node, merge-mining provides a continuous, low-overhead stream of funding.

### 3.1 The "Tari-Tax" (Opt-in)
*   **Mechanism:** The built-in **nyxforge-miner** (RandomX) can be configured to share a small percentage (e.g., 2–5%) of its **Tari (XTM)** merge-mining rewards with the development treasury.
*   **Benefit:** This costs the user **zero XMR** and **zero extra electricity**. It uses the "sidecar" profit of the Tari network to fund the NyxForge ecosystem.

---

## 4. AO Fair Launch Stacking (Network Multiplier)

By participating in the **Arweave AO Fair Launch**, the NyxForge project can capture **AO tokens** to fund cross-chain infrastructure.

### 4.1 Collateral-to-AO Bridge
*   **Mechanism:** The project uses its **Treasury XMR-LP tokens** (from the Nyx-Vaults) as collateral in the AO Fair Launch.
*   **Result:** The project earns **AO tokens**, which are used to pay for the "Compute Units" (CUs) required to run the permanent NyxForge processes on the Arweave network.
*   **Self-Sustainability:** This creates a circular economy where the project's own collateral pays for its network hosting costs.

---

## 5. The "Privacy Bounty" Fund (XMR/ZANO)

For critical security tasks (e.g., Halo2 circuit audits), the project will maintain a **Shielded Bounty Fund**.

### 5.1 Anonymous Audits
*   **Mechanism:** High-value security bounties paid directly in **XMR** or **ZANO**.
*   **Purpose:** Attracts the world's best privacy researchers who may wish to remain anonymous for legal or security reasons.
*   **Funding Source:** Seeded by initial community "Angel Donations" and supplemented by the 20% yield-direction model.

---

## 6. Governance: The Treasury DAO

All funds (NYX, AO, XMR, XTM, ZANO) are managed by the **NyxForge Treasury DAO**, a permanent AO process.

*   **Transparency:** All incoming funding and outgoing grants are recorded permanently on the Arweave holographic log.
*   **Anonymity:** Voting is done using **ZK-votes**, ensuring that participants can direct the project's future without revealing their identity or stake size.
*   **No VCs:** By relying on these math-based yield and emission mechanisms, NyxForge remains beholden only to its users and developers.
