# Storyboard: The 200-Year Lifebond Loop

> **Subject:** High-level walkthrough of the NyxForge Social Policy Bond v2.0 lifecycle applied to Human Longevity.
> **Date:** April 2026

This storyboard illustrates how the NyxForge system implements the "Lifebond" model described by Archer T. Ships, leveraging anonymous issuance, XMR yield endowments, and hybrid human-robotic adjudication to align global incentives for multi-century health.

---

### Scene 1: The Forge (Anonymous Issuance)
*   **Action:** Archer, an anonymous individual, wants to incentivise research that will allow them to live to the year 2226.
*   **The Command:** Using `nyxforge-cli bounty create`, they run the interactive wizard to define **Lifebond 2226**.
*   **The Goal:**
    1.  **Metric:** `bio.human.vital_signs == true` (Robotic check via zkTLS).
    2.  **Qualitative:** "Subject is healthy, sentient, and experiencing subjective well-being" (Optimistic/Jury check).
*   **The Tech:** Archer locks 50,000 XMR into an escrow actor. This generates a **Bounty v2 ID** via deterministic Blake3 hashing. No one knows Archer is the issuer; the market only sees a new high-value goal.

### Scene 2: The Harvest (The XMR Yield Endowment)
*   **Action:** The bounty enters **Active** mode. The locked XMR principal is pointed to a **P2Pool endowment miner**.
*   **The Incentive:** The yield from this mining accumulates in the bounty's **Maintenance Endowment**.
*   **The Market:** Longevity researchers and biotech startups see the 50,000 XMR payout potential. They begin buying bounty units in the **NyxForge Dutch Auction** using `bounty buy`.
*   **The Price:** Initially, the market price is low (0.1 XMR), reflecting a high probability of failure.

### Scene 3: The Long Monitor (Century-Scale Robotic Oracles)
*   **Action:** It is now the year 2090. Archer is 100 years old. 
*   **The Robot:** Every month, a **Robotic Oracle Actor** uses **zkTLS (TLS-Notary)** to fetch a signed, private medical report from a sovereign clinic. 
*   **The Proof:** The oracle proves to the bounty contract: *"I have seen a vital-sign packet for Bounty ID [XYZ] with a valid signature from 2090-05-12."*
*   **The Dividend:** Because the goal is still "True," the bounty distributes a **Stability Dividend** (from the mining yield) to the bondholders. Researchers receive monthly funding to continue their work on Archer's health *now*, rather than waiting 200 years.

### Scene 4: The Crisis (Optimistic Challenge)
*   **Action:** It is the year 2150. A malicious actor asserts that the goal is "Not Met" in an attempt to crash the bounty price and buy up cheap units.
*   **The Command:** They run `bounty assert <id> False ar://malicious-evidence-hash`.
*   **The Defense:** The bondholders (now a massive consortium of scientists) immediately use `nyxforge-cli bounty challenge`, doubling the stake to freeze the assertion.
*   **Escalation:** The dispute triggers **Tier 3 Adjudication**.

### Scene 5: The Jury (Human Arbitration)
*   **Action:** An anonymous **Decentralized Jury** is summoned by the `JuryActor`.
*   **The Evidence:** Archer provides a **Proof-of-Life video**, time-stamped and pinned to **Arweave** in 2150. The jury uses `bounty jury status` to review the Arweave hash.
*   **The Verdict:** The jury votes 12-1 that Archer is "healthy and sentient." The challenger's stake is slashed and added to the Maintenance Endowment. The bounty remains **Active**.

### Scene 6: The Great Redemption (Year 2226)
*   **Action:** The expiry date (January 1st, 2226) arrives. Archer is alive and well at age 230+.
*   **Final Transition:** The oracle quorum performs the final check using `bounty status`. The Bounty State moves to **Redeemable**.
*   **Redemption:** Thousands of bondholders worldwide—the descendants of the original researchers—run `nyxforge-cli bounty redeem`. 
*   **The Payout:** They generate **ZK-BURN proofs** to prove ownership without revealing their identities. Each note is swapped for its 1.0 XMR face value from the principal.

---

### Implementation Success Factors

1.  **Maintenance Dividends:** Holders are paid for success in real-time via XMR yield, making long-dated bounties economically viable.
2.  **Incentive Alignment:** If Archer dies, the principal returns to Archer's estate (`return_address`), and holders get nothing. They are physically incentivised to keep Archer alive.
3.  **Absolute Privacy:** Archer's anonymity prevents targeted ransom or physical threats designed to force bounty expiration. 
4.  **DarkFi Ready:** The system consists of **Actor Messages** and **ZK-Notes**, capable of surviving political or infrastructure collapses over two centuries.
ope