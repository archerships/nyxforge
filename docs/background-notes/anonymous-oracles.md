# Open Source Anonymous Oracle Networks

There is currently no turnkey, production-ready "Chainlink for Anons" that meets all NyxForge constraints. Existing oracle networks (Chainlink, Pyth, Tellor) rely on **public reputation** and **staked identity** to ensure honesty, making them vulnerable to Sybil attacks in an anonymous environment.

However, several open-source protocols and emerging ZK networks provide the essential building blocks:

### 1. TLS-Notary (The fundamental building block)
*   **What it is:** A protocol that allows a "Prover" to prove that certain data exists on a Web2 website (via HTTPS) without revealing private session keys or credentials.
*   **Why it fits:** It is **MPC-based** and **ZK-based**, requiring no special hardware (No SGX/TEE) and running on standard CPUs.
*   **Anonymity:** The website being scraped has no idea who the Prover is, and the Prover can remain anonymous to the Verifier while still providing a mathematically unforgeable proof.
*   **Link:** [tlsnotary.org](https://tlsnotary.org/)

### 2. Reclaim Protocol (zkTLS Network)
*   **What it is:** A decentralized network of "witnesses" that verify Web2 data and generate ZK-proofs.
*   **Status:** Active SDK; moving toward a decentralized node network.
*   **Constraint Check:** Uses standard ZK-SNARKs and can run on commodity hardware.
*   **Link:** [reclaimprotocol.org](https://www.reclaimprotocol.org/)

### 3. Pado Network
*   **What it is:** Uses a combination of MPC and Interactive Zero-Knowledge (IZK) to attest to web data.
*   **Why it fits:** Aligns with the "Push" model where an anon oracle node fetches data, generates a proof, and pushes it to the chain.
*   **Link:** [padolabs.org](https://www.padolabs.org/)

### 4. 0rbit (AO Native)
*   **What it is:** A decentralized oracle network built specifically on AO (Arweave Computer).
*   **The Gap:** 0rbit is decentralized but not inherently anonymous. However, because it is on AO, its requests can be wrapped inside **Shielded AO** logic (ZK-SNARKs + Nullifiers) to anonymize the relationship between the bond and the data fetcher.
*   **Link:** [0rbit.co](https://0rbit.co/)

### 5. DarkFi's "Witness" Model
*   **Concept:** In DarkFi, an "Oracle" is simply a "Witness" that signs a statement. The identity is a **Stealth Address** derived from a Monero key.
*   **Skin in the Game:** Instead of social reputation, it uses mathematical enforcement. The oracle's anonymity is protected by ZK, but their **Stake (XMR/DRK)** is locked. If they provide a false attestation (proven via a dispute game), their stake is slashed automatically by the math.

---

### Comparison for NyxForge

| Technology | Hardware | Privacy Type | Maturity |
| :--- | :--- | :--- | :--- |
| **TLS-Notary** | Commodity | **Data + Session Privacy** | High (Protocol) |
| **Reclaim** | Commodity | Data Proofs | Medium (Network) |
| **0rbit** | Commodity | Transparency (AO) | Early (Network) |
| **NyxForge Stack** | **Commodity** | **Full Anonymity (ZK + P2P)** | **Alpha (Strategy)** |
