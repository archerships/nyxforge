# Tutorial: Purchasing XMR via Cake Wallet

> Version: March 2026
> Strategy: Using self-custodial "Buy & Swap" for maximum reliability.

---

## 1. Introduction
Cake Wallet is a non-custodial, privacy-focused wallet available for iOS and Android. In 2026, it serves as a powerful hub for acquiring Monero (XMR) through integrated fiat on-ramps and internal exchanges.

## 2. Preparation
1.  **Install:** Download Cake Wallet from the iOS App Store or Google Play Store.
2.  **Backup:** Create a new Monero wallet. **Write down your 25-word seed phrase** on physical paper. This is your only recovery method.
3.  **Sync:** Wait for the "Blocks Remaining" status to hit zero. Your wallet must be synchronized to see your balance.

## 3. Method A: The "Bridge Swap" (LTC → XMR)
Direct XMR purchases are often restricted by card processors. Using Litecoin (LTC) as a bridge is the most reliable and cost-effective method.

1.  **Create a Litecoin Wallet:** Tap the Menu > **Wallets** > **Create New Wallet** > Select **Litecoin**.
2.  **Buy Litecoin:**
    *   Tap the **Buy** button on the home screen.
    *   Select **Alchemy Pay** or **MoonPay**.
    *   Enter your fiat amount (e.g., $100 USD).
    *   Complete the provider's KYC and pay via **Credit/Debit Card**.
3.  **Exchange for Monero:**
    *   Once your LTC arrives, tap the **Exchange** tab at the bottom.
    *   Set "Convert From" to **Litecoin (LTC)**.
    *   Set "Convert To" to **Monero (XMR)**.
    *   Select an aggregator (e.g., **Trocador** or **ChangeNOW**).
    *   Confirm the swap.
4.  **Completion:** Your XMR will arrive in your Monero wallet in 10–20 minutes.

## 4. Method B: Direct Fiat Purchase (If Available)
In supported regions, you can buy XMR directly with a card.

1.  **Open Monero Wallet:** Ensure you are in your XMR wallet interface.
2.  **Tap Buy:** If **Monero (XMR)** appears in the "Buy" menu, select it.
3.  **Complete KYC:** Provide the required ID to the on-ramp partner (e.g., Blockbuy).
4.  **Receive:** Your XMR will appear after 10 network confirmations.

## 5. Privacy Tip: Restoring Anonymity
Purchasing with a credit card leaves a paper trail with the on-ramp provider. 
*   **Action:** Once you receive your XMR in Cake Wallet, send it to a **new subaddress** (starts with `8`) within your wallet or to a completely different wallet (like your NyxForge node wallet). 
*   **Result:** This breaks the on-chain link between the KYC purchase and your future NyxForge bond activity.
