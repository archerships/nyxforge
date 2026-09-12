# Research Note: Bearer Bond Simplification and Funding Models
> Started: 2026-04-18
> Status: ongoing -- append as conversation continues

---

## 1. Could bonds be self-contained, self-verifying bearer bonds?

### The core tension in the current architecture

The current NyxForge system bundles two separate concerns:

1. Ownership tracking -- ZK notes, nullifiers, P2P network, order book (complex)
2. Goal verification and payout -- oracle attestation, collateral release, .bond file (can be simple)

Almost all the complexity lives in #1. The question is whether bonds could be bearer
instruments (possession = ownership), eliminating most of that complexity.

### What a bearer model looks like

A .bond SQLite file contains:
- GoalSpec (what needs to happen)
- Oracle pubkeys
- XMR DLEQ adaptor signature (partial sig blob)
- Evidence BLOBs
- WASM verifier

Ownership = knowledge of a secret scalar (one per bond file).
Transfer = hand over the file + scalar (off-chain, over Tor/Signal/etc).
Redemption = oracle publishes s_met → adaptor + s_met completes XMR tx → holder sweeps.
Double-spend prevention = XMR UTXO model (collateral output can only be swept once).

### What you gain vs. current spec

- No libp2p node to run
- No DarkFi dependency
- No MINT/TRANSFER ZK circuits
- No nullifier set (shared state)
- Fully air-gapped redemption with only the .bond file + XMR wallet
- Stateless: the file IS the bond, forever

### What you lose

- Fractional units (bond is indivisible; no liquid order book)
- Market price signal (Horesh's core information mechanism)
- Anonymous transfer (file handoff is only as private as the channel)
- Order book / DEX
- Divisibility for large-collateral goals requiring broad participation

The loss of fractional units is the most serious. Horesh's incentive mechanism depends
on bonds aggregating toward the most capable actors via open-market competition.

### Proposed progression

| Stage | Model | What you get |
| :--- | :--- | :--- |
| MVP | Bearer files, standardized collateral, human oracles | Proves incentive loop end-to-end |
| v2 | ZK notes wrapping the bearer layer | True fungibility, anonymous transfer |
| v3 | Order book + price feed | Continuous market price signal |

---

## 2. XMR DLEQ -- what it is

DLEQ = Discrete Log EQuality proof.

Solves the oracle custody problem: oracle authorizes payout without ever holding funds.

Setup:
- Issuer + oracle pre-commit to two scalars: s_met (goal achieved) and s_fail (not achieved / expired)
- XMR collateral locked: output_A unlocked by s_met, output_B unlocked by s_fail

Oracle attests goal met:
- Oracle publishes s_met (does not hold a full signing key; cannot steal)

Bondholder redeems:
- adaptor + s_met → complete XMR tx signature → sweep output_A

Expiry / not met:
- Oracle publishes s_fail → issuer sweeps output_B (refund)

The DLEQ proof lets oracle prove the scalar it reveals is the one it committed to at
setup, without revealing the other scalar. Same primitive as XMR/BTC atomic swaps.
Well-tested, not experimental.

---

## 3. Standardized collateral units

If all bonds use a fixed collateral unit (e.g. 1 XMR per file):

- Price comparison across bonds is trivial (all trade on 0-1 XMR scale)
- Bond price = market's probability estimate of goal being met by deadline
- Fractional ownership via issuing N separate 1-XMR files for the same GoalSpec
- Horesh price signal preserved: price of one file = P(goal met)

Example Lifebond:
  Collateral: 1 XMR per file, 1000 files issued = 1000 XMR total
  Condition: subject alive, happy, healthy on 2036-01-01
  Price today: ~0.7 XMR if market assigns 70% survival probability

Caveat on fungibility: each file has a different adaptor scalar, so files are not
literally interchangeable (unlike ERC-20 tokens). "Fungible enough" for a pilot --
same price, same condition, same collateral. True fungibility requires ZK notes (v2).

---

## 4. Multi-currency collateral

The .bond format is currency-agnostic. The collateral field specifies the currency
and the appropriate locking primitive:

| Currency | Primitive | Privacy | Maturity |
| :--- | :--- | :--- | :--- |
| XMR | DLEQ adaptor signatures | High (on-chain) | Production |
| ZEC | DLEQ (Sapling, same as XMR) | High | Production |
| BTC | DLC (Taproot/Schnorr) | Low | Production |
| ETH / ERC-20 | Smart contract escrow | Low | Production |
| AR | Warp/SmartWeave contract | Low | Maturing |
| DRK | Native ZK contract | High | Testnet only |

Privacy tradeoff: XMR/ZEC hide redemption amount and recipient on-chain. BTC/ETH
do not -- payout transaction is fully public. For anonymous Lifebonds, XMR or ZEC.
For public philanthropic bonds where transparency is a feature, ETH is simpler.

---

## 5. Funding the marketplace -- mechanisms reviewed

### Problem
Most funding mechanisms require a central entity, conflicting with permissionless design.

### Oracle service fees (most consistent with design goals)
- Developers run reference oracle infrastructure, charge per attestation
- Legitimate service fee, not a protocol tax
- Anyone can run an oracle; developers earn fees by being most reliable
- Revenue scales with platform usage
- Moat: reputation and longevity (200-year oracle worth more than cheap one)

### Issuance fee in reference client
- Reference client charges small fee (e.g. 0.5% of collateral) at bond creation
- Fee goes to published multisig address
- Opt-out possible (fork the client); opt-out unlikely for most users
- Precedent: Uniswap, most open protocols use this model

### NyxForge development bond (most philosophically consistent)
- Issue a bond paying out when NyxForge reaches a measurable milestone
  (e.g. "1000 XMR total collateral locked by 2028")
- Investors buy the bond; bondholders are financially incentivized to fund
  development, marketing, and lobbying
- This is the Wall Street Performer Protocol applied to NyxForge itself
- Also serves as public demonstration the system works before third-party bonds
- Most compelling narrative: "we fund our platform with our own instrument"

### Founding endowment + yield (for long-term sustainability)
- Lock a founding endowment in productive assets
- Yield funds ongoing oracle operations and development indefinitely
- See Section 6 for why XMR does NOT work for this; ETH/stETH does

### Recommended combination

| Need | Mechanism |
| :--- | :--- |
| Near-term development | NyxForge development bond (milestone crowdfund) |
| Ongoing oracle operations | Oracle service fees |
| Long-term sustainability | Founding endowment yield (stETH, not XMR) |
| Marketing / legal / lobbying | % of issuance fee in reference client |

---

## 6. Precedents: Uniswap and Tornado Cash

### Uniswap
- Initially funded: ~$65k Ethereum Foundation grant, then VC (a16z, Paradigm, USV)
- Series A: $11M (2020); Series B: $165M (2022)
- UNI governance token: retroactive airdrop of 400 UNI to all historical users (Sept 2020)
- "Fee switch" allows UNI holders to redirect trading fees to treasury (debated, not consistently active)
- SEC Wells Notice April 2024; no charges as of knowledge cutoff
- Protected by: incorporated entity, VC legal resources, a16z DC lobbying operation

### Tornado Cash
- Initially funded: community grants and early contributor donations
- TORN governance token: retroactive airdrop (Dec 2020)
- August 2022: OFAC sanctioned the smart contract addresses (not just developers)
  -- unprecedented: sanctioning open-source code itself
- Alexey Pertsev arrested Netherlands (2022); Roman Storm arrested US (2023)
- Protocol continues running (immutable contracts)

### Lessons for NyxForge
1. VC funding = named, hittable target; corporate structure = defendants
2. Governance token from identified founding team = potential security (SEC)
3. Development bond is more defensible than a token: outcome-linked, not speculative equity
4. Anonymous founding team + permissionless protocol + no token = smallest legal surface area
5. Tornado Cash convicted partly because founders were known and had incorporated
6. Framing matters: exchange (Uniswap) vs mixer (Tornado Cash) vs outcome finance (NyxForge)

---

## 7. Gitcoin and Octant

### Gitcoin
- Quadratic funding: matching pool + individual donations; matching proportional to
  sqrt of donations (breadth of support weighted over whale concentration)
- GTC governance token
- Went through major restructuring 2023-2024 (layoffs, Owocki stepped back)
- Pivoted from product company to protocol foundation (Allo Protocol)
- Sustainability problem: expensive staff + declining GTC price + donor fatigue

### Octant (Golem Foundation)
- Golem Foundation locked 100,000 ETH in Ethereum beacon chain
- Staking yield (not principal) funds public goods every quarter
- GLM token holders lock tokens, earn yield share, direct it to projects
- Endowment is permanent; funding is perpetual yield

### NyxForge parallel
Both fund activity (teams, development). NyxForge funds outcomes. Horesh would say
both are still input-based, not output-based.

A NyxForge development bond issued via Gitcoin's Allo Protocol = quadratic-funded,
outcome-linked, anonymous instrument. Worth exploring as launch strategy.

Gitcoin's failure mode: funded others' outcomes, had no mechanism for its own.
NyxForge oracle fees + dev bond avoids this trap.

---

## 8. Arweave AO funding mechanism

### How it works
- Users bridge productive assets (initially stETH) into AO
- Bridger keeps principal (withdrawable at any time)
- stETH staking yield flows to AO ecosystem treasury
- Bridger receives AO tokens minted over time (Bitcoin-like emission curve)
- 1/3 of AO tokens to AR holders; 2/3 to bridge participants

### Why it works
- Treasury income is continuous and automatic (yield never stops)
- Principal never spent (depositors can always leave)
- No donor fatigue, no quarterly fundraising
- Scales with TVL: more bridges = more yield = more treasury
- Fair launch: no pre-mine, no VC discount

### Critical correction: XMR does NOT produce yield

XMR locked as bond collateral earns NOTHING. Monero is proof-of-work; there is no
staking, no native yield. Locked XMR just sits in a DLEQ output waiting for release.

The v2.0 spec's "XMR Yield Endowment" requires the issuer to run P2Pool mining
hardware separately -- yield comes from mining work, not from locked collateral.
This is operationally complex and unpredictable, not passive yield.

| | stETH in AO | XMR in NyxForge bond |
| :--- | :--- | :--- |
| Yield source | Ethereum validator rewards | Nothing (no native yield) |
| Requires work | No | Yes (mining hardware) |
| Predictable | ~4% APY | No |
| Passive | Yes | No |

### Practical options for NyxForge yield treasury
1. Use stETH/ETH as collateral -- AO model applies directly, yield is real and passive
2. Accept XMR is non-yielding; fund treasury through oracle fees and issuance fees only
3. Multi-currency collateral: ETH/stETH bonds contribute yield; XMR bonds do not

For the bearer bond MVP, option 2 is the honest path.

---

## 9. Decision: Go with the simplest structure

Agreed 2026-04-18. The bearer bond MVP is the current build target.

Formalized in `doc/00_MVP.md`. Key decisions locked:

- Bearer file model (no ZK ownership circuits)
- XMR DLEQ for collateral (no custodian)
- Fixed collateral unit per file (enables price signal)
- Human oracle panel for qualitative; HTTP-JSON for quantitative
- Oracle operators are domain experts, separate from developers
- Developer revenue: oracle fees + issuance fee in reference client
- Bootstrap funding: grants + NyxForge development bond
- No token, no DAO, no corporate entity for MVP

Deferred to v2: ZK notes, P2P network, order book, DarkFi, browser UI.

Implementation: 5 phases, ~12 weeks total.
First milestone: NyxForge Development Bond on Monero mainnet.
