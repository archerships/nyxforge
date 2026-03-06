# NyxForge — Privacy & Anonymity Design

> Version 0.2 — March 2026
> Subject: Shielded Architecture on a Public, Permanent Ledger (Arweave AO)

---

## 1. The Challenge of Public Persistence

The Arweave AO network is designed for **permanent, public message logs**. Every interaction with a NyxForge process is recorded forever and is publicly readable. To maintain the privacy standards of Monero and DarkFi, NyxForge must ensure that this public log reveals **zero metadata** about users, their holdings, or their transaction history.

## 2. The Shielded State Model (UTXO/ZK-Note)

NyxForge does not store a "balance table" (e.g., `Address A has 100 NYX`). Instead, it uses a **Shielded UTXO model** where the global state is reduced to two append-only sets:

| Set | Data Stored on AO | Purpose |
| :--- | :--- | :--- |
| **Commitment Tree** | `PedersenCommit(bond_id ‖ qty ‖ owner ‖ salt)` | Represents the existence of a bond note without revealing its contents. |
| **Nullifier Set** | `PRF(owner_secret, serial)` | Prevents double-spending by marking a note as "spent" without linking it to the original commitment. |

### 2.1 Local-First State
The raw data for every bond note (quantity, asset ID, owner key, randomness) is stored **exclusively on the user's local hardware** (commodity CPU/SSD). The AO network acts only as a **stateless verifier** of state transitions.

---

## 3. Zero-Knowledge State Transitions

Privacy is enforced through **Halo2 ZK-SNARKs**, generated locally by the user and verified by the AO Compute Unit (CU).

### 3.1 MINT (Issuance)
*   **Private Input:** Bond GoalSpec, quantity, owner secret key.
*   **Public Output:** New commitment on the AO ledger.
*   **Privacy Result:** Observers see a new bond issued but cannot see the quantity or the issuer's long-term identity.

### 3.2 TRANSFER (Trading)
*   **Private Input:** The old note, the new note, and the owner's secret key.
*   **Public Output:** Reveal the **Nullifier** of the old note; append the **Commitment** of the new note.
*   **Privacy Result:** The "transaction graph" is broken. An observer sees a note "die" and a new note "born," but they cannot link them together or determine the amount traded.

### 3.3 BURN (Redemption)
*   **Private Input:** The note being redeemed, the secret key, and the oracle quorum hash.
*   **Public Output:** Reveal the Nullifier; authorize the payout.
*   **Privacy Result:** Prevents "Redemption Correlation" attacks where the timing of an oracle result is linked to a specific user's payout.

---

## 4. Metadata & Network Privacy

The AO protocol's "Scheduler Units" (SUs) and "Compute Units" (CUs) can see the IP addresses of message senders. NyxForge mitigates this through **Anonymizing Gateways**:

1.  **Tor-First Submission:** The NyxForge CLI and Browser UI route all AO messages through the Tor network by default.
2.  **DarkFi P2P Integration:** Messages can be gossiped over DarkFi’s native P2P network (which uses Tor/Nym routing) before being "anchored" to an AO process by a federated gateway.
3.  **Holographic Obfuscation:** Because AO state is recomputed from logs, a user can submit a message to *any* scheduler, making it difficult to build a consistent profile of a single user's activity.

---

## 5. Goal Text Privacy & View Keys

### 5.1 The Problem

Bond goal text (`GoalSpec.title`, `GoalSpec.description`, `GoalMetric.data_id`) describes *what* the bond is measuring. For many bonds this information is sensitive — a lifebond's health metrics, the identity of the subject, or proprietary data sources. Publishing this in plaintext on Arweave's permanent ledger is unacceptable.

### 5.2 Goal Visibility Levels

Each `GoalSpec` carries a `visibility` field with two variants:

| Variant | On-chain Storage | Who Can Read |
| :--- | :--- | :--- |
| `GoalVisibility::Private` | ChaCha20-Poly1305 ciphertext | Anyone holding the bond view key |
| `GoalVisibility::World` | Plaintext | Anyone |

Default is `Private`. Issuers opt into `World` explicitly (e.g., for public impact bonds where transparency is a selling point).

### 5.3 Bond View Keys

The view key is derived from the issuer's spend key and the bond ID, so each bond has an independent view key:

```
bond_view_key = blake3(issuer_spend_key ‖ bond_id)
```

The issuer can share `bond_view_key` with:
- Individual bondholders (to verify goals before purchase)
- Regulators or auditors (selective disclosure)
- The public (equivalent to `World` visibility without changing the on-chain record)

Sharing the view key does **not** expose the issuer's spend key or any other bond's goals.

### 5.4 Encryption Scheme

Goals are encrypted with **ChaCha20-Poly1305** (authenticated, streaming-friendly, no block-size constraints):

```
nonce     = random 12 bytes (stored alongside ciphertext)
plaintext = canonical JSON of GoalSpec fields
ciphertext, tag = ChaCha20Poly1305_Encrypt(bond_view_key, nonce, plaintext)
```

The commitment hash in the MINT circuit commits to the *plaintext* goal data (inside the ZK proof), so the ZK system enforces correctness of the encrypted goals without revealing them.

### 5.5 Oracle Access

Oracles that need to evaluate private goals receive the view key through an out-of-band channel (DarkFi encrypted DM or Tor-routed key exchange). The oracle decrypts the goal spec locally before fetching data. No plaintext goal data is ever sent to AO.

---

## 6. Oracle Attestation Privacy — Noise Bonds

### 6.1 The Linkability Problem

Even with encrypted goal text, an oracle attestation on the public ledger is a **timing signal**. An observer who knows (or guesses) which bonds are real can correlate:

- Attestation timestamp → real-world event (e.g., subject's death)
- Oracle key → oracle operator identity
- Cluster of attestations → "something happened to a subject"

This is analogous to traffic analysis attacks on VPNs: the *fact* of communication leaks information even when the *content* is encrypted.

### 6.2 Noise Bond Anonymity Set

NyxForge introduces **noise bonds** — protocol-level dummy bond series that oracles attest to on a fixed, public schedule regardless of real-world events. Noise attestations are cryptographically indistinguishable from real ones.

**Properties:**
- Fixed emission rate (e.g., N noise attestations per hour per oracle)
- Noise bond IDs are published in a registry so that bondholders can filter them out, but external observers cannot distinguish noise from signal without the registry key
- Real attestations are batched and delayed to align with the nearest noise emission window, limiting timing resolution to the window size (e.g., ±30 minutes)

**Analogy:** The Nym mixnet sends constant-rate dummy packets; NyxForge oracles send constant-rate dummy attestations. Both provide an anonymity set proportional to the steady-state traffic volume.

### 6.3 Implementation Sketch

```
NoiseBondConfig {
    emission_rate_per_hour: u32,   // e.g. 12 (one per 5 minutes)
    window_size_secs: u64,         // e.g. 300 (5 minutes)
    noise_bond_registry_key: [u8; 32],  // shared with bondholders, not public
}
```

An oracle participating in noise-bond mode:
1. Maintains a local queue of real attestations ready to submit.
2. At each window boundary, submits exactly `emission_rate_per_hour / (3600 / window_size_secs)` attestations, padding with noise if the real queue is empty.
3. Real attestations are indistinguishable from noise on the AO ledger (same size, same ZK proof structure, same oracle key).

---

## 7. Key Management & Unlinkability

NyxForge leverages the **Monero (XMR) Spend Key** as the root of all privacy.

*   **Stealth Addresses:** Every bond note commitment uses a one-time derived address. Even if two notes belong to the same user, their commitments on the AO ledger appear completely unrelated.
*   **Deterministic Derivation:** All DRK and NYX keys are derived from the XMR spend key. This ensures the user has a "Single Secret" to protect, while presenting an "Infinite Identities" profile to the public ledger.

---

## 8. Trust Model (Math vs. Hardware)

NyxForge explicitly rejects the use of Trusted Execution Environments (TEEs) or specialized hardware for privacy.

*   **Transparent Proofs:** We use **Halo2**, which requires no "trusted setup" (no toxic waste). The security of the system rests entirely on the mathematical hardness of the discrete logarithm problem over elliptic curves (Pasta curves).
*   **Commodity Verification:** Any standard CPU can verify a Halo2 proof. This ensures that the NyxForge state is verifiable by the community without needing to trust Intel, AMD, or any hardware manufacturer's "enclave."

---

## 9. Privacy Preservation Summary

| Feature | Protection Provided |
| :--- | :--- |
| **Commitments** | Hides balances and asset types. |
| **Nullifiers** | Hides which transaction is linked to which payout. |
| **ZK-SNARKs** | Hides the logic of the trade and the identity of the trader. |
| **Stealth Keys** | Prevents correlating multiple transactions to one wallet. |
| **Tor Routing** | Hides physical location and IP address. |
| **Monero Bridge** | Hides the source of the initial collateral (XMR/ZANO). |
| **Encrypted Goal Text** | Hides bond criteria (subject, data sources) from public ledger. |
| **Bond View Keys** | Enables selective disclosure of goal text without exposing spend key. |
| **Noise Bonds** | Hides oracle attestation timing; prevents correlation of real-world events to specific bond series. |
