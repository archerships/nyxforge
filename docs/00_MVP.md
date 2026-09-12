# NyxForge — MVP Specification
> Version: 1.0 (April 2026)
> Status: CURRENT BUILD TARGET
> Supersedes: 00_CORE.md and 02_PLAN.md for the immediate build
> Next version: 00_CORE.md (ZK notes, order book, P2P network) when MVP is live

---

## 1. What We Are Building

A minimal, working social policy bounty instrument that proves the Horesh incentive
loop end-to-end with no custom blockchain, no ZK ownership circuits, and no
trusted custodian.

One sentence: a .bounty file is a self-contained bearer instrument backed by
crypto collateral (XMR, ZEC, BTC, TARI, NYX, or ETH) locked via DLEQ/PTLC adaptor
signatures or smart contract escrow, verifiable and redeemable by anyone who
holds the file and its secret scalar (or judge-signed credential for ETH).

---

## 2. Design Decisions

| Decision | Choice | Rationale |
| :--- | :--- | :--- |
| Ownership model | Bearer file (possession = ownership) | Eliminates P2P network, ZK notes, nullifiers |
| Collateral currency | XMR, ZEC (shielded Sapling), BTC, ETH, TARI, NYX | Multi-currency from genesis; XMR is default reference implementation; NYX supports testing and early supporter rewards |
| Payout mechanism | DLEQ/PTLC adaptor sigs (XMR/ZEC/BTC/TARI); ETH smart contract escrow; NYX ledger escrow | Judge cannot steal; no custodian; trustless for all MVP currencies |
| Judge type | Human panel for qualitative; HTTP-JSON for quantitative | Matches goal type; domain experts, not developers |
| Collateral unit | Fixed per file (e.g. 1 XMR) | Enables price comparison; natural fractional ownership |
| File format | SQLite (.bounty) + NyxEnvelope (all other files) | 200-year archival; Library of Congress standard; see doc/05_TECH/file-format-spec.md |
| Trading | Bilateral, off-chain (file + scalar over Tor/Signal) | No order book needed for MVP |
| Developer revenue | Judge fees + issuance fee in reference client; NYX may be used for testnet and early supporter rewards | No VC, no equity, no corporate entity required |
| Bootstrap funding | Grants (Gitcoin, Octant, Monero CCS) + development bounty | No VC; no equity; no corporate entity required |

---

## 3. What Is Explicitly Out of Scope (MVP)

These are valid v2+ goals. Do not let them consume design energy until MVP is live.

| Item | Deferred to |
| :--- | :--- |
| ZK MINT/TRANSFER/BURN ownership circuits | v2 |
| libp2p P2P network | v2 |
| DarkFi L1 integration | v3 |
| Order book / DEX | v2 |
| Browser UI (Flutter) | v2 |
| Nullifier set / shared state | v2 |
| Merkle membership proof | v2 |
| Judge slashing | v2 |
| Goal text encryption | v2 |
| Multi-language docs | post-launch |
| Post-quantum verifier slots (full PQ) | v3 |
| Bounties with deadline > 10 years | post-PQ (blocked on CRQC timeline) |
| ZEC Orchard (Pallas/Vesta) locking mechanism | v3 (Sapling is MVP target) |

The existing nyxforge-zk, nyxforge-contract, nyxforge-node crates are preserved
but not on the MVP critical path. Do not modify them during MVP development.

---

## 4. The .bounty File Schema

Every bounty is a single SQLite file. The file IS the bounty -- it contains the goal,
the collateral details, the evidence, and the verification logic.

### 4.1 Tables

```sql
-- Core identity and goal
CREATE TABLE bounty_spec (
    bounty_id       TEXT PRIMARY KEY,  -- Blake3(alg_epoch || currency || goal_hash || collateral_hash || judge_hash || inception)
                                       -- alg_epoch=0x00 classical; 0x01 when PQ algorithms deploy
    schema_version  INTEGER NOT NULL,  -- bump on breaking changes
    alg_epoch       INTEGER NOT NULL DEFAULT 0,  -- 0=classical(ed25519); 1=PQ transition; 2=full PQ
    title           TEXT NOT NULL,
    description     TEXT NOT NULL,
    goal_type        TEXT NOT NULL,     -- derived from terms: 'quantitative' | 'qualitative' | 'hybrid'
    term_aggregation TEXT,              -- 'AND' | 'OR'; null when bounty has exactly one term

    -- Timing
    inception       TEXT NOT NULL,     -- ISO8601 datetime
    deadline        TEXT NOT NULL,     -- goal must be met by this date; enforced <= inception + 10 years
    expiry          TEXT NOT NULL,     -- hard kill: s_fail released after this date

    -- Unit identity (unique per file within a series)
    series_id        TEXT NOT NULL,     -- shared across all files in one issuance
    serial           INTEGER NOT NULL,  -- unique within series
    redemption_value TEXT NOT NULL,     -- amount per file on redemption, in the bounty's collateral currency

    created_at      TEXT NOT NULL
);

-- Deadline constraint note:
-- deadline MUST be <= inception + 10 years. Enforced by the CLI wizard.
-- Rationale: bounties with longer horizons enter the cryptographically relevant
-- quantum computer (CRQC) threat window. See doc/03_RESEARCH/quantum-resistance.md.

-- Bounty terms (one row per term; a bounty has 1..N terms)
-- The bounty pays out when the term_aggregation condition is satisfied across all terms.
CREATE TABLE terms (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    seq             INTEGER NOT NULL,    -- 1-based display order
    goal_type       TEXT NOT NULL,       -- 'quantitative' | 'qualitative' | 'hybrid'
    criterion       TEXT NOT NULL,       -- human-readable description of the condition
    data_id         TEXT,                -- machine-readable source (e.g. 'usgov.cbo.federal_outlays_pct_gdp')
    operator        TEXT,                -- 'lt' | 'lte' | 'gt' | 'gte' | 'eq'
    threshold       REAL,
    aggregation     TEXT                 -- e.g. 'annual_mean' | 'annual_point_in_time'
);

-- Collateral locking details
CREATE TABLE collateral (
    -- Currency and amount
    currency          TEXT NOT NULL,    -- 'xmr' | 'zec' | 'btc' | 'eth' | 'tari' | 'nyx'
    amount            TEXT NOT NULL,    -- decimal string; units are in the named currency (XMR/ZEC/BTC/ETH/TARI/NYX)
    redemption_value  TEXT,             -- informational fiat peg at issuance (e.g. '1000 USD'); not enforced

    -- Locking mechanism (one per series; derived from currency at wizard time)
    lock_mechanism    TEXT NOT NULL,    -- 'dleq_xmr' | 'dleq_zec_sapling' | 'ptlc_btc' | 'ptlc_tari' | 'eth_escrow' | 'nyx_escrow'
    lock_txid         TEXT,             -- funding transaction ID (all chains)
    lock_address      TEXT,             -- DLEQ/PTLC: output address; ETH: escrow contract address
    chain_id          INTEGER,          -- ETH only: 1=mainnet 11155111=Sepolia 17000=Holesky; null for XMR/ZEC/BTC

    -- DLEQ/PTLC fields (xmr, zec, btc, tari) -- null for eth_escrow/nyx_escrow
    t_met             TEXT,             -- commitment T_met = s_met * G (hex); published at judge acceptance
    t_fail            TEXT,             -- commitment T_fail = s_fail * G (hex); published at judge acceptance
    adaptor           BLOB,             -- pre-signed adaptor tx; holder completes with s_met
    holder_pubkey     TEXT,             -- current holder's curve pubkey; updated on transfer
    scalar_encrypted  BLOB,             -- holder's s_met decryption scalar, encrypted to holder_pubkey

    -- ETH escrow fields (eth_escrow only) -- null for dleq_*/ptlc_*/nyx_escrow
    judge_eth_pubkey  TEXT,             -- ETH address of the judge's per-bounty keypair

    -- Algorithm identifiers (for PQ migration)
    key_algorithm     TEXT NOT NULL DEFAULT 'ed25519',               -- 'ed25519' (XMR/NYX) | 'jubjub' (ZEC) | 'secp256k1' (BTC/ETH) | 'ristretto' (TARI)
    enc_algorithm     TEXT NOT NULL DEFAULT 'x25519-chacha20poly1305',
    alg_epoch         INTEGER NOT NULL DEFAULT 0,

    -- PQ forward-compatibility slots (null until v3 migration)
    pq_holder_pubkey  TEXT,
    pq_scalar_enc     BLOB
);

-- Key management note:
-- The holder's private key MUST be stored outside the .bounty file -- in a
-- passphrase-protected keystore or hardware wallet. If the private key and
-- the .bounty file are both accessible to an attacker, the bounty is redeemable
-- by the attacker. The threat model is identical to a software crypto wallet.

-- Judge panel
CREATE TABLE judges (
    pubkey          TEXT PRIMARY KEY,
    role            TEXT NOT NULL,     -- 'quantitative' | 'qualitative' | 'both'
    fee             TEXT NOT NULL,     -- fee per attestation event; units are the bounty's collateral currency
    accepted_at     TEXT               -- timestamp of judge acceptance
);

CREATE TABLE judge_config (
    quorum          INTEGER NOT NULL,  -- minimum signatures to settle
    challenge_days  INTEGER NOT NULL   -- days to challenge an attestation
);

-- Evidence submitted by judges, claimant, holder, or issuer
CREATE TABLE evidence (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type    TEXT NOT NULL,     -- 'application/pdf' | 'video/mp4' | 'application/json'
    description     TEXT NOT NULL,
    storage_kind    TEXT NOT NULL,     -- 'embedded' | 'ipfs' | 'arweave' | 'url'
    blob            BLOB,              -- required when storage_kind='embedded'
    uri             TEXT,              -- ipfs://CID, ar://TXID, https://..., or null for embedded
    sha256          TEXT NOT NULL,     -- hex hash of embedded blob or referenced bytes
    submitted_by    TEXT NOT NULL,     -- judge pubkey, claimant pubkey, holder pubkey, or 'issuer'
    submitted_at    TEXT NOT NULL
);

-- Judge attestations
CREATE TABLE attestations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    judge_pubkey    TEXT NOT NULL,
    result          TEXT NOT NULL,     -- 'met' | 'not_met'
    evidence_id     INTEGER REFERENCES evidence(id),
    sig_algorithm   TEXT NOT NULL DEFAULT 'ed25519',
    signature       BLOB NOT NULL,     -- judge signs (bounty_id || result || evidence_sha256)
    pq_signature    BLOB,              -- null until judges support PQ signing; hybrid attestation
    signed_at       TEXT NOT NULL
);

-- State history
CREATE TABLE history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    state           TEXT NOT NULL,     -- see Section 5
    transitioned_at TEXT NOT NULL,
    note            TEXT
);

-- Self-verifying logic (one-to-many: multiple verifiers can coexist across algorithm epochs)
CREATE TABLE verifier (
    circuit_id      TEXT NOT NULL,
    sig_algorithm   TEXT NOT NULL DEFAULT 'ed25519',  -- algorithm used to sign this verifier blob
    wasm_blob       BLOB NOT NULL,     -- WASM bytecode of the verifier
    version         TEXT NOT NULL,
    PRIMARY KEY (circuit_id, sig_algorithm)
);
```

### 4.2 File naming convention

```
<series_id_prefix_8chars>-<serial_4digits>.bounty
e.g. a1b2c3d4-0001.bounty
```

The file extension is `.bounty` in all user-facing contexts. The internal schema
uses current names (`bounty_spec`, `bounty_id`, `judges`, `judge_config`).

### 4.2a Display ID

`bounty_id` is a full Blake3 hash. For display, the CLI and UI render a short
human-readable ID derived from the first 6 bytes of the hash:

```
NYX-XXXX-XXXX-XXXX   (4 + 4 + 4 uppercase hex chars from bytes [0:2], [2:4], [4:6])
e.g.  NYX-7F3A-C20B-E941
```

The full `bounty_id` is always available via `nyx bounty inspect --json`.

### 4.3 Encryption

For bounties requiring evidence privacy: SQLCipher (AES-256) encrypts the entire
file. Only holders of the BountyViewKey can read embedded evidence BLOBs and
private evidence references. The bounty_id and
series_id are visible in the filename; all other content is encrypted at rest.

### 4.4 Collateral Locking Mechanism by Currency

`lock_mechanism` determines how collateral is locked and how the judge
credential completes the payout. One mechanism per series; immutable after
issuance.

Table.2.LockMech

| Currency | lock_mechanism | Judge credential | Holder action | key_algorithm |
| :--- | :--- | :--- | :--- | :--- |
| XMR | dleq_xmr | s_met scalar (Curve25519) | adaptor + s_met -> Monero sweep tx | ed25519 |
| ZEC shielded | dleq_zec_sapling | s_met scalar (Jubjub) | adaptor + s_met -> ZEC Sapling sweep tx | jubjub |
| BTC | ptlc_btc | s_met scalar (secp256k1) | adaptor + s_met -> Taproot sweep tx (BIP-340) | secp256k1 |
| TARI | ptlc_tari | s_met scalar (Ristretto) | adaptor + s_met -> Tari sweep tx | ristretto |
| ETH | eth_escrow | EIP-712 Release(holder) sig | call NyxForgeEscrow.release(sig) | secp256k1 |
| NYX | nyx_escrow | judge quorum credential | release from NYX reward/test ledger | ed25519 |

#### DLEQ/PTLC path (XMR, ZEC, BTC, TARI)

At judge acceptance the judge generates two random scalars and publishes
their public commitments to the .bounty file:

```
t_met  = s_met  * G   (stored in collateral.t_met)
t_fail = s_fail * G   (stored in collateral.t_fail)
```

The adaptor transaction is pre-signed so that only the holder of s_met can
complete the sweep to the holder address, and only the holder of s_fail can
sweep to the issuer return address. No custodian can move funds without a
scalar. The construction is analogous across supported adaptor-signature curves.

Note: ZEC Orchard (Pallas/Vesta curves) is deferred to v3. Sapling (Jubjub)
is the ZEC MVP target. BTC requires Taproot outputs (BIP-341/342); legacy
P2PKH/P2SH outputs are not supported.

#### ETH escrow path

A NyxForge Escrow contract is deployed to the target chain at `nyx bounty issue`
time. The judge generates a fresh secp256k1 keypair per bounty; the contract
stores the judge's Ethereum address as the sole authorized releaser.

```
On goal met:   judge signs EIP-712 Release(address holder)
               holder calls NyxForgeEscrow.release(sig)

On goal fail:  judge signs EIP-712 Reclaim(address issuer)
               issuer calls NyxForgeEscrow.reclaim(sig)

Timelock:      after expiry + grace_days, issuer may call
               NyxForgeEscrow.timelockReclaim() with no judge involvement
```

Supported chain_id values:

| chain_id | Network |
| :--- | :--- |
| 1 | Ethereum mainnet |
| 11155111 | Sepolia testnet |
| 17000 | Holesky testnet |

The NyxForge Escrow contract source is in `crates/nyxforge-contract/`.
ETH escrow implementation is Phase 5 (parallel to XMR DLEQ).

---

## 5. Bounty Lifecycle

```
DRAFT
  │  issuer calls: nyx bounty issue
  │  locks collateral on-chain, registers judges
  ▼
ACTIVE
  │  judges monitor; evidence accumulates
  │  bilateral trading: file + scalar exchanged off-chain
  │
  ├── [quorum of attestations: result = met]
  │         ▼
  │     REDEEMABLE
  │         │  holder calls: nyx bounty redeem
  │         │  DLEQ/PTLC: adaptor + s_met → sweep tx
  │         │  ETH: judge_sig → contract.release(holder)
  │         ▼
  │     SETTLED
  │
  └── [expiry date passes, goal not met]
            ▼
        EXPIRED
            │  issuer calls: nyx bounty reclaim
            │  DLEQ/PTLC: judge publishes s_fail → issuer sweeps
            │  ETH: judge_sig → contract.reclaim(issuer)
            │  Fallback: on-chain timelock at expiry + grace_days
            ▼
        RECLAIMED
```

States:
- DRAFT: file created, collateral not yet locked
- ACTIVE: collateral locked, judges registered, trading open
- REDEEMABLE: judge quorum attested goal met; s_met available to holder
- SETTLED: collateral swept by holder
- EXPIRED: deadline passed, goal not met, awaiting reclaim
- RECLAIMED: collateral returned to issuer

---

## 6. Judge Protocol

### 6.1 Judge types

Qualitative judge (human panel):
- Reviews embedded or externally referenced evidence submitted to the .bounty file
- Signs attestation: `sign(bounty_id || "met" || evidence_sha256)`
- Used for: alive/healthy/sentient checks, subjective social conditions

Quantitative judge (automated):
- Fetches data from registered data_id source via HTTP-JSON adapter
- Evaluates: `fetched_value OPERATOR threshold`
- Signs attestation with result
- Used for: CO2 levels, poverty rates, mortality statistics

### 6.2 Judge registration

Before a bounty goes ACTIVE:
1. Issuer selects judge panel (pubkeys + roles + fees)
2. Each judge reviews the GoalSpec and accepts or rejects
3. All judges must accept before issuer locks collateral
4. Judge fee (in the bounty's collateral currency) is paid at issuance time, not at attestation

### 6.3 Settlement

When attestation quorum is reached:
1. Judge(s) publish the settlement credential:
   - DLEQ/PTLC (XMR, ZEC, BTC): judge publishes s_met scalar
   - ETH escrow: judge signs EIP-712 `Release(address holder)` message
2. Holder completes the payout:
   - DLEQ/PTLC: combines adaptor + s_met to complete and broadcast the sweep transaction
   - ETH: calls `NyxForgeEscrow.release(judge_sig)` on the target chain
3. Holder updates .bounty file state to SETTLED (optional; file is now spent)

### 6.4 Judge separation from development

Judge operators are domain experts in the goal being measured -- not developers.
Developers build and maintain tooling. Judge operators sign attestations.
Neither depends on the other's revenue. This minimises legal exposure for both.

---

## 7. CLI Commands

The NyxForge unified CLI is invoked as `nyx`. Subcommand groups map directly to
UI mockup sections (see Section 7c for terminology alignment).

```
# Bounty management
nyx bounty create  --subject <name>  \    # --subject is displayed name (maps to bounty_spec.title)
                   --collateral <amt> --denom <XMR|ZEC|BTC|ETH|TARI|NYX> \
                   --deadline <YYYY-MM-DD> --expiry <YYYY-MM-DD> \
                   --grace-days <n> --judge <pubkey> \
                   --out <file>.bounty
nyx bounty inspect <file>.bounty          # dump metadata + verify DLEQ proof
nyx bounty status  <file>.bounty          # current state, attestation count, deadline
nyx bounty list    [--dir <path>]         # list all .bounty files in a directory
nyx bounty verify  <file>.bounty          # run WASM verifier against bundled evidence
nyx bounty transfer <file>.bounty --to <recipient_pubkey>   # re-encrypt scalar
nyx bounty redeem  <file>.bounty          # construct sweep using adaptor + s_met
nyx bounty reclaim <file>.bounty          # issuer sweeps collateral after expiry (s_fail)
nyx bounty prove   <file>.bounty --nonce <hex> --ask <amt> --list-expiry <YYYY-MM-DD>
                                          # produce a signed listing record

# Claim filing (claimant / holder side)
nyx claim file     --bounty <NYX-XXXX> --type <claim_type> --narrative <text>
nyx claim attach   --claim <CLM-XXXX> --file <path> --desc <text>
nyx claim list                            # list claims filed by the current key
nyx claim status   --claim <CLM-XXXX>     # detailed status + judge messages
nyx claim withdraw --claim <CLM-XXXX>     # cancel before verdict (requires WITHDRAW confirmation)

# Judge workflow
nyx judge list-claims  [--status <STATUS>] [--my-key]
nyx judge review  <CLM-XXXX>              # inspect claim + evidence
nyx judge request-info <CLM-XXXX> --msg <text>   # ask claimant for more evidence
nyx judge decide  <CLM-XXXX> --verdict <approve|deny> --note <text>
nyx judge appeal  <CLM-XXXX>              # escalate to panel (3-of-5)
nyx judge history                         # past verdicts signed by this key
nyx judge register [--dry-run]            # register as a judge on-chain

# Judge registry (v2+ — MVP uses direct pubkey selection in bounty-wizard)
# Valid specialties: longevity | actuarial | legal | medical | financial
# Valid roles: primary | backup | panel
# nyx judge registry --fee is denominated in the series' collateral currency
nyx judge registry list     [--specialty <spec>] [--sort <rating|decisions>]
nyx judge registry profile  --id <JDG-XXXX>
nyx judge registry register --key <fingerprint> --specialty <spec> --fee <amount>
nyx judge registry invite   --judge <JDG-XXXX> --series <SER-XXXX> --role <primary|backup|panel>
nyx judge registry rate     --id <JDG-XXXX> --claim <CLM-XXXX> --score <1-5>

# DEX (v2 — order book not in MVP scope; mockup is preview)
nyx dex list | take | create | cancel | trades | pairs | price

# Wallet
nyx wallet balance [--coin <xmr|nyx|tari>]
nyx wallet receive --coin <xmr|nyx|tari>
nyx wallet send    --coin <xmr|nyx|tari> --to <addr> --amount <n>
nyx wallet history [--coin <c>] [--limit <n>]
nyx wallet mine    --coins <xmr,nyx,tari> [--threads <n>]
nyx wallet mine stop
nyx wallet export  --out <file> --passphrase <p>

# Fiat-to-XMR swap (v2 — Haveno/Retoswap style; mockup is preview)
nyx swap list | take | confirm-sent | dispute | status | create | history

# NGO and tax back-office (v2 companion product)
nyx ngo   status | donations | convert | treasury | receipt | processor | config
nyx tax   status | generate | receipts | receipt | schedule-b | export | ledger

# Key management
nyx key generate --label <name> --passphrase <p> [--out <file>.nfkey]
nyx key list   [--usage <bounty|judge|wallet|any>]
nyx key import --format <pem|nfkey|raw> --file <path>
nyx key export --fingerprint <XX:XX> --format <pem|nfkey|qr>
nyx key rotate --old-fingerprint <XX:XX> --passphrase <p>

# Node management
nyx node status | peers | sync | health

# Configuration
nyx config get [--all | <key>]
nyx config set <key> <value>
nyx config reset [<key> | --all --dry-run]
nyx config profile list | create | use | delete
```

### 7a Deadline constraint

`deadline` MUST be <= `inception + 10 years`. The CLI wizard enforces this.
Rationale: bounties with longer horizons enter the CRQC quantum threat window.
Bounties with deadline > 10 years are deferred to the post-PQ release.

### 7b Claim types (canonical set)

The `--type` flag for `nyx claim file` accepts the following values:

| type | Description |
|------|-------------|
| `maturity` | All terms met; collateral payout due |
| `partial` | Terms partially met; reduced payout requested |
| `default` | Issuer failed to lock collateral or breached terms |
| `expired` | Bounty expired without settlement; collateral release requested |

### 7c Terminology alignment

NyxForge uses "bounty" and "judge" everywhere in prose, UI, CLI output, and
all new documentation. User-facing CLI flags also use bounty terminology
(`--bounty`, never `--bounty`). Internal SQL schema names are current:
`bounty_spec`, `bounty_id`, `judges`, and `judge_config`. Legacy Rust types and
RPC names may remain until a dedicated code-rename PR:

| Context | Legacy (internal only) | Current (use everywhere) |
|---------|------------------------|--------------------------|
| Rust types | Bounty, BountyState, OracleAccept | -- (code rename deferred) |
| RPC methods | bounties.get, oracle.attest | bounties.get, judge.attest (code rename deferred) |
| All prose | bounty, .bounty file | bounty, .bounty file |
| All prose | oracle, oracle panel | judge, judge panel |
| CLI display | DRAFT state | displayed as "UNISSUED" in output |

The `--subject` flag in `nyx bounty create` maps to `bounty_spec.title`. It is the
human-readable name of the bounty, typically the name of the person or goal at the
centre of a longevity or social-outcome bounty.

### 7d Claim filing workflow

A claim is a formal request by the current holder (bearer) of a .bounty file
that the judge panel evaluate whether the bounty terms have been met and release
the collateral. Claims are filed against ACTIVE bounties (before expiry).

1. Holder calls `nyx claim file` with the bounty ID, claim type, and narrative.
   The CLI verifies the holder's bearer key matches `collateral.holder_pubkey`.
2. Holder attaches evidence via `nyx claim attach`. Evidence can be embedded as
   BLOBs in the .bounty file or stored as content-addressed external references
   (`ipfs://...`, `ar://...`, or HTTPS). In all cases the .bounty file stores
   SHA-256 hashes so judges verify the referenced bytes before relying on them.
3. The bounty shows an operational status of CLAIMED (claim under review) in the
   holder UI. This is not a core lifecycle state; the bounty remains ACTIVE.
4. Each judge on the panel reviews the claim via `nyx judge review` and can
   request additional info via `nyx judge request-info`.
5. When quorum of judges have decided, the bounty transitions:
   - APPROVE verdict by quorum -> judge quorum reached -> REDEEMABLE
   - DENY verdict by quorum -> claim dismissed; bounty remains ACTIVE
6. Holder can withdraw a pending claim via `nyx claim withdraw` (before any judge votes).

Claim status values (uppercase in all outputs and filters):

| Status | Meaning |
|--------|---------|
| FILED | Claim submitted, not yet assigned |
| EVIDENCE | Judge requested additional evidence |
| REVIEW | Evidence complete; under review |
| DECIDED | Verdict rendered |
| APPEAL | Escalated to appeal panel (3-of-5) |
| WITHDRAWN | Claimant withdrew before verdict |
| APPROVED | Quorum approved payout |
| DENIED | Quorum denied; bounty remains active |

### 7e Collateral currencies — MVP vs v2+

MVP (current build target): XMR, ZEC (Sapling), BTC (Taproot), ETH, TARI, NYX.
NYX is included in MVP to support testing, reward accounting, and early
supporter incentives. TARI and NYX wallet/mining surfaces are therefore active
MVP surfaces, not preview-only UI.

Judge fees in MVP are denominated in the bounty's collateral currency
(e.g. XMR, TARI, or NYX). NYX-denominated fees are allowed when the bounty's
collateral currency is NYX.

### 7f Products in scope for MVP vs v2+

| Feature | Scope |
|---------|-------|
| .bounty creation, inspect, transfer, redeem | MVP |
| Judge accept, attest, verdict | MVP |
| Claim filing workflow (nyx claim) | MVP |
| Key management (nyx key) | MVP |
| XMR/TARI/NYX wallet + mining (nyx wallet) | MVP |
| Node management (nyx node) | MVP |
| Configuration (nyx config) | MVP |
| DEX order book (nyx dex) | v2 preview |
| Fiat-to-XMR P2P swap (nyx swap) | v2 preview |
| NGO donation processing (nyx ngo) | v2 companion |
| IRS tax reporting (nyx tax) | v2 companion |
| Reverse Dutch auction (nyx auction) | v2 preview |
| Judge registry (nyx judge registry) | v2 (MVP uses direct pubkey selection) |
| NYX token emission | MVP |
| TARI merge mining | MVP |

---

## 7g. Listing Record Schema

A listing record is a JSON document produced by `nyx bounty prove`.  It allows a
holder to prove ownership to an exchange and share bounty details without
transferring the scalar or the private key.

```json
{
  "bounty_id":     "<blake3 hex>",
  "holder_pubkey": "<curve pubkey hex>",
  "ask":           1.5,
  "list_expiry":   "2026-06-01",
  "nonce":         "<exchange-provided hex nonce>",
  "bounty_summary": {
    "title":       "...",
    "state":       "active",
    "amount":      1.0,
    "currency":    "xmr",
    "deadline":    "2030-01-01",
    "expiry":      "2030-02-01",
    "series_id":   "...",
    "serial":      1
  },
  "sig": "<holder signs blake3(bounty_id || nonce || ask || list_expiry) with holder_privkey>",
}
```

Verification steps (exchange side):

1. Verify `sig` against `holder_pubkey` over the signed payload.
2. Verify `holder_pubkey` matches `collateral.holder_pubkey` in the `.bounty` file
   (exchange requests the file with `scalar_encrypted` field redacted).
3. Verify `bounty_id` in the listing matches `bounty_spec.bounty_id` in the file.
4. Verify `nonce` matches the one the exchange issued (prevents replay).
5. Verify `list_expiry` is in the future.

The exchange never receives `scalar_encrypted` or the holder's private key.
Actual transfer (file + re-encrypted scalar to buyer) happens directly
between holder and buyer after a match is made.

### Design note: why not a Schnorr ZK proof of knowledge?

A Schnorr PoK was considered as a more private alternative to a plain
signature.  It was rejected for MVP for the following reason:

In this context the two schemes are privacy-equivalent.  The exchange must
verify the proof against `collateral.holder_pubkey` from the bounty file, so
it learns `holder_pubkey` regardless of which scheme is used.  A Schnorr PoK
proves knowledge of the discrete log of a public key -- but if the verifier
already sees that public key to check against, the proof reveals exactly the
same information a plain Schnorr signature does.

The real privacy concern -- an exchange linking multiple listings to the same
holder by recognising the same `holder_pubkey` -- is not solved by a fancier
proof scheme.  It is solved by key derivation: generating a fresh keypair per
listing (BIP-32 style) and updating `holder_pubkey` on every transfer.  That
is a v2 feature and is orthogonal to the proof scheme chosen here.

A Schnorr PoK would offer a genuine advantage only if the bounty file stored a
cryptographic commitment to the holder key rather than the key itself, allowing
ownership proofs that do not reveal which key is being used.  That requires
the ZK ownership circuit infrastructure deferred to v2 (see Section 3).
Do not reopen this decision for MVP.

---

## 8. Implementation Phases

Development follows a UI-first sequence. The Flutter web frontend (with
Rust/WASM backend logic) and its mock API are built before any cryptographic
backend exists. This validates UX early, forces the RPC contract to be defined
before it is implemented, and produces demo-able progress from week one.

The primary development artifact is a static web bundle: Flutter UI compiled
to web + `nyxforge_web.wasm` Rust module. Desktop and mobile shells are
downstream packaging targets built after the browser app stabilizes.

Toolchain:
- Excalidraw  -- user flow sketches (open source, self-hosted or excalidraw.com)
- Penpot      -- screen designs (open source Figma alternative, penpot.app)
- Mockoon     -- fake Rust node returning bounty JSON-RPC responses (mockoon.com)
- Widgetbook  -- optional Flutter widget isolation (widgetbook.io); secondary to browser target
- wasm-pack   -- compiles `nyxforge-core` Rust crate to WASM (`nyxforge_web.wasm`)
- Rust        -- real backend, built last, replaces Mockoon

### Phase 0 -- User flows (Excalidraw) (1 week)

- Sketch every user journey: issuer creation wizard, judge acceptance,
  holder status/transfer/redeem, issuer reclaim, dev stagenet testing
- Annotate screen transitions and data dependencies
- Identify edge cases: expired bounty, rejected judge, quorum not reached

Deliverable: flow diagrams committed to `doc/04_STORYBOARDS/`

### Phase 1 -- Screen designs (Penpot) (1-2 weeks)

- Design all ~10 screens derived from Phase 0 flows:
  bounty list, bounty create wizard (multi-step), bounty detail/inspect,
  bounty status, transfer, redeem, judge accept/attest, judge list, settings
- Define the component library: bounty card, state badge, judge status row,
  XMR amount field, evidence attachment widget
- Export assets to `ui/assets/`

Deliverable: Penpot project file; component inventory in `ui/`

### Phase 2 -- Mock API (Mockoon) (1 week)

- Model all JSON-RPC endpoints the Flutter app will call
- Return realistic fake bounty data for every lifecycle state:
  DRAFT, ACTIVE, REDEEMABLE, SETTLED, EXPIRED, RECLAIMED
- This schema becomes the binding contract for the Rust RPC layer --
  whatever Mockoon returns is what the bounty crate must implement

Key endpoints to mock:

  bounties.list           -> []{bounty_id, title, state, deadline, collateral}
  bounties.get            -> full bounty_spec + collateral + judge panel
  bounties.status         -> {state, attestations_received, quorum, days_remaining}
  judge.status            -> [{pubkey, role, accepted_at, result, signed_at}]
  judge.attest            -> {ok, attestations_received, quorum, new_state}
  dev.mock_attest         -> {ok, message, new_state}
  dev.force_state         -> {ok, message}

Deliverable: Mockoon collection file committed to `src/mock/nyxforge.mockoon.json`

### Phase 3 -- Flutter + Rust/WASM App (5-8 weeks)

- Build all screens as Flutter widgets against the Mockoon mock
- Wire up navigation, state management, and error handling
- No real XMR or cryptography at this stage; all data comes from Mockoon
- Primary target: static web bundle loaded in a browser (not macOS/Linux desktop)
- Compile `nyxforge-core` to `nyxforge_web.wasm` via `wasm-pack build`
- Establish Dart-to-WASM interop boundary: Dart calls `js_interop` or `dart:ffi`
  stubs that forward to WASM exports; mock fallback returns hardcoded results
- The app should be fully navigable in a browser against the mock

WASM implementation sequence (within Phase 3):

  a. Stub WASM module: `nyxforge-core` crate with `wasm-bindgen` exports for
     `.bounty` parse, `bounty_id` derivation, and envelope verify
  b. Dart interop layer: thin Dart wrapper calling into WASM exports via JS interop
  c. Mock fallback: if WASM not loaded, Dart returns hardcoded mock data so
     Flutter UI development is never blocked on WASM build
  d. Static bundle: `flutter build web` + `wasm-pack build --target web` ->
     single directory deployable from any static HTTP server
  e. Browser smoke tests: Playwright or flutter_test web runner; assert that
     the `.bounty` inspect/verify path executes through WASM, not the mock fallback
  f. Local RPC fallback: when the real bounty crate is available (Phase 4),
     Dart switches from WASM exports to JSON-RPC calls to the local daemon;
     WASM module stays for offline verify/inspect operations

Deliverable: complete Flutter + Rust/WASM static web bundle running against
Mockoon in a browser; `nyx bounty list` and `nyx bounty status` also working
via Mockoon

### Phase 4 -- bounty crate and CLI (6-10 weeks)

- Rust struct definitions for all .bounty tables
- SQLite read/write via rusqlite; SQLCipher for encrypted files
- DLEQ adaptor signature library (monero-rs or from spec)
- Blake3 bounty_id derivation
- HTTP-JSON quantitative judge adapter (reqwest + jsonpath)
- Quorum counting and state transition logic
- All CLI commands wired to real file operations (replacing stubs)

Deliverable: bounty crate with full test suite;
CLI create/inspect/status/judge-attest all working against real .bounty files

### Phase 5 -- Collateral and redemption (6-8 weeks)

- XMR DLEQ adaptor setup at bounty issuance
- `nyx bounty issue`: construct DLEQ commitment, broadcast XMR lock transaction
- `nyx bounty redeem`: combine adaptor + s_met → complete XMR sweep
- `nyx bounty reclaim`: judge broadcasts s_fail → issuer sweeps
- Stagenet end-to-end: issue → attest → redeem with real XMR

Deliverable: full bounty lifecycle working on Monero stagenet

### Phase 6 -- Wire Flutter + Rust/WASM App to real backend (2-3 weeks)

- Replace Mockoon calls with real bounty RPC responses
- Integration testing: full flow on stagenet from Flutter + Rust/WASM app
- Handle async XMR broadcast states in the UI (pending, confirmed)
- Switch WASM interop from mock fallback to local RPC daemon calls

Deliverable: complete Flutter + Rust/WASM app running against real backend
on stagenet

### Phase 7 -- First real bounty (2-3 weeks)

- Mainnet issuance of the NyxForge Development Bounty
  (payout if NyxForge reaches 100 XMR total collateral locked by 2028)
- Document the issuance as a worked example
- Grant applications: Gitcoin, Octant, Monero CCS

Deliverable: first live bounty on Monero mainnet; grant applications submitted

---

## 9. Funding Approach

| Timeline | Source | Mechanism |
| :--- | :--- | :--- |
| Now | Personal / grants | Gitcoin, Octant, Monero CCS, ZCash Foundation |
| Phase 5 | Development bounty | Milestone-based; investors buy if they believe in the platform |
| Post-launch | Judge fees | Per-attestation fee to judge operators |
| Post-launch | Issuance fee | Small % in reference client; funds dev team |

No token. No DAO. No corporate entity required for MVP.

---

## 10. What the Existing Code Covers

The current codebase (nyxforge-core, nyxforge-zk, nyxforge-contract, nyxforge-node)
implements the v2.0 full spec (ZK notes, P2P network). That work
is preserved and becomes the v2 build target.

For the MVP, only these crates are on the critical path:

| Crate | MVP role |
| :--- | :--- |
| bounty crate (new or renamed) | .bounty schema, DLEQ primitives |
| `nyxforge-cli` (partial reuse) | CLI commands (bounty create/inspect/transfer/redeem) |
| judge crate (new or renamed) | Judge accept/attest workflow |

All other crates: preserved, not modified during MVP development.

---

## 11. Success Criteria for MVP

- [ ] A .bounty file can be created, inspected, and transferred via CLI
- [ ] A judge panel can accept, submit evidence, and sign attestations
- [ ] Quorum of attestations transitions bounty to REDEEMABLE
- [ ] Holder can construct and broadcast XMR sweep from adaptor + s_met
- [ ] Issuer can reclaim collateral after expiry via s_fail
- [ ] End-to-end test passes on Monero stagenet
- [ ] First real bounty issued on Monero mainnet (NyxForge Development Bounty)
- [ ] At least one grant application submitted
