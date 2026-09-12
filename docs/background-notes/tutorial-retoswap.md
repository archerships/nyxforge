# Tutorial: Purchasing XMR via RetoSwap (Haveno)

> Version: March 2026
> Strategy: Decentralized Atomic-style Swaps using 2-of-2 Multisig.

---

## 1. Introduction
RetoSwap is a primary production instance of the **Haveno** protocol. It is a non-custodial, P2P exchange that runs entirely over **Tor**. In 2026, it is the gold standard for swapping Bitcoin (BTC) for Monero (XMR) without KYC.

## 2. Initial Configuration
1.  **Download:** Get the RetoSwap client from the official GitHub (`retoaccess1/haveno-reto`).
2.  **Verify:** Always check the PGP signature of the installer to ensure it hasn't been tampered with.
3.  **Bootstrap:** On first launch, the app will generate a new Monero seed phrase. **Store this phrase securely.**
4.  **Sync:** The client will automatically connect via Tor. Wait for it to synchronize with the Monero network.

## 3. Preparing Your Account
1.  **Add Payment Method:** Go to **Account** > **Add New Account**.
2.  **Select Bitcoin:** Enter your BTC payout address (used for trade tracking/reconciliation).
3.  **Security Deposit:** Haveno requires a small amount of XMR as a "Security Deposit" to prevent trade griefing.
    *   **New User?** Look for listings with the **"No Deposit"** tag to get your first XMR.
    *   **Have XMR?** Deposit a small amount into the internal RetoSwap wallet.

## 4. Executing the BTC → XMR Swap
1.  **Browse Market:** Go to the **Market** tab and filter for "Buy XMR" using "Bitcoin" as the payment method.
2.  **Take Offer:** Click "Take Offer" on a suitable listing.
3.  **Deposit Phase:** The seller locks their XMR into a 2-of-2 multisig escrow address controlled by the RetoSwap protocol.
4.  **BTC Payment:** The app will provide you with a BTC address. Send the agreed amount of Bitcoin to this address.
5.  **Confirmation:** Wait for 2-3 Bitcoin network confirmations.
6.  **Release:** Once confirmed, the app (or the seller) will release the XMR from escrow directly into your internal RetoSwap wallet.

## 5. Security Posture
*   **Withdrawal:** After the swap, move your funds from the internal RetoSwap wallet to your primary NyxForge or Cake Wallet for long-term storage.
*   **Tor Default:** RetoSwap forces all traffic through Tor. Do not disable this feature.
*   **Arbitration:** If a seller fails to release funds after BTC is confirmed, you can open a dispute with a decentralized arbitrator within the app.
