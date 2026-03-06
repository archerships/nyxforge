# NyxForge — Liquidity & Interoperability Design

> Version 0.1 — March 2026
> Subject: Private Collateral On-boarding via Cake Wallet, RetoSwap, and Vexl

---

## 1. Overview

To ensure NyxForge remains accessible and decentralized, it must provide seamless, private pathways for users to acquire and lock collateral (XMR/ZANO). The **Liquidity Hub** architecture allows NyxForge to interoperate with leading privacy-preserving wallets and exchange protocols without requiring a centralized backend or compromising user anonymity.

---

## 2. The Liquidity Hub Architecture

The Liquidity Hub is a modular component within the `nyxforge-wallet` crate that coordinates funding requests between the NyxForge node and external liquidity providers.

### 2.1 Component Stack

| Layer | Function | Implementation |
| :--- | :--- | :--- |
| **User Interface** | Funding Wizard | Flutter WASM / CLI |
| **Coordination** | Liquidity Hub Module | Rust (Trait-based) |
| **Transport** | Deep Links / RPC | URI Schemes / Tor-routed SDKs |
| **Verification** | Blockchain Scanner | View Key monitoring (monerod) |

---

## 3. Integration Strategies

### 3.1 Cake Wallet (URI / Deep-Linking)
Cake Wallet serves as the primary "Mobile/Desktop Shield" for users who prefer a managed wallet experience.
*   **Mechanism:** Standardized `monero:` and `cake-wallet:` URI schemes.
*   **Workflow:** 
    1.  NyxForge generates a unique **Stealth Address** for the bond.
    2.  NyxForge triggers a deep-link: `monero:<stealth_address>?amount=<qty>&recipient=NyxForgeBond`.
    3.  Cake Wallet opens automatically with pre-filled details for user confirmation.
*   **Privacy:** No metadata is shared between apps beyond the address and amount.

### 3.2 RetoSwap (Native Atomic Swaps)
RetoSwap provides trustless, cross-chain liquidity (e.g., BTC ↔ XMR) directly within the NyxForge application.
*   **Mechanism:** Integration of the **Farcaster Protocol** or RetoSwap Rust SDK.
*   **Workflow:**
    1.  User selects "Swap BTC for Collateral."
    2.  NyxForge starts a local RetoSwap client and connects to a peer (over Tor).
    3.  The atomic swap executes; the resulting XMR is sent directly to the bond's internal lockbox.
*   **Privacy:** Peer-to-peer, math-based security; no centralized exchange or KYC.

### 3.3 Vexl (P2P Offer Bridge)
Vexl facilitates private, local P2P trades (Cash/Bank transfer for XMR).
*   **Mechanism:** Offer Signature & Metadata Import.
*   **Workflow:**
    1.  User browses Vexl for a local seller.
    2.  Vexl generates an "Offer Signature" containing the trade parameters.
    3.  NyxForge imports this metadata to prepare the node for the incoming XMR and associate it with the correct bond issuance.
*   **Privacy:** Leverages the user's existing social network trust via the Vexl protocol.

---

## 4. Shielded Verification Loop

To maintain decentralization, NyxForge does not trust external apps to report "success." It verifies every funding event independently.

1.  **Generation:** Every funding request uses a **one-time Stealth Address** derived from the user's master Monero key.
2.  **Monitoring:** The NyxForge background scanner (via `monerod`) watches the blockchain for incoming transactions to that specific stealth address.
3.  **Activation:** Once the transaction reaches the required confirmation depth (e.g., 10 blocks), the NyxForge node automatically advances the bond from `Draft` to `Active` on the Arweave AO network.

---

## 5. Security & Privacy Mandates

*   **No Linkability:** Under no circumstances shall multiple bonds be funded using the same sub-address. This prevents "Portfolio Correlation" attacks.
*   **Tor Routing:** All communication with RetoSwap peers or Vexl gateways must be routed through **Tor** or the **DarkFi P2P** network to mask the user's IP address.
*   **Commodity Only:** Swap and coordination logic must be pure software, requiring no TEEs (SGX/TDX) or specialized hardware.
*   **Zero-Persistence:** External API keys or session data from providers must never be stored in the permanent `node_data/` directory.

---

## 6. Implementation Roadmap

1.  **Phase 1:** Implement URI handler in the Flutter UI for Cake Wallet integration.
2.  **Phase 2:** Define the `LiquidityProvider` Rust trait and `StealthAddress` generator.
3.  **Phase 3:** Integrate the RetoSwap/Farcaster Rust client into `nyxforge-node`.
4.  **Phase 4:** Add Vexl offer parsing to the bond creation wizard.
