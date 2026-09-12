# Research Brief: Quantum Resistance of NyxForge Cryptographic Stack
> Date: 2026-04-19
> Status: Background research -- not a build target for MVP or v2

---

## 1. Summary

No component of the NyxForge MVP cryptographic stack that depends on
asymmetric cryptography is quantum resistant. The symmetric and hash
primitives (AES-256, Blake3, SHA-256) retain acceptable post-quantum
security at ~128 bits. The asymmetric components -- DLEQ adaptor signatures,
Ed25519 keypairs, Schnorr signatures, and XMR itself -- are all broken by
Shor's algorithm on a cryptographically relevant quantum computer (CRQC).

Switching collateral from XMR to shielded ZEC does not improve the
situation. Both rely on elliptic curve discrete logarithm hardness by
construction. No deployed privacy coin today provides quantum-resistant
collateral locking.

Post-quantum verifier slots are already deferred to v3 in `doc/00_MVP.md
Section 3`. This brief documents the threat model so that decision is
well-founded.

---

## 2. Component-by-Component Assessment

### 2.1 Vulnerable (Shor's algorithm)

Shor's algorithm solves the elliptic curve discrete logarithm problem
(ECDLP) in polynomial time on a CRQC, breaking all of the following:

| Component | Algorithm | Note |
| :--- | :--- | :--- |
| DLEQ adaptor signatures | ECDLP (Curve25519) | Core collateral locking mechanism |
| `holder_pubkey` / `scalar_encrypted` | Ed25519 keypairs | Holder ownership model |
| `bounty prove` listing signatures | Schnorr (Ed25519) | Exchange ownership proof |
| XMR collateral | Ed25519 + RingCT + Bulletproofs | Entire Monero cryptographic stack |
| ZEC Sapling proofs | Groth16 over BLS12-381 | Pairing-based; relies on ECDLP |
| ZEC Sapling keypairs | Jubjub curve | ECDLP |
| ZEC Orchard proofs | Halo 2 over Pallas/Vesta | Inner product arguments; relies on discrete log |
| ZEC Orchard keypairs | Pallas curve | ECDLP |

### 2.2 Acceptable post-quantum security (Grover's algorithm only)

Grover's algorithm provides a quadratic speedup against symmetric
primitives, halving the effective bit-security. 256-bit outputs retain
~128 bits of post-quantum security, which is the current NIST acceptable
floor.

| Component | Algorithm | Post-quantum security |
| :--- | :--- | :--- |
| `bounty_id` derivation | Blake3-256 | ~128-bit |
| Evidence hashes | SHA-256 | ~128-bit |
| File encryption | AES-256 (SQLCipher) | ~128-bit |
| ZEC note encryption | ChaCha20-Poly1305 | ~128-bit |

---

## 3. The Shielded ZEC Comparison

Shielded ZEC was evaluated as an alternative collateral currency with
potentially better quantum properties. The conclusion: it is not materially
better.

ZEC's note encryption (ChaCha20-Poly1305) gives transaction amounts
marginally better quantum confidentiality than XMR's RingCT (which uses
Pedersen commitments -- ECDLP-based). An attacker harvesting shielded ZEC
transactions today cannot learn amounts even with a future CRQC, because
the encryption is symmetric.

However, the ability to SPEND -- which is what matters for bounty collateral
-- still requires ECDLP-based keys in both systems. A CRQC breaks spending
authority in XMR and ZEC equally.

For NyxForge purposes: XMR and ZEC are equivalent under quantum threat.
The collateral currency choice for quantum resistance is a wash.

---

## 4. The Long-Dated Bounty Problem

The harvest-now-decrypt-later (HNDL) attack is the primary practical concern:

An adversary records all on-chain transactions and `scalar_encrypted` BLOBs
today, then decrypts them once a CRQC becomes available.

Timeline risk by bounty duration:

| Bounty horizon | HNDL risk |
| :--- | :--- |
| 1-5 years | Negligible. No CRQC expected before 2030s at earliest. |
| 10-20 years | Low to moderate. CRQC timeline uncertain; most expert estimates place it 2035-2045+. |
| 50+ years | High. A 50-year bounty issued today will almost certainly exist in a post-CRQC world. |
| 200 years (lifebonds) | Near-certain. The entire ECDLP-based collateral stack will be broken within this window. |

This is a fundamental design tension: the lifebond use case (the founding
motivation for NyxForge) is the most exposed to quantum attack.

---

## 5. The DLEQ Replacement Problem

The DLEQ adaptor signature mechanism is the hardest component to replace
post-quantum. It is elegant precisely because it is built on ECDLP: the
oracle pre-commits to two scalars, and publishing one completes an XMR
transaction in the correct direction without custodianship.

There is no direct post-quantum analogue of this construction in deployed
production code. The NIST post-quantum standards (ML-KEM, ML-DSA, SLH-DSA,
FN-DSA) do not map cleanly onto the two-party adaptor pattern:

- ML-KEM (Kyber): key encapsulation, not signatures -- could replace
  `scalar_encrypted` for the holder ownership model
- ML-DSA (Dilithium): lattice-based signatures -- could replace Schnorr for
  `bounty prove` listing records and holder keypairs
- SLH-DSA (SPHINCS+): hash-based signatures -- large signatures (~8-50KB),
  stateless, conservative security; could replace holder keypairs
- FN-DSA (FALCON): compact lattice-based signatures -- could replace Schnorr

None of these replace the DLEQ collateral locking mechanism. A post-quantum
replacement for DLEQ would require either:

a) A post-quantum two-party computation protocol for the adaptor pattern
   (active research area; no production implementations as of 2026).
b) A fundamentally different collateral locking design that does not rely
   on adaptor signatures at all (e.g., threshold multi-sig with a PQ
   signature scheme, accepting the need for an online quorum at settlement).

---

## 6. Recommended Mitigations by Phase

### MVP and v2 (near-term bounties, <10 year horizons)
- No action required. Quantum threat is negligible on this horizon.
- Document the limitation clearly so issuers of long-dated bounties understand
  the risk.

### v2 (holder keypair hardening, low effort)
- Replace Ed25519 holder keypairs with ML-DSA (Dilithium) or ML-KEM + ML-DSA.
- Replace Schnorr `bounty prove` signatures with ML-DSA.
- This hardens the ownership and trading layer without touching DLEQ.

### v3 (post-quantum verifier slots, already planned)
- Implement the post-quantum verifier slot in the `.bounty` schema (already
  noted as deferred in `doc/00_MVP.md Section 3`).
- Track NIST PQC standardisation and Monero/ZEC PQ roadmaps.

### Long-term / lifebonds
- The honest answer: the collateral locking mechanism requires a post-quantum
  privacy coin or a novel PQ adaptor construction that does not exist today.
- Monitor academic literature on post-quantum adaptor signatures.
- Consider a hybrid escrow model for very long-dated bounties as a stopgap:
  DLEQ for the primary path + a PQ-signed multi-sig fallback redeemable
  after a specified date, accepted by the issuer as a trust tradeoff.

---

## 8. Design Decisions Made (2026-04-19)

### 8.1 10-year maximum deadline

Bounties are limited to a maximum deadline of `inception + 10 years`, enforced
by the CLI wizard. Rationale: bounties with longer horizons enter the CRQC
threat window where harvest-now-decrypt-later attacks become plausible.
The 10-year limit keeps all MVP bounties comfortably within the pre-CRQC
window even under accelerated quantum development scenarios.

Bounties with deadlines beyond 10 years are explicitly deferred in
`doc/00_MVP.md Section 3` as "blocked on CRQC timeline". They will become
possible once the collateral locking mechanism migrates to a PQ scheme.

### 8.2 Forward-compatibility changes to the .bounty schema

The following changes were made to `doc/00_MVP.md Section 4.1` to make
future PQ migration easier without breaking existing bounties:

1. `bounty_spec.alg_epoch` (INTEGER DEFAULT 0) -- algorithm generation
   identifier included in `bounty_id` derivation. Epoch 0 = classical
   (Ed25519). Epoch 1 = PQ transition (hybrid). Epoch 2 = full PQ.

2. `bounty_id` derivation updated to:
   `Blake3(alg_epoch || goal_hash || collateral_hash || oracle_hash || inception)`

3. `collateral.key_algorithm` (TEXT DEFAULT 'ed25519') -- records the
   algorithm used for `holder_pubkey` and `scalar_encrypted`.

4. `collateral.enc_algorithm` (TEXT DEFAULT 'x25519-chacha20poly1305') --
   records the algorithm used to encrypt `scalar_encrypted`.

5. `collateral.pq_holder_pubkey` (TEXT, nullable) -- reserved for a future
   ML-DSA or SLH-DSA public key. Null for all MVP bounties.

6. `collateral.pq_scalar_enc` (BLOB, nullable) -- reserved for scalar
   re-encrypted under ML-KEM. Null for all MVP bounties.

7. `attestations.sig_algorithm` (TEXT DEFAULT 'ed25519') -- records which
   algorithm each oracle used to sign its attestation.

8. `attestations.pq_signature` (BLOB, nullable) -- reserved for hybrid
   attestation once oracles support PQ signing.

9. `verifier` table made one-to-many on `(circuit_id, sig_algorithm)` --
   allows classical and PQ verifier WASM blobs to coexist in the same file.

These changes add no runtime cost for MVP bounties. All PQ fields are null.
They eliminate schema version bumps for the ownership and attestation
layers when PQ algorithms deploy.

### 8.3 What these changes do NOT solve

The DLEQ collateral locking mechanism is not made forward-compatible by
schema changes. When XMR migrates to a PQ scheme, bounties must be re-issued
with a new DLEQ setup. Issuers of bounties with deadlines approaching 2035+
should be aware that re-issuance cooperation between issuer and holder may
be required if XMR migrates before the deadline.

---

## 9. Monero and Zcash PQ Migration Paths
> Note: knowledge cutoff August 2025. Both projects have active research
> programs; check MRL and ECC publications for current state.

### 8.1 Monero

No published PQ migration roadmap with a concrete timeline as of mid-2025.
Active work addresses adjacent problems but not PQ directly.

**Seraphis + Jamtis** -- Major in-progress replacement for the transaction
and addressing layer. Uses Ristretto255 (still ECDLP-based). Not PQ, but
explicitly structured to make future cryptographic migrations easier by
separating address format from key scheme. A precondition, not a solution.

**FCMP++ (Full Chain Membership Proofs)** -- Replaces ring signatures with
UTXO set membership proofs using curve trees (Selene and Helios curves).
Still ECDLP-based. If the curve tree construction were replaced with a
hash-based membership proof, FCMP++ could become scaffolding for a
PQ-compatible anonymity set. That replacement does not exist in production
form.

**The hard problem**: RingCT uses Pedersen commitments to hide amounts.
Pedersen commitments are ECDLP-based. PQ alternatives (lattice-based or
hash-based commitments) exist in academic literature but produce
transactions 10-100x larger. Monero's community has historically been
sensitive to transaction size as a fingerprinting and fee concern, making
this a significant social and technical obstacle on top of the engineering
challenge.

Realistic path: Seraphis ships and stabilizes; MRL then investigates a
lattice-based or hash-based replacement for Pedersen commitments. No
committed timeline. Likely 15+ years from production deployment.

### 8.2 Zcash

More structured upgrade process (Electric Coin Company + Zcash Foundation
network upgrades) gives Zcash a more organised migration path -- but still
no committed PQ timeline as of mid-2025.

**The key insight**: the ZK proof layer is both the hardest and most
important component to replace. Orchard uses Halo 2 (polynomial
commitments, discrete log). The most credible PQ replacement is a
hash-based proof system.

**STARKs (hash-based ZK proofs)** -- STARKs rely only on hash function
collision resistance, not on ECDLP or pairings. A Zcash shielded pool
built on STARKs would have a quantum-resistant proof layer. Obstacles:
STARK proofs are currently much larger than Halo 2 proofs (tens to
hundreds of KB vs a few KB); the circuit design for Zcash shielding
semantics would need to be rewritten for a STARK-compatible arithmetization;
the Pallas/Vesta keypairs would need to be replaced separately.

**Keypair migration** -- Replacing Pallas curve keys with ML-DSA or
SLH-DSA is more tractable than replacing the proof system and could arrive
in a network upgrade independently. ECC has the engineering capacity to
do this if prioritized.

Realistic path: keypair migration (ML-DSA) plausible within 5-10 years if
prioritized. Full proof system migration to STARKs or hash-based commitments
is a more fundamental redesign -- likely 10-15+ years.

### 8.3 Comparison for NyxForge collateral currency selection

| | Monero | Zcash |
| :--- | :--- | :--- |
| Structured upgrade process | No (CCS/MRL consensus) | Yes (network upgrades) |
| PQ roadmap published | No | No |
| Keypair migration feasibility | Moderate | Moderate-high |
| Commitment scheme migration | Very hard (RingCT/Pedersen) | Hard (Halo 2 -> STARKs) |
| Likely timeline to full PQ | 15+ years | 10-15 years |
| Note encryption PQ-safe today | No (RingCT amounts) | Yes (ChaCha20 in Orchard) |

Neither coin has a credible near-term PQ path. Zcash is somewhat better
positioned due to upgrade governance and STARKs being a more mature
research target than lattice-based RingCT. Both are on similar practical
timelines. For NyxForge MVP the collateral currency choice is a wash on
quantum resistance grounds; XMR remains preferred for DLEQ compatibility
and privacy maturity.

---

## 10. Key References

- NIST PQC standards (2024): ML-KEM (FIPS 203), ML-DSA (FIPS 204),
  SLH-DSA (FIPS 205), FN-DSA (FIPS 206)
- Shor, P. (1994). Algorithms for quantum computation: discrete logarithms
  and factoring. FOCS 1994.
- Monero post-quantum research: getmonero.org research lab (ongoing)
- Monero Seraphis: github.com/UkoeHB/seraphis_lib (in progress)
- Monero FCMP++: github.com/kayabaNerve/fcmp-plus-plus (in progress)
- Zcash Orchard / Halo 2: github.com/zcash/orchard
- STARKs: Ben-Sasson et al. (2018), "Scalable, transparent, and
  post-quantum secure computational integrity"
- "Post-Quantum Adaptor Signatures" -- active research area; no standard
  as of 2026
