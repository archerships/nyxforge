# TLS-Notary / DECO -- Research Brief

> April 2026
> Status: Research (background, not authoritative)
> See also: `doc/03_RESEARCH/oracle-network-comparison.md`
>           `doc/05_TECH/oracle-spec.md`

---

## 1. Who Is Building It

### TLS-Notary (PSE / Ethereum Foundation)

TLS-Notary is developed by Privacy & Scaling Explorations (PSE), a research
and development group funded by the Ethereum Foundation. PSE builds open-source
cryptographic infrastructure -- their portfolio includes Semaphore (ZK identity),
Aztec, and zkEVM components. TLS-Notary is their primary project for provable
Web2 data bridging.

- Repository: https://github.com/tlsnotary/tlsn
- Language: Rust (core library), WASM bindings for browser use
- License: Apache 2.0 / MIT dual
- Status (as of early 2026): v0.1.x on mainnet; browser extension in beta;
  active mainline development

Core contributors are PSE engineers operating under Ethereum Foundation
funding. There is no token and no company -- TLS-Notary is a public-good
research project. It has received additional grants from Protocol Labs and
several ZK-focused funds.

### DECO (Cornell Tech / Chainlink)

DECO is a related but distinct protocol developed by Ari Juels, Fan Zhang,
and colleagues at Cornell Tech's IC3 (Initiative for CryptoCurrencies and
Contracts). Chainlink Labs acquired the DECO research in 2020 and is
developing it as a commercial product.

Unlike TLS-Notary, DECO uses a two-party protocol (Prover + Verifier only,
no third-party Notary) based on garbled circuits. This eliminates the
requirement for a Notary to be online during the session but is computationally
heavier. DECO is not yet publicly deployed as a standalone product (as of
2026); it appears on Chainlink's roadmap as the foundation for
"Chainlink DECO-based attestations."

For NyxForge purposes, TLS-Notary (PSE) is the actionable implementation.
DECO is noted here as a future alternative if Chainlink ships it publicly.

---

## 2. How TLS-Notary Works

### 2.1 The Problem It Solves

When a user fetches data over HTTPS, the TLS session is encrypted end-to-end
between their client and the server. No third party can verify what data was
returned without being given the session keys -- but possessing the session
keys also allows forging any response. A judge who says "I checked the FDA
website and the drug was approved" cannot prove this to anyone.

TLS-Notary solves this with a three-party MPC protocol that lets a Notary
co-witness a TLS session, producing a proof that specific data was returned
by a specific server at a specific time -- without the Notary ever seeing the
content, and without the session keys being fully held by any single party.

### 2.2 The Three Parties

- Prover: the party fetching data (the bond judge, an oracle node, or the
  NGO staff member running the check)
- Notary: a third party that co-participates in the TLS handshake using
  two-party computation (2PC); signs a commitment to its key share; never
  sees the plaintext
- Verifier: the downstream consumer of the proof (another judge, a NyxForge
  resolution node, or an archival auditor in 2090)

The Notary and Prover split the TLS session keys using 2PC. Each holds a
share; neither can independently decrypt the full session or forge a response.
The Notary signs a commitment to its key share; the Prover uses this to
produce a proof that a specific substring appeared in the authenticated
server response.

### 2.3 What the Proof Guarantees

A TLS-Notary proof is a self-contained JSON document asserting:

1. A TLS session was established with a specific server (verified by the
   server's certificate chain)
2. A specific HTTP response was received during that session
3. A specific substring or field exists within that response (the Prover
   selectively discloses only the relevant portion)
4. The Notary co-signed the session commitment at a specific timestamp

What the proof does NOT reveal: the full page content, the Prover's session
token, cookies, credentials, or any redacted fields. The Prover has full
control over which parts of the response to disclose.

### 2.4 Selective Disclosure

The Prover can redact arbitrary portions of the response before sharing the
proof. If the FDA API returns a 4 KB JSON object, the Prover can prove that
the field `submission_status = "AP"` exists without revealing the rest.
This is implemented via a commitment scheme over the TLS record layer.

Selective disclosure is critical for NyxForge bonds involving personal or
commercially sensitive data sources. A judge can prove a data point was true
without leaking unrelated private information.

### 2.5 Verification

The Verifier receives the proof JSON and verifies:
- The Notary signature over the session transcript commitment
- The TLS certificate chain for the target server
- The redacted plaintext matches the commitment

Verification is fast (milliseconds) and fully offline -- the proof is
self-contained and requires no network access. This means a proof generated
in 2045 can be re-verified in 2145 by anyone who has the proof file and the
TLS-Notary verification library.

---

## 3. Current Use Cases

### 3.1 DeFi Undercollateralized Lending

Protocols are experimenting with TLS-Notary proofs to demonstrate
creditworthiness without revealing bank account details. A borrower proves
"my Plaid-verified bank balance exceeds $50,000" without giving the lender
access to their account number or full statement. The borrower fetches the
Plaid API response, generates a TLS-Notary proof disclosing only the balance
field, and submits it to the lending protocol.

### 3.2 Proof of Social Identity

Applications use TLS-Notary to prove "I have a Twitter/X account with >10,000
followers" or "I am a member of this Discord server" without using OAuth
(which would reveal the user's identity to the application). Used in anonymous
DAO voting experiments where one-person-one-vote is required but full identity
disclosure is undesirable.

### 3.3 ZK Email and Web2 Credential Bridging

PSE and related projects (ZK-Email) use TLS-Notary-style proofs to bridge
Web2 credentials -- GitHub contribution history, LinkedIn employment records,
university email verification -- into ZK circuits without a trusted middleman.
A developer can prove they have merged code to a major open-source repository
without revealing their GitHub username.

### 3.4 Parametric Insurance Triggers

Experimental insurance products use TLS-Notary proofs to verify flight delay
data from airline APIs or rainfall measurements from NOAA, triggering
automatic payouts without requiring the insurer to trust a single data feed
operator. The insured fetches the data, proves it with TLS-Notary, and the
insurance contract pays out if the threshold is met.

### 3.5 Regulatory Compliance Attestations

Startups in the compliance space are piloting TLS-Notary for proving that a
company's financial statements (fetched from accounting APIs) contain specific
figures, without sharing the full document with a counterparty. A startup can
prove its ARR exceeds $1M for an investor without revealing its full P&L.

---

## 4. NyxForge Integration

### 4.1 Role in the Architecture

In the NyxForge oracle stack, TLS-Notary is the Tier 1 data-fetching layer
for bonds whose judgment condition can be verified against a public or semi-
public HTTPS data source. The judge (or an automated oracle node) runs a TLS-
Notary session, produces a proof, and submits it as the bond's attestation
record.

The proof file is archived permanently on Arweave as part of the bond's
evidence package, making it auditable by anyone with the proof file and the
verifier library -- including auditors operating decades in the future.

### 4.2 Example 1 -- Alzheimer's Cure Bounty (FDA Approval API)

Bond condition: "FDA approves a treatment reducing new Alzheimer's diagnoses
by 50% before 2045-12-31."

Data source: FDA Drugs@FDA API
(https://api.fda.gov/drug/drugsfda.json)

Judgment flow:

1. After 2045-12-31, Dr. Marcus Webb opens a TLS-Notary session against
   api.fda.gov using the NyxForge oracle client.

2. He fetches:
   GET /drug/drugsfda.json?search=openfda.brand_name:"MemoraCure"&limit=1

3. The FDA API returns a JSON object including:
   { "submission_status": "AP", "submission_status_date": "20441103" }

4. A NyxForge Notary node (any registered volunteer) co-signs the session
   commitment using 2PC. The Notary sees nothing except the server certificate
   and its own key share.

5. Dr. Webb generates a TLS-Notary proof disclosing only:
   { "submission_status": "AP", "submission_status_date": "20441103" }
   All other fields (manufacturer details, clinical trial numbers, etc.) are
   redacted.

6. The proof JSON is signed with Dr. Webb's NyxForge judge key and committed
   to Arweave (permanent archive).

7. Dr. Chen and Dr. Marchetti each independently run the same session against
   the same endpoint, producing their own proofs.

8. The NyxForge resolution engine receives 2-of-3 matching proofs, verifies
   all three against the Notary signatures and FDA certificate chain, and
   unlocks the PTLC collateral to the ARA redemption address.

What is redacted in each proof: drug manufacturer identity, NDA internal case
numbers, clinical trial IDs, any personally identifiable data of trial
participants, and all other response fields not relevant to the bond condition.

### 4.3 Example 2 -- Longevity Index Bond (WHO Statistics API)

Bond condition: "Global average healthy life expectancy (HALE) exceeds 80
years according to WHO GHO data before 2040-12-31."

Data source: WHO Global Health Observatory OData API
(https://ghoapi.azureedge.net/api/WHOSIS_000015)

Judgment flow:

1. An automated NyxForge oracle node runs a TLS-Notary session against the
   WHO API once per year beginning 2039-01-01 (scheduled via the bond's
   oracle_schedule field).

2. It fetches:
   GET /api/WHOSIS_000015?$filter=SpatialDim eq 'GLOBAL'&$orderby=TimeDim desc

3. The response includes: { "NumericValue": 81.2, "TimeDim": 2039 }

4. The proof selectively discloses only NumericValue and TimeDim.

5. Because the bond specifies an automated judge (type: algorithmic_feed),
   no human review step is required. Three independent oracle nodes run the
   same session against different Notaries and submit matching proofs.

6. 2-of-3 matching proofs trigger resolution. The .bond file is updated to
   REDEEMABLE state and the collateral PTLC is unlocked.

Note on automation: automated oracle sessions are scheduled by the bond's
oracle_schedule field (a cron expression stored in the .bond SQLite file).
The NyxForge oracle daemon reads this field and dispatches TLS-Notary sessions
on the specified schedule. The session output is deterministic -- any node
fetching the same URL at the same time will get the same data.

### 4.4 Example 3 -- Qualitative Fallback with TLS-Notary as Evidence

Bond condition: "The Cochrane Collaboration publishes a systematic review
concluding that treatment X reduces Alzheimer's incidence by >=50% with
high certainty (GRADE A)."

Data source: Cochrane Library structured metadata
(https://www.cochranelibrary.com/cdsr/doi/...)

Judgment flow:

1. Dr. Elena Marchetti identifies the specific Cochrane review URL after
   publication.

2. She fetches the review's machine-readable metadata page, which includes:
   { "grade_certainty": "HIGH", "effect_size": "-52%", "doi": "10.1002/xxx" }

3. A TLS-Notary proof is generated disclosing only grade_certainty,
   effect_size, and doi. The proof establishes that this data appeared on
   the Cochrane website at a specific time -- it cannot be fabricated after
   the fact.

4. Marchetti submits the proof as her attestation, signed with her judge key.

5. The qualitative determination ("does -52% with GRADE A certainty satisfy
   the bond condition?") was made by Marchetti as the designated expert. The
   TLS-Notary proof prevents her from attesting to a review that does not
   exist or was retracted.

Limitation of this example: TLS-Notary proves the page said X at time T. It
does not prevent a judge from choosing to fetch a different page, or from
claiming a different review is the relevant one. The human accountability
layer -- judge identity, signed commitment, reputation stake, and the dispute
mechanism -- remains necessary for qualitative cases. TLS-Notary provides
evidence integrity, not judgment integrity.

### 4.5 Notary Node Operation

Any NyxForge participant can run a Notary node. The Notary:
- Runs the tlsn-notary binary (Rust, commodity hardware, ~256 MB RAM)
- Participates in the 2PC TLS handshake on demand via the NyxForge oracle
  client's notary discovery protocol
- Signs its session transcript commitment with an ephemeral key registered
  on the NyxForge oracle node registry (Arweave)
- Earns a small NYX reward per proof co-signed

Notary nodes are blind witnesses: they learn nothing about the Prover's
session content. A malicious Notary can refuse to sign (denial of service)
but cannot forge a proof. If a Notary goes offline, the Prover retries with
any other registered Notary. This is a permissionless, rotating set with no
central operator.

### 4.6 Long-Duration Archival

Proofs generated in 2045 must be verifiable in 2145. The TLS-Notary proof
JSON is self-contained -- it includes the full TLS certificate chain and
Notary signature. The only external dependency is the verification library.

NyxForge's archival strategy:
- All proof JSONs are pinned to Arweave at submission (permanent, content-
  addressed)
- The TLS-Notary verifier (compiled WASM) is also pinned to Arweave alongside
  each proof, so a future auditor can retrieve the exact version used
- Certificate chain expiry is handled by recording the server cert at proof
  time; the proof is valid regardless of whether the cert later expires

---

## 5. Limitations

| Limitation | Detail |
| :--------- | :------ |
| API availability | If the target API changes its URL, schema, or authentication model, oracle scripts must be updated. Long-dated bonds (2040+) require maintenance planning. |
| No dispute layer | TLS-Notary proves data integrity but has no escalation mechanism. Disputes between judges go to Tier 2 (Reality.eth) or Tier 3 (Kleros / UMA). |
| Notary availability | The Prover needs a live Notary to co-sign the session. If all registered Notaries are offline, proof generation fails. Mitigated by operating multiple geographically distributed Notary nodes. |
| TLS version dependency | Designed for TLS 1.3. Sites still running TLS 1.2 require a compatibility adapter. Most major government APIs (FDA, WHO, NIH) are already TLS 1.3. |
| Proof size | Proofs are typically 10-100 KB depending on response size and number of redacted fields. Fine for Arweave; too large for direct on-chain storage. |
| Computational cost | The 2PC MPC step takes 1-10 seconds on commodity hardware. Not a bottleneck for once-per-deadline bond resolution, but relevant for high-frequency oracle use cases. |
| Human judgment gap | TLS-Notary proves what a data source says; it cannot prove the data source is authoritative, correct, or the right one. Expert judgment remains necessary for qualitative bonds. |

---

## 6. Reference Links

- TLS-Notary project: https://tlsnotary.org
- GitHub (tlsn): https://github.com/tlsnotary/tlsn
- PSE (Privacy & Scaling Explorations): https://pse.dev
- DECO paper (Cornell IC3): https://eprint.iacr.org/2019/1440
- TLS-Notary browser extension: https://github.com/tlsnotary/tlsn-extension
- ZK-Email (related project): https://prove.email
- FDA Drugs@FDA API: https://open.fda.gov/apis/drug/drugsfda
- WHO GHO OData API: https://www.who.int/data/gho/info/gho-odata-api
