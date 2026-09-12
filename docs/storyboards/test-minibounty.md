# Storyboard: 5-Minute Fast-Cycle Test (Flash Bounty)

> **Subject:** Rapid end-to-end verification of the NyxForge v2.0 logic.
> **Date:** April 2026

This storyboard defines the standard "Short-Now" test path to verify ZK-circuits, optimistic disputes, and state transitions without waiting for multi-century deadlines.

---

### Scene 1: The Fast Forge (Minute 0:00)
*   **Action:** Developer Alice creates a "Hello World" test bounty.
*   **The Command:** `nyxforge-cli bounty create`
*   **Input:** Alice enters `+5m` for the deadline and `test.ping` for the metric.
*   **Result:** 10 XMR locked. **Bounty ID `0xTEST123`** is broadcast to the local node.

### Scene 2: The Flash Sale (Minute 0:30)
*   **Action:** Developer Bob buys 5 units of the test bounty.
*   **The Command:** `nyxforge-cli bounty buy 0xTEST123 5`
*   **Result:** Bob now holds ZK-notes for 50% of the bounty's face value.

### Scene 3: The Robotic Ping (Minute 1:00)
*   **Action:** Alice simulates a successful robotic oracle check.
*   **The Command:** `nyxforge-cli bounty dev mock-attest 0xTEST123 True`
*   **Result:** The UI updates the status badge to show "Metric: True (Last checked 10s ago)."

### Scene 4: The Speed Dispute (Minute 2:00)
*   **Action:** Alice asserts the goal is met to trigger the "Optimistic Game."
*   **The Command:** `bounty assert 0xTEST123 True ar://test-proof`
*   **The Challenge:** Bob immediately disputes her assertion to test the escalation path.
*   **The Command:** `bounty challenge 0xTEST123 ar://dispute-reason`
*   **Result:** The bounty state moves to **Challenge**. A 60-second window for jury selection begins.

### Scene 5: The Flash Jury (Minute 3:30)
*   **Action:** Alice uses a dev shortcut to resolve the dispute instantly.
*   **The Command:** `bounty dev summon-jury 0xTEST123`
*   **The Vote:** `bounty jury vote 0xTEST123 True`
*   **Result:** The `JuryActor` finalizes the state. The bounty transitions to **Redeemable**.

### Scene 6: The Quick Cash-out (Minute 5:00)
*   **Action:** The 5-minute timer expires. Bob runs the redemption logic.
*   **The Command:** `bounty redeem 0xTEST123`
*   **Result:** Bob’s local wallet receives his XMR payout. The bounty moves to **Settled**.
