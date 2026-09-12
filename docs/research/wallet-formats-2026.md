# Research Brief: Cryptocurrency Wallet Formats & Interoperability (2026)

**Status:** [ACTIVE RESEARCH: 2026-04-21]
**Objective:** Define a future-proof storage standard for the NyxForge Hub by evaluating current proprietary formats and emerging industry-wide common wallet proposals.

---

## 1. Landscape of Legacy/Proprietary Formats

| Wallet / Ecosystem | Format | Encryption / KDF | Interoperability |
| :--- | :--- | :--- | :--- |
| **Monero (Cake/Feather)** | `.keys` (JSON) + Cache | ChaCha20 / CryptoNight | High (within XMR ecosystem) |
| **Bitcoin (Electrum)** | `.json` | AES / PBKDF2 | Low (Electrum-specific) |
| **Bitcoin Core** | `wallet.dat` (SQLite/BDB) | AES-256-CBC | Very Low |
| **Tari (Minotari)** | `wallet.sqlite3` | SQLite encryption | Native only |

### Key Takeaway:
Proprietary formats create "walled gardens," making it difficult for users to migrate metadata (labels, notes, transaction history) even when they possess the underlying BIP-39/Polyseed recovery phrases.

---

## 2. Emerging Common Wallet Standards (2026 Proposals)

The industry is moving toward a **Common Stack** to ensure long-term data preservation and sovereign migration.

### A. Standard Encrypted Wallet Payload (Draft BIP)
A universal serialization format designed to store the entire state of a wallet.
*   **Format:** CBOR (Concise Binary Object Representation).
*   **Scope:** Includes root secrets, **Output Descriptors**, UTXO set, and extensive metadata (transaction labels).

### B. Standard Encryption Envelope (Draft BIP)
A standardized "security shell" for wallet files to ensure library-agnostic decryption.
*   **KDF:** **Argon2id** (replacing older KDFs like CryptoNight or PBKDF2).
*   **Encryption:** **XChaCha20-Poly1305** (Authenticated Encryption with Associated Data).

### C. Output Descriptors (BIP-380 series)
Descriptors provide a universal human-readable "blueprint" for how a wallet builds addresses.
*   **Impact:** Acts as a universal receipt; any compliant software can recreate a complex multisig or timelocked wallet from its descriptor string.

---

## 3. Advanced Recovery Standards

*   **BIP-128 (Timelock-Recovery):** A standard format for storing cold-storage recovery plans, critical for inheritance and self-custody "dead man switches."
*   **BIP-392 (Silent Payments):** Extends descriptors to support reusable, privacy-preserving Bitcoin addresses, ensuring migration doesn't break incoming payment detection.

---

## 4. Strategic Recommendation for NyxForge

To ensure the NyxForge Hub becomes the primary "Sovereign Orchestrator," the internal `.bounty` and `.wallet` files should implement the **Standard Encryption Envelope** spec:

1.  **Standardized Security:** Use **Argon2id** for all password-to-key derivations.
2.  **Containerized Metadata:** Use **CBOR** to wrap Monero `.keys` data alongside NyxForge bounty metadata.
3.  **Cross-Compatible Recovery:** Always export an **Output Descriptor** as part of the backup process.

---

## 5. Next Steps
*   [ ] Draft the technical specification for the `NyxEnvelope` container using Argon2id.
*   [ ] Prototype a converter for `.keys` -> `NyxEnvelope` (CBOR).
*   [ ] Integrate BIP-128 support for automated bounty redemption upon maturity.
