# Tutorial: Purchasing XMR via Vexl

> Version: March 2026
> Strategy: "Web of Trust" P2P model for Bitcoin, followed by a privacy bridge to Monero.

---

## 1. Introduction
Vexl is a Bitcoin-focused P2P marketplace that uses your phone's social graph to find trusted trade partners. It is strictly No-KYC. To get Monero, you buy Bitcoin on Vexl and then "bridge" it to XMR.

## 2. Setup & Social Graph
1.  **Install:** Download Vexl via TestFlight (iOS) or APK/Play Store (Android).
2.  **Privacy Profile:** Use a pseudonym and a secondary phone number for registration if desired.
3.  **Sync Contacts:** Vexl hashes your contacts locally to show you offers from "friends" or "friends-of-friends."
    *   **Pro Tip:** Add a known Bitcoin community number to your contacts to see a wider pool of vetted "friend-of-friend" offers.

## 3. Step 1: Buying Bitcoin (BTC)
1.  **Find Offer:** Browse "Sell BTC" offers in your network.
2.  **Select Method:** Choose **Cash in Person** or **Instant Bank Transfer** (e.g., Revolut, SEPA).
3.  **Chat:** Open an encrypted chat with the seller. Finalize the amount and payment details.
4.  **Pay:** Send the fiat payment.
5.  **Receive:** Once confirmed, the seller releases the BTC to your Vexl (or external) wallet.

## 4. Step 2: The Monero Bridge (BTC → XMR)
To convert your No-KYC Bitcoin to Monero, use one of these 2026 methods:

### **Method A: Atomic Swaps (High Privacy)**
Use a tool like **Eigenwallet** or [UnstoppableSwap.net](https://unstoppableswap.net).
*   **Action:** Perform a peer-to-peer swap of BTC for XMR without any middleman or centralized service. 
*   **Result:** Mathematically guaranteed security; no KYC required.

### **Method B: In-Wallet Exchange (Ease of Use)**
Transfer your Vexl BTC to **Cake Wallet**.
*   **Action:** Use the "Exchange" tab in Cake Wallet to convert BTC to XMR via a provider like **Trocador**.
*   **Result:** Fast and simple, though it relies on third-party aggregators.

## 5. Security Mandates
*   **No KYC:** If a seller on Vexl asks for your ID, cancel the trade immediately and report them.
*   **Use Tor:** Ensure your Monero wallet and Vexl sessions are routed through Tor or a trusted VPN to mask your IP address.
*   **Subaddresses:** Always receive your final XMR into a fresh **subaddress** (starting with `8`).
