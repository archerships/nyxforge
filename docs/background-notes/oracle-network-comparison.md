# Oracle Network Comparison -- NyxForge Constraint Matrix

> April 2026
> Status: Research (background, not authoritative)
> See also: `doc/03_RESEARCH/anonymous-oracles.md`
>           `doc/03_RESEARCH/chainlink-oracle-brief.md`
>           `doc/05_TECH/oracle-spec.md`

---

## 1. Constraint Definitions

The following constraints are derived from the NyxForge architecture mandate
(`doc/05_TECH/oracle-spec.md` and `doc/00_CORE.md`). Each is binary or graded:

| ID  | Constraint         | What it means |
| :-- | :----------------- | :------------ |
| C1  | Anon operators     | Oracle nodes can participate without a public identity or KYC. Sybil resistance comes from stake, not reputation. |
| C2  | Commodity hardware | Runs on a standard CPU/server. No Intel SGX, TDX, or other trusted execution environments required. |
| C3  | Non-EVM compatible | Can deliver results to a non-EVM target (XMR PTLC/DLEQ, AO process, or off-chain .bounty file). Not locked to Ethereum. |
| C4  | Century-scale      | Has a plausible survival path for 20-200 year bounties. Incentive model does not assume any single operator or token persists. |
| C5  | Web2 data fetching | Can retrieve and attest to data from standard HTTPS APIs (FDA, WHO, government databases). |
| C6  | Qualitative support| Can handle outcomes that cannot be reduced to a single API call -- expert review, panel vote, documentary evidence. |
| C7  | Dispute mechanism  | Includes or integrates with a challenge/escalation layer. False attestations can be contested. |
| C8  | Production maturity| Y = mainnet, battle-tested; P = testnet or limited mainnet; N = alpha/concept. |
| C9  | Cost model         | Sustainability of per-resolution cost at scale. L = low (<$1), M = moderate ($1-$10), H = high (>$10 or token-dependent). |

Rating key: Y = fully meets constraint / P = partially meets / N = does not meet / -- = not applicable

---

## 2. Comparison Table

Table.1.OracleMatrix

| Network               | C1 Anon | C2 Comm HW | C3 Non-EVM | C4 Century | C5 Web2 | C6 Qualit. | C7 Dispute | C8 Mature | C9 Cost |
| :-------------------- | :-----: | :--------: | :--------: | :--------: | :-----: | :--------: | :--------: | :-------: | :-----: |
| TLS-Notary / DECO     |    Y    |     Y      |     Y      |     P      |    Y    |     N      |     N      |     P     |    L    |
| Reclaim Protocol      |    Y    |     Y      |     P      |     N      |    Y    |     N      |     N      |     P     |    L    |
| DarkFi Witness        |    Y    |     Y      |     Y      |     P      |    P    |     P      |     Y      |     N     |    L    |
| 0rbit (AO-native)     |    P    |     Y      |     Y      |     Y      |    Y    |     N      |     N      |     P     |    L    |
| Pado Network          |    Y    |     Y      |     P      |     N      |    Y    |     N      |     N      |     N     |    L    |
| UMA Optimistic Oracle |    P    |     Y      |     P      |     P      |    P    |     Y      |     Y      |     Y     |    M    |
| Reality.eth           |    P    |     Y      |     P      |     P      |    N    |     Y      |     Y      |     Y     |    L    |
| Kleros                |    P    |     Y      |     P      |     P      |    N    |     Y      |     Y      |     Y     |    M    |
| Tellor                |    P    |     Y      |     P      |     P      |    P    |     N      |     Y      |     Y     |    M    |
| Chainlink Functions   |    N    |     Y      |     N      |     N      |    Y    |     P      |     P      |     Y     |    M    |

---

## 3. Network Notes

### TLS-Notary / DECO
The strongest single component for C5 (Web2 data). A prover fetches an HTTPS
page and generates an MPC-based proof that specific data existed in the
response, without revealing credentials or session state. The verifier learns
only the claimed fact. Fully anonymous, runs on commodity hardware, chain-
agnostic (the proof is a file, not an on-chain transaction).

Gaps: no built-in dispute layer (C7), no incentive model for century-scale
node operation (C4 = partial: the protocol survives, but who runs provers in
2080 requires a separate incentive design). Cannot handle qualitative outcomes
(C6) -- it can only attest to what a page says, not whether an expert agrees
with it.

NyxForge role: primary Tier 1 oracle for all bounties with API-verifiable
outcomes. The prover can be the judge themselves, an NGO employee, or a
volunteer oracle node.

### Reclaim Protocol
A live network of "witness" nodes that run TLS-Notary-style proofs as a
service. More turnkey than raw TLS-Notary but introduces a semi-trusted witness
layer. Witnesses are pseudonymous (Ethereum addresses) rather than fully
anonymous, and the network is still maturing. Non-EVM delivery is partial --
the proof JSON can be consumed off-chain but there is no native non-EVM
delivery mechanism.

NyxForge role: a viable fallback if running a TLS-Notary prover in-house is
too complex for the MVP. Watch for mainnet decentralization progress in 2026.

### DarkFi Witness Model
The best architectural match for NyxForge's anonymity mandate. Oracle identity
is a stealth address derived from a Monero-style key. Honesty is enforced by
locked stake (XMR/DRK): a provably false attestation triggers automatic slash
via a ZK dispute proof. No public identity is ever required. The model is
conceptually mature but DarkFi infrastructure is still in alpha (as of April
2026, testnet only).

Partial C5: DarkFi witnesses can in principle fetch any URL but there is no
zkTLS integration yet -- Web2 attestation requires the witness to be trusted
rather than cryptographically provable.

NyxForge role: target architecture for anonymous oracle nodes in Phase 3+.
Track DarkFi mainnet timeline.

### 0rbit (AO-native)
A decentralized oracle network built as an AO process. Operators are publicly
identified (Arweave wallet addresses) but requests can be routed through
Shielded AO logic to decouple the bounty identity from the data request. Strong
century-scale fit (C4 = Y) because AO processes are permanent and 0rbit
incentives are tied to AR token which has a 200-year emission schedule.

NyxForge role: natural choice for the AO resolution layer if NyxForge adopts
AO for Long Now bounty archival. Can handle price feeds, Web2 data, and
cross-process calls. Does not replace the anonymity layer -- that must be
added on top via Shielded AO.

### Pado Network
Combines MPC (for HTTPS session privacy) with Interactive Zero-Knowledge for
more complex computations over private data. Still early (alpha network as of
early 2026). Non-EVM delivery is partial -- proofs can be consumed off-chain
but the delivery infrastructure is Ethereum-oriented. No dispute mechanism.

NyxForge role: monitor as an alternative to TLS-Notary for richer Web2
attestations. Not ready for MVP.

### UMA Optimistic Oracle
A pull-based optimistic oracle: anyone can assert an answer by posting a bounty;
a dispute window opens; if unchallenged it settles. If challenged, the
assertion escalates to UMA's token-holder dispute court. No permissioned
operator set -- any address can propose or dispute.

Best network for qualitative outcomes (C6 = Y): the question can be
free-form English ("Did the FDA approve drug X before 2045?") and human
proposers respond. EVM-native but the resolution result can be bridged off-
chain. Century-scale is partial -- UMA token incentives may not persist for
200 years, but the optimistic pattern itself could be re-implemented.

NyxForge role: strong candidate for Tier 2 escalation (the "bounty-doubling
game" already described in oracle-spec.md is essentially this pattern). Also
useful as Tier 1 for qualitative bounties where no API is authoritative.

### Reality.eth
A minimalist optimistic oracle: ask a question in plain English, stake ETH on
the answer, wait for the challenge window, escalate to Kleros or Gnosis DAO if
disputed. Extremely low cost for uncontested verdicts. Fully permissionless --
no operator identity required to answer, but respondents are pseudonymous
Ethereum addresses (C1 = partial).

No native Web2 data fetching (C5 = N) -- it relies on humans to look things up
and post answers.

NyxForge role: well-matched to Tier 2 escalation for the MVP. Simple to
integrate, low cost, and the dispute escalation path to Kleros is already
designed in oracle-spec.md.

### Kleros
A decentralized dispute court. Jurors are drawn randomly from a staked pool,
evidence is submitted as IPFS documents, and the majority verdict is binding
on-chain. Not an oracle per se -- it is a dispute resolution layer. Best used
as the final arbitration step (Tier 3) for contested bounty outcomes.

Operators are pseudonymous (Ethereum addresses) but staking is public (C1 =
partial). No Web2 data fetching. Excellent for qualitative outcomes that
require human judgment on documentary evidence.

NyxForge role: designated Tier 3 arbitration layer in oracle-spec.md. No
change recommended.

### Tellor
A permissionless oracle where anyone can submit data by staking TRB tokens.
Disputed values can be challenged; persistent disputes go to a TRB token-holder
vote. More decentralized than Chainlink but still fully public (C1 = partial).
Web2 data fetching requires a custom adapter. No native qualitative outcome
support.

NyxForge role: low priority. Covers similar ground to UMA but with weaker
qualitative support and a smaller ecosystem. Not recommended over UMA/Reality.

### Chainlink Functions
Included for reference and contrast. The dominant EVM oracle network but
structurally incompatible with NyxForge's anonymity and non-EVM requirements.
Best-in-class for EVM bounties with API-verifiable outcomes and no privacy
requirement. See `doc/03_RESEARCH/chainlink-oracle-brief.md` for full analysis.

---

## 4. Recommended Stack by Tier

This maps the comparison results to the three-tier architecture in
`doc/05_TECH/oracle-spec.md`.

| Tier | Purpose | Recommended network(s) | Rationale |
| :--- | :------ | :--------------------- | :-------- |
| Tier 1 -- data fetch | Retrieve and prove Web2 outcomes | TLS-Notary (primary); Reclaim (fallback) | Best anonymity + Web2 attestation; no EVM dependency |
| Tier 1 -- qualitative | Outcomes requiring expert judgment | UMA Optimistic Oracle | Free-form questions, permissionless proposers, low cost when uncontested |
| Tier 1 -- price feeds | Collateral valuation | 0rbit (if AO stack); RedStone (EVM sidechain) | Already in oracle-spec.md; low cost, commodity hardware |
| Tier 2 -- escalation | Dispute doubling game | Reality.eth or UMA | Matches the bounty-doubling pattern; cheap and battle-tested |
| Tier 3 -- arbitration | Final contested verdicts | Kleros | Already specified; human jury on documentary evidence |
| Future -- anon nodes | Anonymous stake-based oracles | DarkFi Witness | Best long-run fit; blocked on DarkFi mainnet |

---

## 5. Gaps Not Covered by Any Network

The following requirements have no adequate existing solution and will require
custom NyxForge development:

1. Century-scale operator incentives -- no existing network has a credible
   200-year sustainability model. NyxForge must implement the Long Now
   endowment pattern (XMR yield -> archivist bounties) described in
   oracle-spec.md Section 3.

2. Anonymous Web2 attestation with dispute -- TLS-Notary proves data but has
   no dispute layer. DarkFi has a dispute layer but no zkTLS. Combining them
   is a research problem, not a solved product.

3. Non-EVM delivery of on-chain verdicts to PTLC/DLEQ contracts -- every
   mature dispute layer (UMA, Kleros, Reality.eth) is EVM-native. Bridging
   their verdicts to XMR PTLC scripts requires a custom adapter or a trusted
   relayer, which reintroduces centralization risk.
