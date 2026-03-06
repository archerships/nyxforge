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

### AI-assisted exploration (recommended starting point)

```bash
nyxforge-cli bond explore
```

Requires `ANTHROPIC_API_KEY` to be set in your environment.

1. Describe your goal in plain English:
   ```
   Describe the social or environmental goal you'd like to fund:
   > I want to reduce unsheltered homelessness in the US to under 100,000 people by 2030
   ```

2. The AI searches existing bonds on the network for overlapping goals and
   explains the similarities.  If a close match is found you can choose to
   **back that bond** instead of creating a new one — concentrating capital
   and market signal on a single instrument.

3. The AI drafts a new bond specification with suggested values for every
   field (data ID, operator, threshold, deadline, evidence format, etc.) and
   flags any measurement ambiguities as notes.

4. You choose:
   - **Create bond from AI draft** — launches the wizard pre-filled with the
     AI's suggestions; edit any field before submitting.
   - **Back existing bond [N]** — shows full bond details and instructions
     for finding it on the market.
   - **Start fresh** — launches the wizard with no pre-fills.
   - **Cancel**

Example AI output:
```
── Analysis ──────────────────────────────────────────
Two bonds target US homelessness reduction using HUD PIT data, but neither
specifies unsheltered-only. Creating a new bond with the unsheltered sub-metric
would provide a more focused instrument and avoid double-counting.

── Similar existing bonds ────────────────────────────
[1] US Homelessness Reduction 2030  (high)
    ID: a1b2c3d4…
    Targets total PIT count < 50,000. Your goal differs: unsheltered-only
    and a higher threshold (100,000).

── AI-drafted bond ───────────────────────────────────
Title:    US Unsheltered Homelessness Below 100k by 2030
Goal:     Annual HUD Point-in-Time unsheltered count falls below 100,000 …
Data ID:  us.hud.pit_count.unsheltered   lt 100000
Aggreg.:  annual_point_in_time
Deadline: 2030-01-01

Note: Confirm whether "unsheltered" should include those in cars/encampments.
      HUD publishes this breakdown annually each January.
```

---

### Proposal workflow (recommended)

Before locking collateral, you can publish your bond as a proposal and invite
community feedback.  This helps catch ambiguous goal definitions, wrong data
IDs, or unclear measurement methodology before the bond goes live.

```
bond propose  →  community reviews and comments  →  bond create
```

**Step 1 — Publish a proposal**

```bash
nyxforge-cli bond propose
```

Same wizard as `bond create`.  The bond is stored in `Proposed` state — no
collateral is locked and nothing is tradeable yet.  A bond ID is returned.

**Step 2 — Community posts questions and suggestions**

Any node participant can comment on the proposal:

```bash
nyxforge-cli bond comment <BOND-ID>
```

You will be prompted to enter your question or suggestion.  Comments are
attributed to your DRK public key.

**Step 3 — Review the feedback**

```bash
nyxforge-cli bond comments <BOND-ID>
```

Prints all comments in chronological order, showing the commenter's key
prefix, timestamp, and body text.

**Step 4 — Submit for oracle approval**

When the goal spec is clear and the community is satisfied, submit it to the
listed oracles for acceptance:

```bash
nyxforge-cli bond submit <BOND-ID>
```

The bond moves to `PendingOracleApproval`.  Each oracle listed in the bond's
`OracleSpec` must explicitly accept before the bond can go live.

**Step 5 — Oracles accept or reject**

Each oracle operator runs:

```bash
nyxforge-cli bond oracle-accept <BOND-ID>   # accept
nyxforge-cli bond oracle-reject <BOND-ID>   # decline with reason
```

Check the current status at any time:

```bash
nyxforge-cli bond oracle-status <BOND-ID>
```

Output:
```
Bond state: PendingOracleApproval

03a1b2c3…  ✔ accepted   2030-01-15T12:00:00Z
04d5e6f7…  ✘ rejected   2030-01-15T13:00:00Z
  Reason: data_id 'us.hud.pit_count' is ambiguous — specify sheltered vs unsheltered
08a9b0c1…  … pending
```

**If an oracle rejects:** You must either replace the oracle or revise the
goal spec to address the rejection reason.  Use `bond revise-oracles` to
replace the oracle list — this clears all existing responses and oracles must
re-accept from scratch:

```bash
nyxforge-cli bond revise-oracles <BOND-ID>
```

You will be prompted to enter a new comma-separated list of oracle keys.

**Step 6 — Issue the bond**

Once all oracles have accepted (bond state = `Draft`), lock collateral and
go live:

```bash
nyxforge-cli bond issue <BOND-ID>
```

---

### Skipping the community proposal step

```bash
nyxforge-cli bond create
```

Runs the wizard and submits directly to oracle approval — skipping the
community comment stage.  Oracle approval is still required.  Once all
oracles accept, run `bond issue <BOND-ID>` to lock collateral and go live.

The wizard walks you through every field in order.  Press **Enter** to accept
a default.  Press **Ctrl-C** at any point to cancel without submitting.

---

### Goal

**Title** — A short human-readable name for the goal.
Example: `US Homelessness Reduction 2030`

**Description** — A longer explanation of what the bond is trying to achieve.

**Data ID** — A dot-separated string that identifies which data series the oracle
should fetch and measure.  Oracle nodes register data adapters keyed to specific
Data IDs; when a bond is being evaluated, the oracle finds the adapter that
matches this ID, fetches the current value, and applies the operator and
threshold to determine whether the goal has been met.  If no registered oracle
has an adapter for a given Data ID the bond cannot be verified, so confirm
that your chosen oracle nodes support the ID before issuing.

The naming convention is:

```
<country-or-org>.<agency>.<metric>[.<variant>]
```

| Data ID | Source | Measures |
|---|---|---|
| `us.hud.pit_count` | HUD Point-in-Time survey | Total US homeless count |
| `us.hud.pit_count.unsheltered` | HUD PIT survey | Unsheltered homeless count |
| `noaa.co2.monthly_mean_ppm` | NOAA Mauna Loa Observatory | Atmospheric CO₂ (ppm) |
| `who.malaria.deaths_per_100k` | WHO Global Health Observatory | Malaria deaths per 100k population |

You can define any Data ID agreed upon with your oracle operators.  New data
sources are added by writing a `DataSource` adapter in the oracle node (see
[Running an Oracle](#9-running-an-oracle)).

**Operator** — The comparison used to evaluate whether the goal has been met.

The oracle measures the real-world value and compares it against the
threshold using this operator:

| Choice | Symbol | Goal is met when… |
|---|---|---|
| `lt` | `<` | measured value is **less than** threshold |
| `lte` | `<=` | measured value is **less than or equal to** threshold |
| `gt` | `>` | measured value is **greater than** threshold |
| `gte` | `>=` | measured value is **greater than or equal to** threshold |
| `eq` | `==` | measured value **equals** threshold exactly |

**Example:** A bond targeting a reduction in homelessness would use `lt`
(less than) with a threshold of `50000`.  The goal is met when the HUD
count drops below 50,000.  A bond targeting growth in renewable energy
capacity would use `gte` (greater than or equal to) with the target
gigawatt figure.

**Threshold** — The numeric target value (decimal).
Example: `50000`

**Aggregation** *(optional)* — How the data source should be aggregated
before the comparison is applied.
Example: `annual_mean`, `annual_point_in_time`.  Leave blank to use the
raw value.

**Evidence format** *(optional)* — Expected format of the oracle's evidence
attachment.  Leave blank if not required.

**Deadline** — The date by which the goal must be achieved.  Format: `YYYY-MM-DD`.
If the goal is not met by this date the bond expires and holders receive nothing.
Example: `2030-01-01`

---

### Economics

**Total supply** — Number of individual bond units to issue.  All units are
identical and interchangeable.

**Redemption value** — Amount of DRK paid per bond unit when the goal is met
and the bond is redeemed.  Enter whole DRK units.

**Floor price** — Minimum sale price per unit on the secondary market.
Enter whole DRK units.

---

### Oracle Network

**Quorum** — Minimum number of independent oracle attestations required before
the result is accepted.  Default: `3`.  A higher quorum increases security but
may slow verification.

**Oracle public keys** — Comma-separated hex public keys of the oracle nodes
authorised to attest on this bond.  If you leave this blank, your wallet's DRK
key is used as the sole oracle (suitable for testing).

**Required oracle stake** — Minimum DRK each oracle must have staked.  Oracles
that attest fraudulently lose this stake.  Default: `100` DRK.

**Slash fraction** — Fraction of staked DRK slashed from a fraudulent oracle.
Range: `0.0` – `1.0`.  Default: `0.5` (50% slash).

---

### Verification

**Attestation threshold** — Number of matching attestations needed from the
quorum to finalise the result.  Must be ≤ quorum.  Default: equal to quorum
(unanimous).

**Challenge period** — Seconds during which the oracle result can be disputed
before it is finalised on-chain.  Default: `86400` (24 hours).

**DAO override** — Whether the DAO governance contract can override the oracle
consensus result.  Default: `No`.

---

### Preview and submit

After all prompts the wizard prints the complete bond as JSON for review:

```
--- Bond preview ---
{
  "id": "a1b2c3...",
  "goal": { "title": "US Homelessness Reduction 2030", ... },
  ...
}

Submit this bond? [y/N]
```

Enter `y` to submit.  On success the node prints the bond ID:

```
✔  Bond issued: a1b2c3d4e5f6...
```

On cancellation or error, no bond is created and no collateral is locked.

---

### Goal specification examples

**Housing**
```
data_id:     us.hud.pit_count.unsheltered
operator:    lte
threshold:   50000
aggregation: annual_point_in_time
deadline:    2030-01-01
```

**Environmental**
```
data_id:     noaa.co2.monthly_mean_ppm
operator:    lt
threshold:   350.0
aggregation: annual_mean
deadline:    2045-01-01
```

**Public health**
```
data_id:     who.malaria.deaths_per_100k
operator:    lt
threshold:   1.0
aggregation: global_annual
deadline:    2035-01-01
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

| State | Meaning |
|---|---|
| `Proposed` | Published for community review; open for questions and suggestions |
| `PendingOracleApproval` | Submitted to oracles; waiting for each listed oracle to accept |
| `Draft` | All oracles accepted; ready for collateral lock |
| `Active` | Collateral locked; bonds circulating on market |
| `Redeemable` | Goal met; holders can redeem |
| `Expired` | Deadline passed, goal not met; worthless |
| `Settled` | All notes redeemed |

### Inspect a single bond

```bash
nyxforge-cli bond get <BOND-ID>
```

Prints the full bond JSON including goal spec, oracle configuration, and
current state.

---

## 7. Trading Bonds

Bond trading is available through the web UI at `http://localhost:8080`.

- **Buy** — place a bid order specifying the bond ID, quantity, and maximum
  price per unit (in DRK).
- **Sell** — place an ask order specifying the bond ID, quantity, and minimum
  price per unit.

All trades are anonymous.  Your wallet keys never leave your device.  Each
trade generates a ZK transfer proof so no on-chain record links buyer to seller.

Bond price on the secondary market reflects the market's estimate of the
probability that the goal will be achieved before the deadline:

```
price ≈ redemption_value × P(goal met before deadline)
```

As the deadline approaches and the metric improves or worsens, the price
adjusts.  This price signal is itself useful social information.

---

## 8. Redeeming Bonds

When a bond reaches `Redeemable` state:

1. Open the web UI and navigate to **My Bonds**.
2. Select the bond and click **Redeem**.
3. The UI generates a ZK burn proof proving you own a bond note and that the
   goal was verified.
4. The node verifies the proof and issues a DRK payout note to your wallet.
5. The payout note can be spent like any other DRK.

Redemption is anonymous.  The payout note is a fresh anonymous DRK note with
no on-chain link to the bond note that was burned.

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
