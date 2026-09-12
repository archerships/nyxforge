## What Are Social Policy Bonds?

Governments, foundations, and private actors spend enormous sums trying to
achieve social and environmental goals -- reducing homelessness, improving
public health, passing legislation, accelerating scientific research. Most of
that spending funds inputs: salaries, lobbying contracts, clinical trials,
marketing campaigns. There is rarely a mechanism to verify whether any given
dollar actually moved the outcome. And there is no market signal to indicate
which approach is most likely to succeed.

Social policy bonds (SPBs) are financial instruments that invert this logic.
Instead of funding activities, they fund results.

A social policy bond works as follows:

1. An issuer defines a specific, measurable goal with a fixed deadline (e.g.,
   "the price of XMR exceeds $1,000 for 30 consecutive days by 2028").

2. The issuer locks collateral (cryptocurrency) into an escrow that can only
   be released by one of two outcomes: goal achieved (collateral goes to the
   bondholder) or deadline expired (collateral returns to the issuer).

3. The issuer sells the bonds at whatever price the market will bear. That
   price is the market's probability estimate that the goal will be achieved.
   A bond trading at 0.5 XMR against a 1 XMR redemption value implies the
   market assigns 50% probability to success.

4. Bond buyers are free to use any means to achieve the goal. They profit only
   if the goal is actually reached, verified by an independent oracle panel.
   There is no reporting requirement, no strategy approval, no governance
   overhead.

The result is a market mechanism that:

- Identifies the most capable actors (those who can move the outcome) and
  concentrates resources toward them via open competition.
- Produces a real-time probability signal on whether any given goal is
  achievable and at what cost.
- Separates the decision about what goal to fund (the issuer's choice) from
  the decision about how to achieve it (the bond buyer's expertise).
- Requires no trust between issuer and buyer. The collateral is locked and
  the oracle verification is defined at issuance. Neither party can cheat.

NyxForge is an open, anonymous, permissionless infrastructure for issuing,
trading, and settling social policy bonds -- with no centralized custodian,
no KYC, and no dependence on any existing financial institution.

---

## Case #1: Cryptocurrency Lobbying

### The problem

If you hold a large position in a cryptocurrency, it is in your interest to
increase the coin's adoption and value. 

But how do you allocate spending most
effectively? Adding new features? Marketing to vendors? Lobbying regulators?

There is no clear mechanism to identify which activity produces the most value
per dollar. 

And from a developer's perspective, building on an existing chain
often makes less financial sense than launching a new chain with its own token
economics.

Social policy bonds solve both problems simultaneously.

### Example: Monero (XMR) development

Suppose you are an XMR holder with 10,000 XMR, each worth $500. Total value:
$5,000,000.

You issue 2,500 SPBs, each of which pays the bearer 1 XMR if the price of XMR
rises to $1,000 and remains there for 30 days.

You lock 2,500 XMR as collateral (one per bond). You sell the bonds at 0.01
XMR ($5) each for total proceeds of $12,500.

A DC lobbying firm, CryptoLobby Inc., believes it can pass legislation
eliminating the capital gains tax on cryptocurrency. It buys all 2,500 bonds
for $12,500.

CryptoLobby spends $500,000 on lobbying. The legislation passes. XMR rises
from $500 to $1,000.

Table.1.Monero Lobbying

| Party | Outcome |
| :--- | :--- |
| CryptoLobby receipts | 2,500 XMR @ $1,000 = $2,500,000 |
| CryptoLobby costs | $500,000 lobbying + $12,500 bonds = $512,500 |
| CryptoLobby net profit | $1,987,500 |
| Issuer: remaining holdings | 7,500 XMR @ $1,000 = $7,500,000 |
| Issuer: bond proceeds retained | $12,500 |
| Issuer total | $7,512,500 |
| Issuer gain vs. doing nothing | +$2,512,500 (original position: $5,000,000) |

### Price signal

If bonds trade at 0.5 XMR ($250 at original price) in the secondary market,
that price implies the market assigns a 50% probability that the lobbying will
succeed. This gives the issuer real-time feedback on whether the effort is
working -- something no traditional donation or grant mechanism provides.

### Why this investor funds NyxForge

This bond cannot exist without NyxForge. Any XMR whale who would issue such a
bond has a direct financial incentive to fund NyxForge development. The return
on a $10,000 grant dwarfs the expected value of holding XMR passively and
hoping someone else improves the ecosystem.

### What the bond issuer does NOT need to do

- Trust the lobbying firm's judgment about strategy
- Evaluate the merits of specific regulatory approaches
- Run a grant program with reporting requirements
- Know anything about lobbying at all

The market takes care of all of that. Bond buyers self-select based on their
confidence in their own ability to move the outcome.

---

## Case #2: Nuclear Energy Regulatory Relief

### The problem

Nuclear power is capital-intensive and heavily regulated. A single project can
wait a decade for regulatory approval. Investors in nuclear companies face
regulatory risk that is largely outside their control and impossible to hedge
with conventional instruments.

### Proposed bond structure

A nuclear energy investor (or a consortium) issues bonds that pay the bearer
if a specific, verifiable regulatory outcome is achieved by a fixed deadline.

Example goal specification:

    data_id:   valar.atomics.win
    terms:     Valar Atomics wins their case against the NRC 
    deadline:  2030-01-01


Interpretation: the bond pays 1 XMR per bond if Valar Atomics wins their case against the NRC by January 2030.

### Who buys these bonds

- Nuclear advocacy organizations (e.g., Nuclear Energy Institute)
- Consulting firms that specialize in regulatory strategy
- Law firms that litigate against the NRC
- Energy VCs with nuclear portfolio exposure

Each buyer is a potential actor who can influence the outcome. The bond price
reflects the market's probability estimate of success. A high price signals
that the market believes the goal is achievable; a low price signals doubt.

### Why this investor funds NyxForge

The investor's existing nuclear portfolio may be worth hundreds of millions of
dollars. 

If the right regulatory outcomes were achieved, the value of funding the develop of Nyxforge is large relative to the grant cost.

---

## Case #3:  Cancer research

### The problem

Traditional medical philanthropy funds inputs: researchers, trials, equipment.
There is no mechanism to ensure that spending produces results. A VC-style
model introduces equity, which conflicts with a non-profit mission. Neither
model gives the donor a direct stake in the outcome.

### Worked example

Suppose the parent of a child with a rare cancer wants to incentivize
development of a cure. They raise $100,000 from their family network and use
it to issue 100 bonds, each backed by 1 XMR (at $500 per XMR, 0.2 XMR per
bond = $100,000 total collateral at issuance).

The bonds pay the bearer 1 XMR if the child's cancer is in complete remission
as determined by a panel of oncologists (qualitative oracle panel) by 2030.

A biotech VC firm that specializes in rare pediatric cancers buys all 100 bonds
for 0.05 XMR each ($25 per bond = $2,500 total). The VC firm then:

1. Funds one of its portfolio companies to develop a targeted therapy.
2. Monitors progress; if the therapy works, redeems the bonds for 100 XMR.

Table.2.CancerCase

| Scenario | VC firm outcome |
| :--- | :--- |
| Cure found, bonds redeemed | 100 XMR received - 5 XMR paid - therapy cost |
| No cure, bonds expire | Loses $2,500 bond purchase; recoups from other portfolio returns |

The family's 100 XMR collateral is either redeemed by the VC (goal met) or
returned to the family (goal not met). The family's downside is losing the
opportunity cost of locking that XMR; their upside is a cure for their child.

### Key structural feature

The prize is held in escrow, released only on verified performance, and the performer is
free to use any method they choose.

The VC firm profits only if the child is cured, so all of its strategy (which company to
fund, how much to spend, which therapy pathway to pursue) is directed toward
that outcome.



### Why this investor funds NyxForge

Any family or foundation with a specific medical mission would benefit from this
model. The key obstacle is that NyxForge does not yet exist. A grant to build
NyxForge is instrumentally identical to a grant to fund the research itself --
except that NyxForge, once built, enables an unlimited number of such bonds for
any goal.

---

## Legal Risks

Developing NyxForge -- permissionless
financial infrastructure with privacy properties -- could expose developers and
funders to prosecution under the same legal theories used against the developers
of Tornado Cash and Samourai Wallet. This argues for anonymous development and
anonymous funding.

### The Tornado Cash Prosecution

Tornado Cash was a set of immutable Ethereum smart contracts, launched in 2019,
that mixed cryptocurrency deposits and withdrew equivalent amounts to different
addresses, breaking the on-chain transaction trail. The protocol was non-custodial:
the developers never held user funds. After publishing the code, they had no
technical ability to modify or shut the contracts down.

The developers were Roman Storm, Roman Semenov (both American), and Alexey
Pertsev (Dutch-Russian). They had received funding from Paradigm, a major
crypto venture capital firm. The protocol was governed by a DAO, and the
developers held TORN governance tokens.

In August 2022, the U.S. Treasury's Office of Foreign Assets Control (OFAC)
sanctioned not just the developers but the smart contract addresses themselves --
the first time in history that OFAC sanctioned open-source code. The sanction
made it illegal for U.S. persons to interact with the contracts, even though no
human being could prevent the contracts from running.

Alexey Pertsev was arrested in the Netherlands in August 2022, within days of
the OFAC announcement. He was held in pretrial detention for nine months.
In May 2024, a Dutch court convicted him of money laundering and sentenced him
to five years and four months in prison. The court held Pertsev liable for
money laundering committed by third parties who used Tornado Cash, even though
the protocol was automated and non-custodial. The immutability of the contracts
-- the fact that Pertsev literally could not have shut them down after
deployment -- was not a defense.

Roman Storm was arrested in the United States in August 2023. He was charged
with conspiracy to commit money laundering, conspiracy to violate OFAC
sanctions, and conspiracy to operate an unlicensed money transmitting business.
Roman Semenov was also charged but remains at large.

The Tornado Cash prosecutions established several precedents:

- Writing and publishing open-source smart contracts can constitute "operating"
  a money transmitting business under 18 U.S.C. 1960.
- Developers can be held liable for money laundering by third parties who use
  their code, even if those developers received no portion of the laundered funds.
- The immutability of deployed code (developers cannot shut it down) does not
  establish a defense.
- Holding governance tokens in a protocol's DAO is treated as evidence of
  ongoing operation and profit-sharing.
- VC backing and a publicly identified founding team create identifiable
  defendants. The protocol continues running; only the people who could be
  found were arrested.

### The Samourai Wallet Prosecution

Samourai Wallet was a Bitcoin privacy wallet that operated for years under
pseudonyms. Its two founders -- Keonne Rodriguez and William Lonergan Hill --
used the handles "TDevD" and "SW" for much of the project's life before their
real identities became known. The wallet featured Whirlpool, a CoinJoin
implementation that mixed Bitcoin transactions among multiple participants, and
Stowaway, a collaborative transaction construction tool. Samourai was
non-custodial: the company never held or transmitted user funds.

In April 2024, the DOJ arrested Rodriguez and Hill in coordinated operations.
They were charged with conspiracy to operate an unlicensed money transmitting
business and conspiracy to commit money laundering. The indictment alleged
that Samourai had processed over $2 billion in unlawful transactions and had
facilitated more than $100 million in money laundering from darknet markets,
fraud schemes, and other criminal enterprises.

The prosecution's theory was that by operating coordination servers for the
CoinJoin mixing -- even though no funds ever passed through those servers --
Samourai was functioning as an unlicensed money transmitter. The developers
argued they were providing a software tool. The government argued that
providing the coordination infrastructure made them the operators of a
financial service, regardless of whether they touched the money.

The Samourai case added its own set of precedents:

- Non-custodial software wallets can be prosecuted as money transmitting
  businesses if the developers operate any server infrastructure involved in
  the transaction flow.
- Pseudonymous operation provides some protection but is not sufficient if
  server infrastructure can be traced to real identities through hosting
  providers or network analysis.
- Years of pseudonymous public communication with users does not necessarily
  accelerate identification, but any infrastructure with a traceable hosting
  footprint creates exposure.

### What These Cases Mean for NyxForge

NyxForge shares the relevant structural features of both prosecuted protocols:
it is permissionless, non-custodial, privacy-preserving, and enables anonymous
financial transactions. The distinguishing argument -- that NyxForge funds
social outcomes rather than obscuring transaction trails -- may not protect
against a regulator motivated by political rather than legal reasoning. The
Tornado Cash contracts were designed for financial privacy, not money laundering.
The prosecution did not accept the distinction.

The common factor in both cases is that the developers were identifiable.
Pertsev was known; Rodriguez and Hill were known. Their identities gave
prosecutors a target. The Tornado Cash contracts themselves continue to run
today, unaltered and unstoppable. The arrests did not shut down the protocol.
They punished the people who could be found.

Bitcoin's author or authors have never been identified. The Bitcoin protocol
has survived multiple regulatory attempts to restrict it, in part because there
is no founding team to subpoena or prosecute. Satoshi's anonymity was not
incidental to Bitcoin's survival -- it was structural.

Anonymous development does not make NyxForge disappear or make it less useful.
It makes NyxForge persecution-resistant by eliminating the target. A protocol
with no known author has no one to arrest.

### Funding Anonymous Development

If wealthy investors and VC firms could issue outcome-linked bonds today, they
would likely use capital to hire scientists, programmers, and lobbyists, paying
only on verified results. That is precisely what NyxForge enables. The
difficulty is that NyxForge does not exist yet, and the investors who need it
most cannot use it to fund its own construction.

The solution is to replicate the bond mechanism manually, using milestones in
place of oracle attestations.

A donor commits funds in escrow that are released upon verified delivery of a
specific development milestone. The developer is anonymous; the deliverable is
verifiable. The donor does not need to trust the developer's identity, only
their ability to produce the specified output.

### Escrow mechanism: Monero 2-of-3 multisig

The escrow is implemented using Monero's native multisig capability. Three
parties each generate a portion of a shared wallet:

- The funder (donor)
- The developer (anonymous)
- A neutral arbitrator (a trusted member of the Monero community or a
  pre-agreed pseudonymous third party)

Funds are sent to the multisig address. To release a payment, any two of the
three parties must sign the transaction. The arbitrator cannot steal the funds
-- they hold only one of three required signatures -- but they can break a
deadlock if a dispute arises over whether a milestone was delivered.

This construction has several properties that suit the NyxForge bootstrap:

- Neither party needs to reveal their real identity. All three participants
  can be pseudonymous XMR addresses.
- The funder cannot withhold payment after verified delivery without the
  arbitrator siding with them. The developer has recourse.
- The developer cannot claim payment for undelivered work without the
  arbitrator siding with them. The funder has recourse.
- The arbitrator cannot collude unilaterally -- they need one of the two
  principals to complete a transaction.
- The entire arrangement is on-chain and final. There is no platform to
  deplatform either party.

The arbitrator for early milestones could be a known, pseudonymous figure in
the Monero community with a reputation to protect -- a long-standing contributor,
a CCS moderator, or a respected forum member. Their identity does not need to
be real-world verified; their reputation within the community is sufficient
collateral.

### Milestone sequence

Table.3.Milestones

| Milestone | Deliverable | Payment |
| :--- | :--- | :--- |
| 1 | Development plan and v1 spec reviewed and accepted | $5,000 |
| 2 | nyxforge-bond crate: all schema tables, passing test suite | $15,000 |
| 3 | CLI: bond create, inspect, status working on stagenet | $10,000 |
| 4 | Oracle workflow: accept, attest, evidence BLOBs, quorum | $10,000 |
| 5 | XMR DLEQ lock and sweep end-to-end on stagenet | $25,000 |
| 6 | Flutter UI functional on desktop (macOS/Linux) | $20,000 |
| 7 | First bond issued on Monero mainnet | $10,000 |
| Total | | $95,000 |

At each step, the funder and arbitrator review the deliverable -- typically a
git commit, a stagenet transaction ID, or a screen recording -- and co-sign
the release transaction. No personal identity is required at any point. The
developer receives XMR to a stealth address; the funder commits XMR from a
stealth address.

If the funder decides the pace of progress is unsatisfactory, they simply
decline to fund the next milestone. The developer loses only the opportunity
cost of time already spent. The funder loses only the funds already released,
which correspond to delivered work.

The bootstratpping process would demonstrate that NyxForge's incentive
model works before the platform exists to implement it natively.

---

## Development Cost Estimate

The following estimates assume a single full-time developer building a Rust
backend with a Flutter desktop UI. The largest cost driver is Phase 1: there
is no production-ready Rust library for XMR-specific DLEQ adaptor signatures.
The developer will implement this from academic specifications and validate
against Monero stagenet.

Table.4.DevCost

| Phase | Work | Estimate |
| :--- | :--- | :--- |
| 1 | nyxforge-bond crate, DLEQ primitives | 6-10 weeks |
| 2 | Bond file operations (create, inspect, transfer, verify) | 3-4 weeks |
| 3 | Oracle tooling, attestation, HTTP-JSON adapter | 3-4 weeks |
| 4 | XMR lock/sweep transactions, stagenet E2E | 6-8 weeks |
| 5 | Flutter UI (8-10 screens, local RPC client) | 5-8 weeks |
| 6 | Integration, error handling, UX testing | 2-3 weeks |
| 7 | Mainnet bond, documentation, grant applications | 2-3 weeks |
| Total | | 27-40 weeks |

Table.5.CostRange

| Developer rate | Low estimate | High estimate |
| :--- | :--- | :--- |
| $100/hr (mid-level) | $108,000 | $160,000 |
| $150/hr (senior Rust/crypto) | $162,000 | $240,000 |

The milestone sequence in Table.3.Milestones totals $95,000. A complete funding
commitment of $100,000-$160,000 covers the low end of the range at mid-level
rates and funds a working v1 with Flutter UI and one live mainnet bond.

Two factors most likely to extend the timeline beyond the high estimate:

1. DLEQ implementation complexity. If the XMR adaptor signature protocol
   requires implementing cryptographic primitives from scratch rather than
   composing existing library functions, Phase 1 could consume 12 or more
   weeks instead of 6-10.

2. Flutter learning curve. A developer starting from zero Flutter knowledge
   adds 3-5 weeks to Phase 5. A developer already proficient in Flutter
   reduces Phase 5 to 4-5 weeks.

## The Ask

A working v1 -- Rust backend with Flutter desktop UI, DLEQ collateral on
Monero mainnet, CLI and GUI oracle workflow -- requires roughly 27-40 weeks
of focused development from a single senior Rust developer, at a total cost
of $100,000-$240,000 depending on rate and DLEQ implementation difficulty.
See Table.4.DevCost and Table.5.CostRange above.

The recommended funding path is milestone-based 2-of-3 multisig escrow as
described above, supplemented by grants from aligned foundations:

Table.6.FundingSources

| Source | Mechanism | Rationale |
| :--- | :--- | :--- |
| Anonymous XMR donors | 2-of-3 multisig milestone escrow | Aligned investors per Cases 1-3 |
| Monero CCS | Community grant in XMR | XMR holders benefit from Case 1 |
| Zcash Foundation | Foundation grant | ZEC holders benefit from same logic |
| Gitcoin / Octant | Public goods funding rounds | Open-source infrastructure |

 







