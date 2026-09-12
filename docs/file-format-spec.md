# NyxForge File Format Specification
> Version: 1.0 (2026-04-25)
> Status: DRAFT -- supercedes informal format notes in 00_MVP.md and wallet-design-spec.md
> Scope: All persistent data files in the NyxForge application suite

---

## 1. Overview

NyxForge uses six distinct file types. Each maps to a single responsibility.
No file type is a superset of another.

Table.1.FileTypes

| Extension     | Name              | Stores                                   | Format         | Scope |
| :---          | :---              | :---                                     | :---           | :---  |
| `.bounty`     | Bearer Instrument | Bounty spec, collateral, evidence, verifier | SQLite         | MVP   |
| `.nfkey`      | Key File          | One encrypted Ed25519 / secp256k1 keypair | NyxEnvelope    | MVP   |
| `.nyx-wallet` | Bounty Wallet     | Index of .bounty files + user metadata   | NyxEnvelope    | MVP   |
| `.crypto-wallet` | Crypto Wallet  | Root seed + per-coin derived keys        | NyxEnvelope    | MVP   |
| `.nyx-judge`  | Judge File        | Judge identity, claim queue, verdicts    | NyxEnvelope    | MVP   |
| `.nyx-node`   | Node File         | Peer table, DEX listings, trade history  | NyxEnvelope    | v2+   |

Design principles:
- The `.bounty` file IS the financial instrument. Losing it is equivalent to
  burning a banknote. All other files are operational support.
- Private keys MUST NOT appear in `.bounty` files. They belong in `.nfkey`.
- NyxEnvelope (Section 2) is the encryption shell for all non-.bounty files.
  It is not applied to `.bounty` because SQLite + SQLCipher handles that case.
- Every file type carries an `alg_epoch` field enabling in-place PQ migration
  when post-quantum algorithms are ready (see doc/03_RESEARCH/quantum-resistance.md).

---

## 2. NyxEnvelope -- Common Encryption Format

All file types except `.bounty` share a common binary envelope. The envelope
applies the encryption standards recommended in doc/research/wallet-formats-2026.md:
Argon2id KDF, XChaCha20-Poly1305 AEAD, and CBOR payload encoding.

### 2.1 Motivation

Proprietary per-format encryption leads to inconsistent KDF parameters,
non-interoperable backup tools, and per-format audit burden. A single envelope
means one KDF implementation, one test vector suite, one upgrade path, and
consistent security properties across the entire application.

### 2.2 Binary Layout

All integers are little-endian. Total fixed header size: 80 bytes.

```
Offset  Size  Field
------  ----  -----
0       6     magic: ASCII "NYXENV"
6       1     file_type_code (see Table.2.TypeCodes)
7       1     format_version: 0x01
8       1     alg_epoch: 0x00 (classical) | 0x01 (PQ transition) | 0x02 (full PQ)
9       1     kdf_type: 0x01 = Argon2id
10      4     kdf_m: u32, Argon2id memory parameter in KiB (default 65536 = 64 MiB)
14      1     kdf_t: u8, Argon2id iterations (default 3)
15      1     kdf_p: u8, Argon2id parallelism / lanes (default 4)
16      32    salt: random bytes for Argon2id
48      24    nonce: random bytes for XChaCha20-Poly1305 (24-byte extended nonce)
72      8     payload_len: u64, byte length of the encrypted payload that follows
80+     N     encrypted payload: XChaCha20-Poly1305 ciphertext
80+N    16    Poly1305 authentication tag (appended after ciphertext by the AEAD)
```

Table.2.TypeCodes

| Code | Extension       |
| :--- | :---            |
| 0x01 | (reserved, .bounty uses SQLite) |
| 0x02 | .nyx-wallet     |
| 0x03 | .crypto-wallet  |
| 0x04 | .nfkey          |
| 0x05 | .nyx-judge      |
| 0x06 | .nyx-node       |

### 2.3 Key Derivation

```
key = Argon2id(
    password = user_passphrase (UTF-8),
    salt     = header[16..48],
    m        = header[10..14] as u32,
    t        = header[14],
    p        = header[15],
    out_len  = 32
)
```

The 32-byte output is used directly as the XChaCha20-Poly1305 encryption key.
No additional key stretching is applied.

Associated data passed to the AEAD (not encrypted, authenticates the header):
```
ad = header[0..80]   (the entire fixed header, including salt and nonce)
```

This binds the ciphertext to the header, preventing header-swapping attacks.

### 2.4 Payload Encoding

The decrypted payload is CBOR (RFC 8949). All NyxEnvelope implementations
MUST:
- Use deterministic CBOR encoding (map keys sorted by length then lexicographic).
- Use text strings (major type 3) for all human-readable string fields.
- Use byte strings (major type 2) for all binary fields (keys, hashes, signatures).
- Include a `file_type` text field as the first map entry to allow format
  identification after decryption.
- Include a `version` integer field as the second map entry for schema migration.

### 2.5 Default KDF Parameters

```
kdf_m: 65536  (64 MiB; matches OWASP recommendation for interactive logins)
kdf_t: 3
kdf_p: 4
```

These defaults are stored in the header. Implementations MUST read parameters
from the file header on open rather than assuming defaults. This allows
parameters to be upgraded on file save without breaking backwards compatibility.

### 2.6 PQ Migration Path

When alg_epoch advances to 0x01 (PQ transition), the format adds an optional
second encrypted payload block following the classical block:

```
[NyxEnvelope v1 header][classical ciphertext][pq_nonce: 32 bytes][pq_ciphertext]
```

The PQ block uses ML-KEM-768 + ML-DSA-65 (NIST FIPS 203/204). The classical
block is retained for backwards compatibility during the transition period.
At alg_epoch 0x02 (full PQ), the classical block is dropped.

This is a forward-declaration. Do not implement until PQ algorithms are
standardized in the Rust ecosystem (post-MVP).

---

## 3. .bounty -- Bearer Instrument

### 3.1 Summary

The `.bounty` file is specified in full in `doc/00_MVP.md Section 4`. This
section provides a format-level summary for consistency with the other types.

Format: SQLite 3 database. For bounties requiring evidence privacy, the entire
file is encrypted with SQLCipher (AES-256-CBC; see rusqlite + sqlcipher feature).

File is NOT wrapped in NyxEnvelope. SQLite provides its own robust binary
container and SQLCipher handles encryption. Wrapping would add overhead with
no benefit.

### 3.2 Filename Convention

```
<series_id_prefix_8chars>-<serial_4digits>.bounty
e.g.  a1b2c3d4-0001.bounty
```

### 3.3 Core Tables (abbreviated)

| Table          | Key contents                                              |
| :---           | :---                                                      |
| `bounty_spec`  | bounty_id (Blake3), title, deadline, expiry, series_id    |
| `terms`        | per-term goal criteria, operator, threshold               |
| `collateral`   | currency, amount, lock_mechanism, holder_pubkey, adaptor  |
| `judges`       | pubkey, role, fee, accepted_at                            |
| `evidence`     | content_type, storage_kind, blob or uri, sha256, submitted_by |
| `attestations` | judge_pubkey, result, signature, pq_signature             |
| `history`      | state transitions with timestamps                         |
| `verifier`     | WASM bytecode of the self-verifying redemption circuit    |

Full schema: `doc/00_MVP.md Section 4.1`.

### 3.4 Key isolation requirement

`collateral.holder_pubkey` contains the holder's curve public key.
`collateral.scalar_encrypted` contains s_met encrypted to that public key.
The holder's private key MUST be stored in a `.nfkey` file, never in `.bounty`.

---

## 4. .nfkey -- Key File

### 4.1 Purpose

Stores a single encrypted asymmetric keypair. One file per key. Keypairs for
different purposes (holder key, judge key, issuer key) are kept in separate
`.nfkey` files.

Default storage location: `~/.nyx/keys/`

### 4.2 Envelope

NyxEnvelope with file_type_code 0x04.

### 4.3 CBOR Payload Schema

```
{
  "file_type":    text,     // "nfkey"
  "version":      uint,     // 1
  "label":        text,     // human-readable name, e.g. "judge-key-2026"
  "usage":        text,     // "bounty" | "judge" | "wallet" | "general"
  "key_algorithm": text,    // "ed25519" | "secp256k1" | "jubjub"
  "created_at":   text,     // ISO8601 datetime
  "fingerprint":  bytes,    // first 20 bytes of Blake3(public_key)
  "public_key":   bytes,    // raw public key (32 bytes for ed25519)
  "private_key":  bytes,    // raw private scalar (32 bytes for ed25519)
  "alg_epoch":    uint,     // 0 = classical; 1 = PQ transition
  "pq_public_key":  bytes?, // null until v3 PQ migration
  "pq_private_key": bytes?  // null until v3 PQ migration
}
```

### 4.4 Fingerprint Display Format

The fingerprint is displayed as colon-separated uppercase hex pairs:
```
7A:B2:C4:D9:11:FF:...:88:FC   (20 bytes = 20 pairs)
```

### 4.5 CLI commands (updated from 00_MVP.md)

The `.nyx` extension in CLI output is replaced by `.nfkey` throughout:

```
nyx key generate --label <name> --passphrase <p> [--out <file>.nfkey]
nyx key list     [--usage <bounty|judge|wallet|any>]
nyx key import   --format <pem|nfkey|raw> --file <path>
nyx key export   --fingerprint <XX:XX> --format <pem|nfkey|qr>
nyx key rotate   --old-fingerprint <XX:XX> --passphrase <p>
nyx key sign     --fingerprint <XX:XX> --data <hex>
nyx key verify   --fingerprint <XX:XX> --data <hex> --sig <hex>
```

Config key `keystore.dir` points to the directory containing `.nfkey` files.
Default: `~/.nyx/keys/`

---

## 5. .nyx-wallet -- Bounty Wallet

### 5.1 Purpose

An index of `.bounty` files the user holds, plus user-applied metadata (labels,
tags). The `.nyx-wallet` file does NOT contain the `.bounty` files themselves --
those are bearer instruments that live on the filesystem independently.

The distinction matters: a `.nyx-wallet` can be deleted and reconstructed by
re-scanning a directory of `.bounty` files. The `.bounty` files cannot be
reconstructed from the wallet.

### 5.2 Envelope

NyxEnvelope with file_type_code 0x02.

### 5.3 CBOR Payload Schema

```
{
  "file_type":   text,      // "nyx-wallet"
  "version":     uint,      // 1
  "wallet_id":   bytes,     // 16 random bytes (stable local identifier)
  "label":       text,      // display name, e.g. "My Bounty Portfolio"
  "created_at":  text,      // ISO8601
  "bounty_dir":  text,      // default directory to scan for .bounty files
  "entries": [
    {
      "bounty_id":      text,    // Blake3 hex (from bounty_spec.bounty_id)
      "file_path":      text,    // absolute path to the .bounty file
      "label":          text?,   // user-applied display name (nullable)
      "tags":           [text],  // user-applied tags
      "added_at":       text,    // ISO8601
      "key_fingerprint": text?   // fingerprint of the .nfkey controlling this bounty
    }
  ],
  "settings": {
    "default_sort":  text,   // "deadline" | "value" | "state" | "added"
    "show_settled":  bool,
    "show_expired":  bool,
    "warn_expiry_days": uint // warn when deadline < N days away (default 30)
  },
  "alg_epoch": uint
}
```

### 5.4 Consistency model

The wallet index is advisory. On open, the application SHOULD:
1. Verify each `file_path` exists and the `.bounty` file is readable.
2. Verify `bounty_id` in the entry matches `bounty_spec.bounty_id` in the file.
3. Flag any entry whose file has moved or is missing rather than silently
   dropping it.

On `nyx wallet scan --dir <path>`, the application adds any `.bounty` files
found in the directory that are not already in the index.

---

## 6. .crypto-wallet -- Cryptocurrency Wallet

### 6.1 Purpose

Stores the root secret and per-coin derived keys for the native NyxForge
cryptocurrency wallet (XMR, NYX, TARI, BTC, ZEC, ETH). This is the wallet
holding uncommitted collateral before it is locked into a `.bounty`.

The crypto wallet is NOT a replacement for the Monero daemon's internal wallet
format (`.keys` + cache file). It is the NyxForge application's sovereign view
of the user's keys, from which coin-specific wallet files can be re-derived.

### 6.2 Envelope

NyxEnvelope with file_type_code 0x03.

### 6.3 CBOR Payload Schema

```
{
  "file_type":   text,      // "crypto-wallet"
  "version":     uint,      // 1
  "wallet_id":   bytes,     // 16 random bytes
  "label":       text,      // display name, e.g. "Primary Collateral Wallet"
  "created_at":  text,      // ISO8601
  "root_type":   text,      // "polyseed" | "bip39" | "monero_legacy"
  "root_secret": bytes,     // entropy bytes of the root mnemonic (25 words for
                            // monero_legacy; 16 bytes for polyseed; BIP-39 per spec)
                            // Protected by the NyxEnvelope passphrase.
  "coins": [
    {
      "currency":           text,    // "xmr" | "btc" | "eth" | "zec" | "tari" | "nyx"
      "derivation_path":    text?,   // BIP-44 path for BTC/ETH; null for XMR/polyseed
      "output_descriptor":  text?,   // BIP-380 descriptor for BTC; null for others
      "rpc_endpoint":       text?,   // per-coin RPC override; null = use global config
      "address_labels": {
        // map of address (text) -> label (text)
      },
      "last_synced":        text?    // ISO8601 of last successful sync
    }
  ],
  "mining": {
    "enabled":        bool,
    "coins":          [text],    // ["xmr", "nyx", "tari"]
    "thread_count":   uint?,     // null = auto
    "pool_url":       text?      // null = solo mining
  },
  "alg_epoch": uint
}
```

### 6.4 Root secret types

| root_type        | root_secret content                                  |
| :---             | :---                                                 |
| `polyseed`       | 16 bytes of Polyseed entropy (Monero-native, v2 mnemonic) |
| `bip39`          | BIP-39 entropy bytes (12 or 24 words)                |
| `monero_legacy`  | 32-byte spend key scalar (Monero 25-word mnemonic)   |

All three can derive the XMR spend/view keypair. `polyseed` is the recommended
default for new wallets (compact, supports network embedding, deterministic
wallet birthday).

### 6.5 Tx cache

The transaction cache for each coin (full tx history, UTXO set) is NOT stored
in the `.crypto-wallet` file. It is stored in a companion cache directory
(`~/.nyx/cache/<currency>/`) that can be deleted and rebuilt by rescanning the
blockchain. Only stable identity material lives in the envelope.

---

## 7. .nyx-judge -- Judge File

### 7.1 Purpose

Stores the operational state of a NyxForge judge: their signing key reference,
specialty and fee configuration, the queue of claims awaiting their review, and
a signed verdict history.

A judge does not need to run a node. The `.nyx-judge` file is self-contained for
offline claim review and signing workflows.

### 7.2 Envelope

NyxEnvelope with file_type_code 0x05.

### 7.3 CBOR Payload Schema

```
{
  "file_type":    text,     // "nyx-judge"
  "version":      uint,     // 1
  "judge_id":     bytes,    // 16 random bytes (stable local identifier)
  "key_fingerprint": text,  // fingerprint of the .nfkey used for signing verdicts
  "display_name": text?,    // optional public label (pseudonym or org name)
  "specialties":  [text],   // subset of canonical set:
                            // ["longevity", "actuarial", "legal", "medical", "financial"]
  "fee_schedule": [
    {
      "currency":           text,    // "xmr" | "btc" | "eth" | "zec" | "tari" | "nyx"
      "fee_per_decision":   text,    // decimal string (e.g. "0.05")
      "minimum_collateral": text?    // minimum bounty collateral to accept assignment
    }
  ],
  "registered_series": [
    {
      "series_id":    text,   // bounty series identifier
      "bounty_id":    text,   // specific bounty within series
      "role":         text,   // "primary" | "backup" | "panel"
      "accepted_at":  text,   // ISO8601
      "status":       text    // "PENDING_ACCEPT" | "ACTIVE" | "SETTLED" | "EXPIRED"
    }
  ],
  "claim_queue": [
    {
      "claim_id":       text,    // CLM-XXXX display ID
      "bounty_id":      text,    // Blake3 hex
      "claim_type":     text,    // "maturity" | "partial" | "default" | "expired"
      "filed_at":       text,    // ISO8601
      "status":         text,    // FILED | EVIDENCE | REVIEW | DECIDED | APPEAL
                                 // | WITHDRAWN | APPROVED | DENIED
      "evidence_hashes": [text], // SHA-256 hex of each attached evidence object
      "claimant_pubkey": text,   // public key of the filing holder
      "notes":          text?,   // judge's private working notes (not shared)
      "my_verdict":     text?,   // "approve" | "deny" | null if undecided
      "my_verdict_at":  text?    // ISO8601 of verdict decision
    }
  ],
  "verdict_history": [
    {
      "claim_id":    text,    // CLM-XXXX
      "bounty_id":   text,    // Blake3 hex
      "verdict":     text,    // "approve" | "deny"
      "decided_at":  text,    // ISO8601
      "note":        text?,   // public verdict note (shared with claimant)
      "signature":   bytes    // Ed25519 sig over Blake3(bounty_id || verdict
                              //   || evidence_hashes_concat || decided_at)
    }
  ],
  "created_at": text,
  "alg_epoch":  uint
}
```

### 7.4 Verdict signature

The `verdict_history[*].signature` field commits the judge irrevocably to their
decision. The signed payload is:

```
Blake3(bounty_id_bytes || verdict_bytes || evidence_sha256_concat || decided_at_bytes)
```

where `evidence_sha256_concat` is the raw bytes of all evidence SHA-256 hashes
concatenated in the order they appear in `claim_queue[*].evidence_hashes`.

This signature is one judge's verdict input. When enough judge signatures satisfy
the configured quorum, the holder can submit the quorum set to `nyx bounty redeem`.

---

## 8. .nyx-node -- Node File (v2+)

### 8.1 Purpose

Stores the persistent state of a NyxForge P2P node: peer table, active DEX
listings, trade history, and node configuration.

This file type is v2+ (P2P network and DEX are deferred per `doc/00_MVP.md
Section 3`). It is specified here for completeness so that file format design
is complete before implementation begins.

### 8.2 Envelope

NyxEnvelope with file_type_code 0x06.

### 8.3 CBOR Payload Schema

```
{
  "file_type":  text,       // "nyx-node"
  "version":    uint,       // 1
  "node_id":    bytes,      // 32 random bytes (local P2P identity; not a signing key)
  "peer_list": [
    {
      "peer_id":    text,       // libp2p PeerId (base58)
      "addresses":  [text],     // multiaddrs (e.g. "/onion3/.../8333")
      "last_seen":  text,       // ISO8601
      "reputation": uint        // 0-100 local score; not shared
    }
  ],
  "active_listings": [
    {
      "listing_id":      text,   // local UUID
      "bounty_id":       text,   // Blake3 hex
      "holder_pubkey":   text,   // hex
      "ask":             text,   // decimal, in the bounty's collateral currency
      "currency":        text,   // collateral currency of the listed bounty
      "list_expiry":     text,   // ISO8601
      "nonce":           bytes,  // exchange-issued anti-replay nonce
      "signature":       bytes,  // holder signature over listing payload (see 00_MVP.md 7g)
      "created_at":      text,
      "status":          text    // "OPEN" | "MATCHED" | "EXPIRED" | "CANCELLED"
    }
  ],
  "trade_history": [
    {
      "trade_id":           text,   // local UUID
      "bounty_id":          text,   // Blake3 hex
      "counterparty_pubkey": text,  // hex
      "direction":          text,   // "bought" | "sold"
      "price":              text,   // decimal in collateral currency
      "settled_at":         text    // ISO8601
    }
  ],
  "dex_config": {
    "fee_tier":        text,      // "standard" | "maker"
    "tor_proxy":       text?,     // "socks5://127.0.0.1:9050"
    "bootstrap_peers": [text],    // initial multiaddrs for peer discovery
    "listing_ttl_hours": uint     // how long to broadcast a listing (default 48)
  },
  "created_at": text,
  "alg_epoch":  uint
}
```

---

## 9. Default File Locations

```
~/.nyx/
  keys/           <- .nfkey files
  wallets/        <- .nyx-wallet and .crypto-wallet files
  judges/         <- .nyx-judge files
  nodes/          <- .nyx-node files
  bounties/       <- default directory for .bounty files
  cache/          <- tx caches (coin-specific subdirs; purgeable)
    xmr/
    btc/
  config.toml     <- application config (not a NyxEnvelope file)
```

---

## 10. Compatibility and Migration

### 10.1 Version bumping

When a breaking schema change is required, `version` in the CBOR payload is
incremented. The application reads `version` after decryption and dispatches to
the appropriate migration function. Old versions are upgraded in place on save.

### 10.2 KDF parameter upgrade

If the application's minimum acceptable KDF parameters exceed those stored in a
file header (e.g. `kdf_m` is below the new minimum), the application prompts for
the passphrase on open, re-derives with the new parameters, and re-encrypts the
file. No data loss; no manual user action beyond re-entering the passphrase.

### 10.3 Cross-format references

Files reference each other by content identifier, not by path, to survive moves:

| Reference                                | Identifier used           |
| :---                                     | :---                      |
| .nyx-wallet -> .bounty                   | `bounty_id` (Blake3 hex)  |
| .nyx-wallet -> .nfkey                    | key `fingerprint` (hex)   |
| .nyx-judge  -> .nfkey                    | key `fingerprint` (hex)   |
| .nyx-judge  -> .bounty (via claim_queue) | `bounty_id` (Blake3 hex)  |
| .nyx-node   -> .bounty (via listings)    | `bounty_id` (Blake3 hex)  |

### 10.4 Interoperability with industry standards

Per doc/research/wallet-formats-2026.md recommendations:

- `.crypto-wallet` stores an Output Descriptor (`output_descriptor` field) for
  BTC coins, enabling reconstruction in any BIP-380-compliant wallet.
- The `root_secret` in `.crypto-wallet` is a standard BIP-39 or Polyseed
  entropy blob, not a NyxForge-proprietary format, so keys can be recovered
  in any compatible wallet without NyxForge tooling.
- `.nfkey` supports PEM export (`nyx key export --format pem`) for use with
  standard OpenSSL tooling.

---

## 11. Security Properties Summary

Table.3.Security

| Property                      | Mechanism                                           |
| :---                          | :---                                                |
| Passphrase hardening          | Argon2id (m=65536, t=3, p=4) -- GPU/ASIC resistant |
| Encryption                    | XChaCha20-Poly1305 -- authenticated, misuse-resistant |
| Header integrity              | AEAD associated data covers full 80-byte header    |
| Key isolation                 | Private keys live only in .nfkey; never in .bounty |
| Bearer security               | .bounty loss = instrument loss; no server recovery |
| PQ forward compatibility      | alg_epoch in all files; migration path in Section 2.6 |
| Nonce reuse resistance        | 24-byte XChaCha20 nonce -- collision probability negligible |
| Side-channel resistance       | s_met scalar exists in RAM only during redeem/transfer |
