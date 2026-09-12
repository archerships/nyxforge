# NyxForge Plugin Architecture
**Version:** 0.1 (2026-04-25)
**Status:** CURRENT -- canonical spec for the pluggable application suite

---

## 1. Goals

The NyxForge suite must be usable in minimal form (bounty holder only, no
collateral infrastructure) and in full form (issuer + judge + wallet + DEX).
The plugin system achieves this by:

- Shipping `nyxforge-core` alone for holder use.
- Letting each additional use case be a separately installed plugin.
- Having the CLI and UI automatically surface only installed plugin features.

---

## 2. Phases

### Phase 1 -- Cargo Feature Flags (MVP)

All code ships in a single binary compiled with a set of Cargo feature flags.
Missing plugin support is caught at runtime, not compile time: the binary
always compiles; an uninstalled plugin produces a human-readable error and
exits with code 1.

This is the current build target.

### Phase 2 -- Process-per-Plugin (v2+)

Each plugin runs as a separate OS process.  The host discovers plugins by
scanning `~/.nyx/run/`.  Plugins expose a JSON-RPC 2.0 server over a Unix
domain socket at `~/.nyx/run/<plugin-id>.sock`.  The host communicates with
plugins exclusively over that socket -- no shared memory, no FFI.

Phase 2 is deferred.  The Phase 1 stub pattern (Section 5) is written so
Phase 2 can be introduced without changing call sites.

---

## 3. Plugin Registry

### 3.1 nyxforge-core (always installed)

No plugin required.  Included in every build.

| UI screen | bin | File types used |
| :--- | :--- | :--- |
| Bounty Portfolio | bounty-portfolio | .bounty, .nyx-wallet |
| Bounty Claim | bounty-claim | .bounty |
| Key Manager | key-manager | .nfkey |

CLI subcommands: `nyx bounty`, `nyx claim`, `nyx key`

### 3.2 plugin-bounty-tools

Purpose: create and manage bounty series as an issuer.
Requires: `plugin-crypto-wallet` (collateral source).

| UI screen | bin | File types used |
| :--- | :--- | :--- |
| Bounty Wizard | bounty-wizard | .bounty, .nyx-wallet, .crypto-wallet |
| Issuer Dashboard | issuer-dashboard | .bounty, .nyx-wallet |

CLI subcommands: `nyx bounty issue`, `nyx bounty series`

### 3.3 plugin-judge-tools

Purpose: judge identity, case queue, verdict signing.
Requires: `.nfkey` signing key only (no crypto-wallet dependency).

| UI screen | bin | File types used |
| :--- | :--- | :--- |
| Judge Interface | bounty-judge | .nyx-judge, .bounty |
| Judge Registry (v2+) | judge-registry | .nyx-judge |

CLI subcommands: `nyx judge`

Note: the Judge Registry screen is v2+ scope.  MVP uses direct pubkey
selection in the Bounty Wizard.

### 3.4 plugin-crypto-wallet

Purpose: crypto wallet for uncommitted collateral custody and NYX early-supporter
reward accounting.
MVP currencies: XMR, BTC, ZEC, ETH, TARI, NYX.

| UI screen | bin | File types used |
| :--- | :--- | :--- |
| NyxForge Wallet | nyx-wallet | .crypto-wallet |

CLI subcommands: `nyx wallet`

### 3.5 plugin-trading (v2+)

Purpose: secondary market, DEX, Dutch auctions, fiat on-ramp.
Requires: `plugin-crypto-wallet`, `plugin-node-tools`.

| UI screen | bin | File types used |
| :--- | :--- | :--- |
| Bounty Market | bounty-market | .bounty, .nyx-node |
| Bounty DEX | bounty-dex | .bounty, .nyx-node, .crypto-wallet |
| Bounty Auction | bounty-auction | .bounty, .nyx-node, .crypto-wallet |
| XMR Swap (Retoswap) | xmr-swap | .crypto-wallet |

CLI subcommands: `nyx dex`, `nyx auction`, `nyx swap`

### 3.6 plugin-ngo (v2+)

Purpose: nonprofit donation acceptance, auto-convert to XMR, tax reporting.
Requires: `plugin-bounty-tools`, `plugin-crypto-wallet`.

| UI screen | bin | File types used |
| :--- | :--- | :--- |
| NGO Admin | ngo-admin | .nyx-wallet, .crypto-wallet |
| Tax Reports | tax-report | .nyx-wallet |

CLI subcommands: `nyx ngo`, `nyx tax`

---

## 4. Crate Boundaries

```
nyxforge-core          -- Bounty/BountyState types, SQLite schema, claim
                          protocol, settlement logic, .bounty file I/O
nyxforge-wallet        -- XMR light wallet (MoneroSource trait), .crypto-wallet
                          and .nyx-wallet I/O; RemoteMonerod implementation
nyxforge-miner         -- RandomX CPU miner, P2Pool mini stratum client
nyxforge-judge         -- Judge identity, signing key, verdict signing,
                          .nyx-judge file I/O
nyxforge-dex (v2+)    -- Order, OrderBook, Trade types; DEX gossip protocol
nyxforge-node (v2+)   -- P2P peer table, gossip state, .nyx-node file I/O
nyxforge-rpc (v2+)    -- JSON-RPC 2.0 server shared by all Phase 2 plugins
nyxforge-cli           -- All `nyx` subcommand dispatch; feature-gated stubs
```

The Cargo workspace feature graph mirrors the plugin dependency graph:

```
nyxforge-cli
  [feature = "plugin-bounty-tools"]   --> nyxforge-core + nyxforge-wallet
  [feature = "plugin-judge-tools"]    --> nyxforge-core + nyxforge-judge
  [feature = "plugin-crypto-wallet"]  --> nyxforge-wallet + nyxforge-miner
  [feature = "plugin-trading"]        --> all of the above + nyxforge-dex
                                          + nyxforge-node
  [feature = "plugin-ngo"]            --> nyxforge-core + nyxforge-wallet
```

---

## 5. Phase 1 Stub Pattern

Every plugin-gated CLI entry point follows this pattern.  The binary always
compiles.  Missing-plugin detection is a runtime check, not a build error.

```rust
#[cfg(feature = "plugin-bounty-tools")]
pub use crate::bounty_tools::run as run_bounty_tools;

#[cfg(not(feature = "plugin-bounty-tools"))]
pub fn run_bounty_tools(_args: &ArgMatches) -> ! {
    eprintln!("error: plugin-bounty-tools is not installed");
    eprintln!("       install: nyx plugin install bounty-tools");
    std::process::exit(1);
}
```

The same pattern applies to every UI entry point: the host process attempts
to launch the plugin binary; if the binary is absent (or compiled without the
feature), it shows an "Install plugin?" prompt and offers a deep link to the
plugin's install flow.

---

## 6. Plugin Manifest (Phase 2 TOML format)

Each Phase 2 plugin ships a `plugin.toml` alongside its binary.  The host
discovers plugins by scanning `~/.nyx/plugins/*/plugin.toml`.

```toml
[plugin]
id      = "plugin-bounty-tools"
name    = "Bounty Tools"
version = "0.1.0"
scope   = "mvp"          # "mvp" | "v2+"

[plugin.requires]
nyxforge-core    = ">=0.1.0"
plugin-crypto-wallet = ">=0.1.0"

[plugin.files]
reads  = [".bounty", ".nyx-wallet", ".crypto-wallet"]
writes = [".bounty", ".nyx-wallet"]

[[plugin.ui.nav_items]]
label = "Issue"
bin   = "bounty-wizard"

[[plugin.ui.nav_items]]
label = "Dashboard"
bin   = "issuer-dashboard"

[plugin.rpc]
socket  = "~/.nyx/run/bounty-tools.sock"
methods = ["bounty.create", "bounty.issue", "bounty.list", "bounty.series"]
```

---

## 7. CLI Subcommand Surface

The `nyx` binary exposes exactly the subcommands for installed plugins.
Attempting an uninstalled subcommand prints the stub error (Section 5).

| Subcommand | Plugin required |
| :--- | :--- |
| `nyx bounty` | core |
| `nyx claim` | core |
| `nyx key` | core |
| `nyx bounty issue` | plugin-bounty-tools |
| `nyx bounty series` | plugin-bounty-tools |
| `nyx judge` | plugin-judge-tools |
| `nyx wallet` | plugin-crypto-wallet |
| `nyx dex` | plugin-trading (v2+) |
| `nyx auction` | plugin-trading (v2+) |
| `nyx swap` | plugin-trading (v2+) |
| `nyx ngo` | plugin-ngo (v2+) |
| `nyx tax` | plugin-ngo (v2+) |
| `nyx plugin` | core (plugin manager meta-command) |

---

## 8. Default File Locations

| File type | Default path |
| :--- | :--- |
| .nfkey | `~/.nyx/keys/<fingerprint>.nfkey` |
| .nyx-wallet | `~/.nyx/wallets/default.nyx-wallet` |
| .crypto-wallet | `~/.nyx/wallets/default.crypto-wallet` |
| .nyx-judge | `~/.nyx/judges/<fingerprint>.nyx-judge` |
| .nyx-node | `~/.nyx/node/state.nyx-node` |
| plugin sockets | `~/.nyx/run/<plugin-id>.sock` |
| plugin manifests | `~/.nyx/plugins/<plugin-id>/plugin.toml` |
| tx cache | `~/.nyx/cache/` (purgeable) |

---

## 9. Cross-References

- File format schemas: `doc/05_TECH/file-format-spec.md`
- UI mockup index: `doc/04_STORYBOARDS/out/index.html`
- CLI reference: `doc/05_TECH/user-manual.md`
- Current build target: `doc/00_MVP.md`
