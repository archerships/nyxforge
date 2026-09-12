> **STATUS: HISTORICAL / SUPERSEDED — 2026-04-25**
> This document describes the original DarkFi L1 / DRK-token / ZK-note architecture
> that was the pre-MVP NyxForge design (March 2026). It was superseded by the
> bearer-file approach specified in `doc/00_MVP.md`: SQLite .bounty files,
> DLEQ/PTLC adaptor signatures, XMR/ZEC/BTC/ETH collateral, no DarkFi dependency.
> Preserved for historical reference only. Do not use as an authority source.


# NyxForge User Manual

> Anonymous, decentralised social policy bond market

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Installation & Setup](#2-installation--setup)
3. [Starting and Stopping the Node](#3-starting-and-stopping-the-node)
4. [Wallet](#4-wallet)
5. [Creating a Bond](#5-creating-a-bond)
6. [Viewing Bonds](#6-viewing-bonds)
7. [Trading Bonds](#7-trading-bonds)
8. [Redeeming Bonds](#8-redeeming-bonds)
9. [Running an Oracle](#9-running-an-oracle)
10. [Privacy Model](#10-privacy-model)
11. [Troubleshooting](#11-troubleshooting)
12. [Glossary](#12-glossary)

---

## 1. Introduction

**Social Policy Bonds** are financial instruments that pay out only when a
measurable social or environmental goal is achieved — reduced homelessness,
lower CO₂ levels, improved literacy rates, and so on.  Traditional schemes
require trusted institutions to issue and settle bonds.

**NyxForge removes that requirement.** Anyone can:

- **Define** a goal with verifiable, on-chain criteria.
- **Issue** bonds backed by a DarkFi DAO treasury or individual collateral.
- **Trade** bonds anonymously on a ZK order-book DEX.
- **Verify** goal completion through a decentralised oracle network.
- **Redeem** bonds via anonymous ZK settlement — no KYC, no bank, no app store.

Bond ownership and all trades are zero-knowledge anonymous.  Only goal
verification results are public.

---

## 2. Installation & Setup

### Prerequisites

- Rust 1.79+ (`rustup` recommended)
- `wasm-pack` — `cargo install wasm-pack`

### Build from source

```bash
git clone <repo-url>
cd nyxforge
cargo build --workspace
```

### First run

```bash
./nyxforge --start
```

This builds the node binary (if needed), starts the background node process,
waits for the RPC server to become available, and opens the web UI.

---

## 3. Starting and Stopping the Node

The `nyxforge` script manages all background processes.

| Command | Effect |
|---|---|
| `./nyxforge --start` | Build and start node + UI (default) |
| `./nyxforge --stop` | Stop all NyxForge processes |
| `./nyxforge --status` | Show live status of node, UI, RPC, miner, and wallet |
| `./nyxforge --dryrun` | Print all actions without executing them |
| `./nyxforge --help` | Print usage |

### Status output

```
nyxforge-node   pid 12345   up 0:03:22
flutter-ui      pid 12346   port 8080
RPC             http://127.0.0.1:8888/rpc   OK
miner           active   hashrate ~1.2 kH/s
wallet          drk: 03a1b2c3...
```

### Cleaning up build artifacts

```bash
./scripts/nyxforge-clean.sh               # removes build artifacts only
./scripts/nyxforge-clean.sh --include data  # also removes node data + wallet
./scripts/nyxforge-clean.sh --dry-run     # show sizes without deleting
```

> **Warning:** `--include data` deletes your wallet keys. There is no recovery
> if keys are lost. Back up your seed phrase first.

---

## 4. Wallet

A wallet must exist before you can issue bonds or trade.

### Create a wallet

```bash
nyxforge-cli wallet create
```

This generates a new keypair and stores it locally.  Your DRK public key is
shown on creation — keep a record of it.

### Show wallet addresses

```bash
nyxforge-cli wallet addresses
```

Output includes:

- `xmr` — Monero-compatible address (for receiving DRK)
- `drk` — DarkFi public key (used as oracle key and issuer identity)

---

## 5. Creating a Bond

Bond creation has three sequential steps: running the wizard to write the
`.bond` file, collecting oracle acceptance, and issuing (locking collateral
on-chain).  No collateral is at risk until Step 3.

---

### Step 1 -- Run the wizard

```
nyxforge-cli bond create
```

The wizard walks through every field in order.  Press Enter to accept a
default.  Press Ctrl-C at any point to cancel; no file is written until
you confirm at the preview screen.

---

#### Goal

**Title** -- A short human-readable name.
Example: `US Homelessness Reduction 2030`

**Description** -- A longer explanation of what the bond is trying to achieve.

**Goal type** -- One of `quantitative`, `qualitative`, or `hybrid`.

- Quantitative: evaluated against a measurable criterion (a data source,
  operator, and threshold).  Oracle fetches the value and applies the
  comparison automatically.
- Qualitative: evaluated by human oracle judgment.  Oracles review evidence
  BLOBs submitted to the `.bond` file and sign their assessment.
- Hybrid: both criterion and human judgment required.

---

#### Criterion (quantitative and hybrid goals only)

**Data ID** -- A dot-namespaced string identifying the real-world data source.
Convention: `<source-authority>.<dataset>.<metric>`.

| Data ID | Source | Measures |
| :--- | :--- | :--- |
| `us.hud.pit_count.unsheltered` | HUD Point-in-Time survey | Unsheltered US homeless count |
| `noaa.co2.monthly_mean_ppm` | NOAA Mauna Loa Observatory | Atmospheric CO2 (ppm) |
| `who.malaria.deaths_per_100k` | WHO Global Health Observatory | Malaria deaths per 100k |
| `us.nrc.new_reactor_approvals` | US Nuclear Regulatory Commission | New reactor construction permits |

Confirm that your chosen oracle operators support the Data ID before issuing.
There is no global registry; the issuer and oracle panel agree on the meaning
when the oracle accepts the goal spec.

**Operator** -- The comparison applied to the observed value vs the threshold.

| Operator | Meaning | Goal is met when... |
| :--- | :--- | :--- |
| `lt` | less than | observed value < threshold |
| `lte` | less than or equal | observed value <= threshold |
| `gt` | greater than | observed value > threshold |
| `gte` | greater than or equal | observed value >= threshold |
| `eq` | equal | observed value == threshold |

**Threshold** -- The numeric target value.  Example: `50000`

**Aggregation** *(optional)* -- How the raw data stream is reduced to a single
value before comparison.  Example: `annual_mean`, `annual_point_in_time`.
Leave blank to compare the raw value directly.

---

#### Timing

**Deadline** -- The date by which the goal must be achieved.  Format:
`YYYY-MM-DD`.  Example: `2030-01-01`

**Expiry** -- The date after which, if quorum has not been reached with result
`met`, the oracle publishes `s_fail` and the issuer may reclaim collateral.
Must be after the deadline.  Format: `YYYY-MM-DD`.

The gap between deadline and expiry is the settlement window: time for oracles
to gather evidence, reach quorum, and allow the challenge period to pass.  A
bond past its deadline but before expiry remains ACTIVE.

Example: deadline `2027-06-30`, expiry `2027-07-31`.  If the goal is met by
June 30, oracles have July to attest and reach quorum (-> REDEEMABLE).  If
they cannot confirm the goal was met, the bond expires July 31 (-> EXPIRED)
and collateral returns to the issuer via `bond reclaim`.

Rule: `expiry > deadline`.  The wizard enforces this at input time.

**Grace period** -- Number of days after expiry before the XMR timelock
matures.  Default: `30`.

If oracles are unable or unwilling to publish `s_fail` (incapacitated,
unresponsive, or defunct), the collateral would otherwise be locked
permanently.  The grace period is a safety net: an XMR timelock is set to
mature at `expiry + grace_days`.  Once that date passes, the issuer can sweep
collateral without any oracle involvement.  Under normal operation the oracle
publishes `s_fail` before the grace period elapses.  A value of 30-90 days
is typical.

---

#### Collateral

**Currency** -- Collateral currency.  One of: `XMR` (Monero), `ZEC` (shielded
Zcash Sapling), `BTC` (Bitcoin Taproot), `ETH` (Ethereum).  Default: `XMR`.
The locking mechanism is determined automatically from the currency (see
Section 4.4 of the spec for details).

**Chain** *(ETH only)* -- Target Ethereum network.  `mainnet` (chain ID 1),
`Sepolia` (11155111), or `Holesky` (17000).

**Amount per file** -- Collateral locked inside each `.bond` file, in the
chosen currency.  All files in the series carry this amount.  Example: `1.0`

**Redemption value** -- Amount paid to the holder on redemption, in the
collateral currency.  May equal or differ from the locked amount depending on
the issuance design.  Optional informational field; not enforced on-chain.

**Series size** -- Number of identical `.bond` files to create.  All units
share the same goal, collateral, and oracle panel but have unique serial
numbers.  Example: `100`

---

#### Oracle panel

**Oracle public keys** -- One or more 32-byte hex pubkeys of oracle operators
who will attest on this bond.  Comma-separated.

**Oracle roles** -- Per oracle: `quantitative`, `qualitative`, or `both`.

**Oracle fees** -- Fee per oracle per attestation event, denominated in the
bond's collateral currency.  Paid at issuance, not at attestation time.

**Quorum** -- Minimum number of agreeing attestations to reach a result.
Must be <= the number of oracle panel members.  Default: `3`.

**Challenge period** -- Days after quorum is reached during which the result
can be disputed before the bond is finalised.

---

#### Preview and confirm

After all prompts the wizard prints a full preview of the bond spec.  Enter
`y` to write the `.bond` file(s) to disk in DRAFT state.  No collateral is
locked at this point.

```
--- Bond preview ---
title:          US Homelessness Reduction 2030
goal_type:      quantitative
data_id:        us.hud.pit_count.unsheltered
operator:       lte  threshold: 50000
deadline:       2030-01-01  expiry: 2030-02-01  grace_days: 30
currency:       XMR  mechanism: dleq_xmr
collateral:     1.0 XMR per file  x 100 files
oracle panel:   3 oracles  quorum: 3  challenge: 7 days

Write bond files? [y/N]
```

On confirmation the CLI writes `<series_id>-0001.bond` through
`<series_id>-0100.bond` and prints the series ID.

---

### Step 2 -- Oracle acceptance

Each oracle in the panel must review the goal spec and accept or reject before
the bond can be issued.  Share each `.bond` file with the relevant oracle
operator out-of-band (Signal, email, Tor).

The oracle runs:

```
nyxforge-cli oracle accept <file>    # accept; writes accepted_at to the file
nyxforge-cli oracle reject <file>    # decline; writes rejection reason
```

Check acceptance status:

```
nyxforge-cli bond status <file>
```

Output:
```
state:    DRAFT
oracles:
  03a1b2c3...  accepted   2030-01-15T12:00:00Z
  04d5e6f7...  rejected   2030-01-15T13:00:00Z
    reason: data_id ambiguous -- specify sheltered vs unsheltered
  08a9b0c1...  pending
```

If an oracle rejects: revise the goal spec to address the objection and
re-run the wizard.  Oracle acceptance cannot be updated in place; a revised
bond is a new file.

All oracles must accept before proceeding to Step 3.

---

### Step 3 -- Issue the bond

Once all oracles have accepted, lock collateral on-chain:

```
nyxforge-cli bond issue <file>
```

This is the cryptographically significant step.  The CLI performs the
appropriate setup for the bond's collateral currency:

For XMR, ZEC, and BTC (DLEQ/PTLC path):

1. The oracle pre-commits to two scalars -- `s_met` (goal achieved) and
   `s_fail` (goal not achieved) -- and publishes their public commitments
   `t_met = s_met*G` and `t_fail = s_fail*G` to the `.bond` file.  The
   adaptor partial transaction is stored in `collateral.adaptor` and can
   only be completed by the holder of `s_met` or `s_fail`.

2. The CLI broadcasts a locking transaction to the appropriate network
   (Monero, Zcash Sapling, Bitcoin Taproot) and writes `lock_txid` to the
   file.

For ETH (smart contract escrow path):

1. A NyxForge Escrow contract is deployed to the target chain (mainnet,
   Sepolia, or Holesky).  The oracle's per-bond Ethereum address is
   registered as the authorized releaser.

2. ETH is sent to the contract address, which holds it until the oracle
   authorises release or the timelock matures.

In all cases the CLI then:

3. Transitions the bond from DRAFT to ACTIVE in the `history` table.

After this step the bond is live.  The collateral is on-chain and cannot be
moved without either the oracle's settlement credential (`s_met` scalar or
EIP-712 release signature) or -- as a safety net -- the on-chain timelock
at `expiry + grace_days`.  Neither the oracle nor the developer can steal it.

The bond file is now a bearer instrument.  Whoever holds the file and can
decrypt the scalar inside it is the owner.  See [Trading Bonds](#7-trading-bonds).

---

### Goal specification examples

**Housing**
```
data_id:     us.hud.pit_count.unsheltered
operator:    lte
threshold:   50000
aggregation: annual_point_in_time
deadline:    2030-01-01
expiry:      2030-02-01
grace_days:  30
```

**Environmental**
```
data_id:     noaa.co2.monthly_mean_ppm
operator:    lt
threshold:   350.0
aggregation: annual_mean
deadline:    2045-01-01
expiry:      2045-03-01
grace_days:  60
```

**Public health**
```
data_id:     who.malaria.deaths_per_100k
operator:    lt
threshold:   1.0
aggregation: global_annual
deadline:    2035-01-01
expiry:      2035-03-01
grace_days:  60
```

---

## 6. Viewing Bonds

### List all bonds

```bash
nyxforge-cli bond list
```

Prints a table with Bond ID, state, and title for every bond the node knows about.

```
Bond ID                                                             State         Title
----------------------------------------------------------------------------------------------------
a1b2c3d4e5f6...                                                     Draft         US Homelessness ...
```

### Bond states

Table.1.BondStates

| State | Short | Entry criteria | Description |
| :--- | :--- | :--- | :--- |
| DRAFT | draft | Wizard completes and confirms; `.bond` file written to disk | File exists; no collateral locked; oracles have not yet accepted |
| ACTIVE | active | All oracles accepted; issuer runs `bond issue`; collateral locked on-chain via DLEQ/PTLC or ETH escrow | Collateral is live; file is a tradeable bearer instrument |
| REDEEMABLE | redeem | Oracle quorum reached with result `met`; `s_met` scalar published | Holder may run `bond redeem` to sweep XMR collateral |
| SETTLED | settled | Holder runs `bond redeem`; adaptor + `s_met` broadcasts XMR sweep | Collateral has been paid out; file is spent |
| EXPIRED | expired | Expiry date passes with no quorum reached | Goal not met in time; collateral returnable to issuer |
| RECLAIMED | reclaim | Issuer runs `bond reclaim`; oracle publishes `s_fail` (or timelock matures at expiry + grace_days) | Collateral returned to issuer; file is closed |

### Inspect a single bond

```bash
nyxforge-cli bond get <BOND-ID>
```

Prints the full bond JSON including goal spec, oracle configuration, and
current state.

---

## 7. Trading Bonds

Bonds are bearer instruments: whoever holds the `.bond` file and can decrypt
the scalar inside it is the owner.  Trading is bilateral and off-chain --
there is no order book or DEX in the MVP.  An exchange acts purely as a
matchmaking service; it never takes custody of the bond.

Bond price on the secondary market reflects the crowd's estimate of the
probability the goal will be met before the deadline:

```
price ≈ redemption_value x P(goal met before deadline)
```

As the deadline approaches and the metric improves or worsens, the price
adjusts.  This price signal is itself useful social information independent
of any individual trade.

---

### Buying directly from an issuer (primary issuance)

When you buy a bond directly from the person who created it, there is a
subtlety that does not apply to secondary trades: the issuer was present at
bond creation and could have retained the secret scalar that controls
redemption.  Simply re-encrypting the scalar to your public key after the fact
does not help -- the issuer still has the plaintext from before.

NyxForge eliminates this risk by letting you participate in the bond setup
before collateral is locked:

1. Generate your keypair if you do not have one:

```
nyxforge-cli wallet show
```

Your public key is shown in the output (`holder_pubkey` field).

2. Send your public key to the issuer before they run the creation wizard.

3. The issuer runs the wizard with your pubkey:

```
nyxforge-cli bond create --holder-pubkey <your_pubkey_hex>
```

   With this flag set, the scalar that controls redemption is generated on
   your machine (derived from your private key) and the issuer never sees it.
   The lock address is derived from your pubkey and the oracle adaptor point.

4. Once the bond is issued and `lock_txid` is visible on-chain, verify it:

```
nyxforge-cli bond inspect <file>
nyxforge-cli bond verify <file>
```

   Confirm `collateral.holder_pubkey` matches your public key and
   `collateral.lock_txid` shows the correct amount on the correct network.

5. Pay the issuer.

If the issuer refuses to use `--holder-pubkey` and instead offers to
"transfer" a bond they already created to themselves, treat this as an
untrusted transaction: the issuer could have retained the scalar and could
sweep the collateral if the goal is met.  Either insist on a fresh bond with
your pubkey, or apply the same diligence you would give any counterparty
whose honesty you cannot verify.

---

### Proving ownership to an exchange (without transferring)

A holder can prove they own a bond and share its details with an exchange
without handing over the file or the scalar.  The exchange issues a nonce;
the holder signs it with the private key corresponding to `holder_pubkey`
in the file.

```
nyxforge-cli bond prove <file> --nonce <hex> --ask <amount> --list-expiry <YYYY-MM-DD>
```

This produces a signed listing record containing:

- Bond summary (title, state, collateral, deadline) -- readable by anyone
- `holder_pubkey` -- the key the exchange uses to verify the signature
- `ask` and `list_expiry` -- the terms of the listing
- `sig` -- holder's signature over `blake3(bond_id || nonce || ask || list_expiry)`

The exchange verifies the signature against `holder_pubkey` and confirms
that pubkey matches the `collateral.holder_pubkey` field in the bond file
(requested with `scalar_encrypted` redacted).  If valid, the listing is
published.  The holder's private key and scalar never leave their machine.

---

### Selling: full transfer to buyer

Once a buyer is found (on- or off-exchange), the seller transfers the bond
directly to the buyer peer-to-peer:

```
nyxforge-cli bond transfer <file> <buyer_pubkey>
```

This re-encrypts the scalar inside the file to the buyer's public key and
updates `holder_pubkey`.  The seller sends the updated file to the buyer
(over Signal, Tor, or any channel).  The buyer simultaneously sends payment
(XMR or agreed currency) to the seller.  There is no atomic swap in MVP --
both parties must trust the exchange's escrow or each other directly.

After transfer the seller's copy of the file is inert: the scalar is now
encrypted to the buyer's key and the seller cannot decrypt it.  This is
enforced by the file format -- `scalar_encrypted` is an ECIES ciphertext bound
to `holder_pubkey`, and `bond verify` will reject any file whose ciphertext
does not match the stated holder.

Security note for sellers: once you run `bond transfer`, you lose the ability
to redeem even if the goal is met before you receive payment.  Verify the
buyer's payment commitment before running the command, or use an exchange
escrow.

Security note for buyers in a secondary trade: the seller legitimately held
the scalar during their ownership period.  Unlike a primary issuance, you
cannot prove the seller deleted their pre-transfer copy.  This is the same
trust model as any bearer instrument (cash, paper bond): once you hold it,
you can redeem it, but a dishonest previous holder who did not truly transfer
could race you to redemption.  The risk is mitigated by the exchange escrow
(which confirms delivery before releasing payment) and by the on-chain
finality of the sweep: the first party to broadcast the sweep transaction wins.

---

### Buying: receiving a bond

When you receive a `.bond` file from a seller:

1. Verify the file is intact and the bond is in the expected state:

```
nyxforge-cli bond inspect <file>
nyxforge-cli bond verify <file>
```

2. Confirm `collateral.holder_pubkey` matches your public key (the seller
   should have run `bond transfer` with your pubkey before sending).

3. Confirm `collateral.lock_txid` is visible on the appropriate network
   (Monero, Zcash, Bitcoin, or Ethereum) with the correct amount locked.

---

## 8. Redeeming Bonds

When a bond reaches REDEEMABLE state:

```
nyxforge-cli bond redeem <file>
```

The CLI completes the payout using the oracle's settlement credential:

For XMR, ZEC, and BTC (DLEQ/PTLC): the oracle has published `s_met`.
The CLI combines `collateral.adaptor + s_met` to complete and broadcast
the sweep transaction, sending collateral to your wallet address on the
appropriate network.

For ETH: the oracle has published an EIP-712 `Release(address holder)`
signature.  The CLI calls `NyxForgeEscrow.release(oracle_sig)` on the
target chain, releasing ETH to your Ethereum address.

In both cases the CLI then writes SETTLED to the `history` table.  The file
is now spent -- the collateral has been paid out.

The redemption transaction itself is on-chain and therefore visible, but
your identity is not revealed: for XMR/ZEC the transaction appears as a
normal shielded sweep; for BTC it appears as a Taproot spend; for ETH it
is a contract call from your address.

---

## 9. Running an Oracle

Oracle nodes are the "eyes" of the NyxForge network. They monitor bonds, fetch real-world data, and post signed attestations when a goal is met.

### Sovereign Infrastructure & The "Long Now"
NyxForge is designed to support bonds with maturity dates ranging from **1 year up to 200 years** (matching the expected lifespan of the Arweave network). To ensure oracles survive for centuries, the system follows a **Sovereign Hardware** model:

- **Decentralized Execution:** Oracle nodes run on commodity hardware (home servers, laptops, or decentralized clouds like Akash). There is no central server to shut down.
- **Rotational Persistence:** We do not expect a single computer to run for 200 years. Instead, the **NYX Fair Launch Emission** creates a perpetual economic incentive. If an oracle operator goes offline, the unclaimed rewards attract a new operator to take over the task on modern hardware.
- **Evidence Archiving:** For long-term bonds, oracles don't just "check" a website; they fetch data and upload the signed evidence (PDFs, CSVs, zkTLS proofs) to the **Arweave Permaweb**.
- **Holographic Resolution:** Once evidence is on Arweave, it is permanent. In the year 2200, the bond can be resolved by "replaying" the immutable evidence logs, even if the original data source has been dead for a century.

### Oracle Requirements
Oracles must:

- Hold the oracle public key registered in the bond's `OracleSpec`.
- Stake at least `required_stake` DRK before attesting.
- Fetch data from the bond's `data_id` source.
- Sign a `true` or `false` attestation and gossip it to the P2P network.

Fraudulent attestations (e.g. attesting `true` when the goal was not met)
result in `slash_fraction` of staked DRK being burned.

---

## 10. Privacy Model

| What is public | What is private |
|---|---|
| Bond goal specs and deadlines | Who holds which bonds |
| Oracle attestation results | Trade amounts and counterparties |
| Nullifier set (prevents double-spend) | Wallet balances |
| Bond state transitions | Redemption amounts |

All bond ownership is represented as ZK notes.  Spending a note reveals its
nullifier (preventing double-spend) but reveals nothing about the owner, the
amount, or how long they held the note.

---

## 11. Troubleshooting

**`RPC request failed: Is the node running?`**
Run `./nyxforge --status` to check.  If the node is not running, start it
with `./nyxforge --start`.

**`No wallet found. Create one first with: nyxforge-cli wallet create`**
You must create a wallet before issuing bonds.  Run `nyxforge-cli wallet create`.

**`invalid bond ID: expected 32-byte hex`**
Bond IDs are 64 hex characters (32 bytes).  Copy the full ID from
`nyxforge-cli bond list`.

**`Node error: bond not found`**
The node does not have a record of that bond ID.  It may be on a different node
or may not have been submitted yet.

**`Must be between 1 and N` (attestation threshold)**
The attestation threshold cannot exceed the quorum you set earlier in the wizard.
Re-run `nyxforge-cli bond create` and enter a threshold ≤ quorum.

---

## 12. Glossary

Terms are listed alphabetically.  Cross-references appear as *italicized term*.

---

aggregation
: How a raw data stream is reduced to a single value before the *operator*
  comparison is applied.  Examples: `annual_mean`, `annual_point_in_time`,
  `sum`, `count`.  Leave blank to compare the raw value directly.

attestation
: A signed statement from an *oracle* declaring whether a bond's *goal* was
  met or not met.  Each attestation includes a result (`met` or `not_met`),
  a timestamp, and an optional evidence *BLOB*.  Once *quorum* attestations
  agreeing on `met` are recorded, the bond becomes REDEEMABLE.

bearer bond
: A bond whose ownership is determined entirely by possession of the `.bond`
  file and the corresponding secret scalar.  There is no name registry or
  custodian.  Transferring ownership means re-encrypting the scalar to the
  recipient's public key.

.bond file
: A SQLite archive (optionally encrypted with SQLCipher) that contains the
  complete state of a single bond unit: goal spec, oracle panel, collateral
  commitment, attestations, evidence BLOBs, and history.  The filename
  follows the pattern `seriesid-NNNN.bond`.

bond create
: The CLI wizard that collects goal type, criterion, timing, collateral, oracle
  panel, and quorum settings, then writes one or more `.bond` files to disk
  in DRAFT state.

challenge period
: A window of time (in days) after *quorum* is reached during which the result
  can be disputed before the bond is finalised as REDEEMABLE or EXPIRED.
  Set by the issuer at bond creation.

chain ID
: An integer that identifies the Ethereum network.  Used only for ETH
  collateral bonds.  1 = Ethereum mainnet; 11155111 = Sepolia testnet;
  17000 = Holesky testnet.  Stored in `collateral.chain_id` and determines
  which network the NyxForge Escrow contract is deployed to.

collateral
: Crypto assets locked on-chain as the payout backing for a bond.
  Supported currencies: XMR (Monero), ZEC (shielded Zcash Sapling),
  BTC (Bitcoin Taproot), ETH (Ethereum).  Held in a DLEQ/PTLC adaptor
  output or smart contract escrow during the ACTIVE state.  Released to
  the holder on goal met (s_met or oracle release sig) or returned to
  the issuer on reclaim (s_fail or oracle reclaim sig).

criterion
: The human-readable description of the condition that must be met for a
  *term* to be satisfied.  For quantitative terms, a machine-readable
  `data_id`, `operator`, and `threshold` accompany the criterion text.
  For qualitative terms, the criterion text is the sole specification.

data_id
: A dot-namespaced string identifying the real-world metric an *oracle* must
  observe to evaluate the bond's *criterion*.  Convention:
  `<source-authority>.<dataset>.<metric>`.  Examples:
  `noaa.co2.monthly_mean_ppm`, `clinicaltrials.gov.NCT04567890.enrollment`.
  There is no global registry; the issuer and oracle panel agree on the
  meaning when the oracle accepts the goal spec.

deadline
: The date by which the bond's *goal* must be achieved for the bond to pay
  out.  Format: `YYYY-MM-DD`.  After this date, oracles begin gathering
  evidence.  The bond remains ACTIVE until *expiry*.

DLEQ
: Discrete Logarithm Equality proof.  The cryptographic primitive used to
  lock XMR and ZEC *collateral* via adaptor signatures.  The oracle
  pre-commits to two adaptor secrets: `s_met` (goal met) and `s_fail`
  (goal not met).  Publishing one completes the sweep in the corresponding
  direction.  For Bitcoin, the analogous construction is a PTLC (Point
  Time-Locked Contract) on secp256k1 Taproot.  For ETH, the equivalent
  function is performed by the NyxForge Escrow smart contract.

evidence
: A file (PDF, JSON, video, or HTTP-JSON feed snapshot) attached to an
  *attestation* as a BLOB inside the `.bond` SQLite archive.  Its SHA-256
  hash is signed alongside the attestation result.

expiry
: The date after which, if *quorum* has not been reached with result `met`,
  the oracle publishes `s_fail` and the issuer may reclaim *collateral* via
  `bond reclaim`.  Must be after the *deadline*.  The gap between deadline
  and expiry is the *settlement window*.  If the oracle fails to publish
  `s_fail`, the *grace period* timelock provides a trustless fallback.

grace period
: A number of days added to *expiry* (`expiry + grace_days`) after which an
  on-chain *timelock* matures and the issuer can reclaim *collateral* without
  any oracle involvement.  For DLEQ/PTLC chains (XMR, ZEC, BTC) this is a
  transaction-level timelock; for ETH it is enforced by the Escrow contract's
  `timelockReclaim()` function.  Protects against oracle incapacitation, death,
  or unwillingness to publish the settlement credential.  Set by the issuer at
  bond creation.  Default: 30 days.  Under normal operation the oracle
  publishes the credential before the grace period elapses.

goal
: The real-world outcome a bond is designed to incentivise.  May be
  quantitative (evaluated against a *criterion*), qualitative (evaluated
  by oracle judgment), or hybrid (both).

goal type
: One of `quantitative`, `qualitative`, or `hybrid`.  Determines whether
  the bond requires a *criterion*, oracle judgment, or both.

holder
: The current owner of a `.bond` file.  Ownership is determined by possession
  of the file and the ability to decrypt the *scalar* stored within it.  The
  scalar is encrypted to the holder's public key and is re-encrypted on
  *transfer*.  Security note: if an attacker obtains both the `.bond` file
  and the holder's private key, the bond is redeemable by the attacker.
  Private keys must be stored in a passphrase-protected keystore or hardware
  wallet, separate from the `.bond` file itself.

issuer
: The party that creates and funds a bond.  The issuer locks *collateral*
  and defines the *goal*, *oracle panel*, *deadline*, *expiry*, *quorum*,
  and *challenge period*.

listing record
: A signed JSON document produced by `bond prove` that allows a *holder* to
  prove ownership of a `.bond` file to an exchange without transferring the
  file or the *scalar*.  Contains the bond summary, `holder_pubkey`, ask
  price, list expiry, exchange nonce, and a signature over those fields made
  with the holder's private key.  The exchange verifies the signature and
  publishes the listing; the actual transfer happens peer-to-peer between
  holder and buyer.

operator
: The comparison applied between the observed data value and the *threshold*
  in a quantitative *criterion*.  One of: `eq`, `gte`, `lte`, `gt`, `lt`.

oracle
: An independent party that monitors real-world data, evaluates whether a
  bond's *goal* was met, and submits a signed *attestation*.  Each oracle
  has a 32-byte public key, a role (`quantitative`, `qualitative`, or
  `both`), and a fee per attestation denominated in the bond's collateral
  currency.

oracle panel
: The set of oracles authorised to attest on a specific bond.  Defined by
  the issuer at creation time.  Each panel member must explicitly accept
  the bond's goal spec before the bond can be issued.

quorum
: The minimum number of agreeing *attestations* required to finalise a bond
  result.  Set by the issuer.  Must be <= the number of oracles in the
  *oracle panel*.

REDEEMABLE
: Bond state reached when *quorum* attestations agreeing on `met` are
  recorded and the *challenge period* has passed.  The holder may then
  run `bond redeem` to claim the *collateral* using the oracle's settlement
  credential.

s_fail
: The oracle's settlement credential for the goal-not-met outcome.
  For DLEQ/PTLC bonds (XMR, ZEC, BTC): a scalar that completes the
  adaptor transaction returning *collateral* to the issuer.  For ETH:
  an EIP-712 `Reclaim(address issuer)` signature authorising the Escrow
  contract to release funds to the issuer.  Published after *expiry* if
  the goal was not met.

s_met
: The oracle's settlement credential for the goal-met outcome.
  For DLEQ/PTLC bonds (XMR, ZEC, BTC): a scalar published when *quorum*
  is reached; the holder combines it with the adaptor to complete the
  sweep.  For ETH: an EIP-712 `Release(address holder)` signature the
  holder uses to call `NyxForgeEscrow.release(sig)`.  Published when
  *quorum* is reached with result `met`.

series
: A set of identical `.bond` files sharing a `series_id`, each with a
  unique serial number (`NNNN`).  All units in a series have the same
  goal, collateral, oracle panel, and terms.  Individual units can be
  transferred or redeemed independently.

settlement window
: The time between *deadline* and *expiry*.  Oracles use this window to
  gather evidence, submit *attestations*, and allow the *challenge period*
  to pass before the bond is finalised.

social policy bond (SPB)
: A financial instrument that pays out only if a specified real-world
  outcome is achieved by a given date.  Holders profit by buying low
  (outcome uncertain) and redeeming high (outcome confirmed).  The market
  price reflects the crowd's probability estimate of the outcome occurring.
  Originated by Ronnie Horesh.

term
: One measurable condition within a bond.  A bond has one or more terms.
  Each term has a `goal_type` (quantitative, qualitative, or hybrid), a
  human-readable `criterion`, and -- for quantitative/hybrid terms -- a
  `data_id`, `operator`, and `threshold`.  The bond pays out when the
  *term aggregation* condition is satisfied across all terms.

term aggregation
: The logical operator that combines multiple *term* results into a single
  bond outcome.  `AND` (default): all terms must be met.  `OR`: any one
  term must be met.  Null (omitted) when a bond has only one term.  The
  bond transitions to REDEEMABLE when quorum of oracles attests the
  aggregation condition as satisfied.

threshold
: The numeric target value in a quantitative *criterion*.  The observed
  `data_id` value is compared against the threshold using the *operator*.
  Example: threshold `50000` with operator `lte` means the goal is met
  when the measured value is 50,000 or below.

timelock
: An on-chain mechanism that prevents *collateral* from being moved until a
  specified date.  In NyxForge, the timelock matures at `expiry + grace_days`.
  For DLEQ/PTLC chains (XMR, ZEC, BTC): enforced at the transaction level.
  For ETH: enforced by the `timelockReclaim()` function in the NyxForge Escrow
  contract.  If no oracle publishes a settlement credential before that date,
  the issuer can recover collateral without oracle involvement.  Implemented
  at Phase 4/5.

lock mechanism
: The on-chain method used to hold and release a bond's *collateral*.
  Determined by the collateral currency and immutable after issuance.
  `dleq_xmr` (Monero DLEQ adaptor, Curve25519), `dleq_zec_sapling`
  (Zcash Sapling DLEQ adaptor, Jubjub), `ptlc_btc` (Bitcoin PTLC,
  secp256k1 Taproot), `eth_escrow` (Ethereum smart contract).

PTLC
: Point Time-Locked Contract.  The Bitcoin equivalent of a DLEQ adaptor
  signature.  Uses BIP-340 Schnorr signatures on secp256k1 Taproot outputs.
  The oracle pre-commits to scalars `s_met` and `s_fail`; publishing one
  completes the Taproot spend in the corresponding direction.  More private
  than a hash-based HTLC because the adaptor point is not visible on-chain.

XMR
: Monero.  The default *collateral* currency for NyxForge bonds.  Chosen
  for on-chain privacy and the availability of *DLEQ* adaptor signatures
  compatible with the bond redemption mechanism.  All four supported
  currencies (XMR, ZEC, BTC, ETH) use different locking mechanisms; see
  *lock mechanism*.
