# Tutorial: Mocking Up the NyxForge UI

This is a living document. Each section is added as the corresponding step is
completed in the development workflow.

The stack follows the UI-first build sequence defined in `doc/00_MVP.md`:

  Phase 0  Excalidraw user flows  ->  doc/04_STORYBOARDS/
  Phase 1  Penpot screen designs  ->  ui/assets/
  Phase 2  Mockoon mock API       ->  mock/nyxforge.mockoon.json
  Phase 3  Flutter + Widgetbook   ->  src/flutter/

---

## Prerequisites

| Tool | Install | Version confirmed |
| :--- | :--- | :--- |
| Node.js | system | v25.8.1 |
| mmdc (Mermaid CLI) | `npm install -g @mermaid-js/mermaid-cli` | v11.12.0 |
| Inkscape | `brew install --cask inkscape` | v1.4.3 |
| mockoon-cli | `npm install -g @mockoon/cli` | (install before Phase 2) |
| Flutter | https://flutter.dev/docs/get-started/install | (install before Phase 3) |

---

## Phase 0: User Flow Diagrams (Mermaid)

User flows live in `doc/04_STORYBOARDS/` as `.mmd` files.  They are compiled
to SVG by mmdc and committed alongside the source.

### Files

| File | Contents |
| :--- | :--- |
| `bond-lifecycle.mmd` | State machine: DRAFT -> ACTIVE -> REDEEMABLE/EXPIRED -> SETTLED/RECLAIMED |
| `bond-create-wizard.mmd` | Flowchart: bond create wizard, goal type through file write |
| `holder-flow.mmd` | Flowchart: holder actions branched by bond state |
| `oracle-flow.mmd` | Flowchart: oracle accept/reject/attest/quorum workflow |

### Viewing diagrams

Option A -- neovim (recommended): open any `.mmd` file and press `<leader>p`.
mmdc compiles it and opens the SVG in Brave.  Requires the nvim plugins
installed in `~/.config/nvim/init.lua` (lazy.nvim + diagram.nvim + image.nvim).

Option B -- HTML viewer: compile all diagrams to trimmed SVGs and open each
as a separate dark-background HTML page in Brave.

```
bin/story-view
```

Option C -- compile all to SVG manually:

```
cd prj/nyxforge
for f in doc/04_STORYBOARDS/*.mmd; do
  out="doc/04_STORYBOARDS/out/$(basename "$f" .mmd).svg"
  mmdc -i "$f" -o "$out" -b '#0d0f1a'
  inkscape "$out" --export-area-drawing --export-plain-svg --export-filename="$out" 2>/dev/null
done
```

The Inkscape pass trims the SVG canvas to the actual drawing bounds.  Mermaid
reserves extra space for back-edge routing (curved arrows that loop upward),
which can leave large empty areas on the left or right.  Inkscape's
`--export-area-drawing` removes that padding.

SVGs are written to `doc/04_STORYBOARDS/out/`.

### Editing diagrams

Edit the `.mmd` source file directly.  Mermaid syntax reference:
https://mermaid.js.org/intro/

Re-run mmdc or `<leader>p` after saving to see updated output.

### Key concepts in the flows

deadline vs expiry:

- deadline: the date by which the goal must be achieved for the bond to pay out.
- expiry: the date after which, if the goal was not met, the oracle publishes
  `s_fail` and the issuer may reclaim collateral.  Must be after the deadline.

The gap between them is the settlement window -- time for oracles to gather
evidence, attest, reach quorum, and allow the challenge period to pass.  A
bond past its deadline but before expiry is still ACTIVE.  Once expiry passes
without quorum, the bond moves to EXPIRED and collateral is returnable.

Example: deadline 2027-06-30, expiry 2027-07-31.  If the goal is met by
June 30, oracles have July to attest and reach quorum (-> REDEEMABLE).  If
not, the bond expires July 31 (-> EXPIRED) and collateral returns to the
issuer via `bond reclaim`.

Rule: expiry > deadline.  The CLI wizard enforces this at input time.

---

## Phase 2: Mock API (Mockoon)

The mock API defines the JSON-RPC contract between the Flutter UI and the
Rust backend before any Rust code is written.

### File

`mock/nyxforge.mockoon.json` -- Mockoon environment.  Single route: POST /rpc.
Responses are selected by matching the `method` field in the request body.

### Starting the mock server

```
mockoon-cli start --data mock/nyxforge.mockoon.json --port 8888
```

Or use `bin/dev-hologram` which starts it as a background process alongside
Flutter and the diagram generator.

### Endpoints

All requests are POST to `http://127.0.0.1:8888/rpc` with body:

```json
{"jsonrpc": "2.0", "id": 1, "method": "<method>", "params": {...}}
```

| Method | Returns |
| :--- | :--- |
| `bonds.list` | Array of all bonds (4 sample bonds: DRAFT, ACTIVE, REDEEMABLE, EXPIRED) |
| `bonds.get` | Full bond detail; pass `params.state` to select the sample |
| `bonds.status` | Compact status summary for a bond |
| `oracle.status` | Oracle's view of a bond (accepted, attested, fee) |
| `oracle.accept` | Mark bond accepted by this oracle |
| `oracle.reject` | Reject bond with a reason string |
| `oracle.attest` | Submit attestation (result: met or not_met) |
| `dev.mock_attest` | Dev shortcut: write a fake attestation directly |
| `dev.force_state` | Dev shortcut: jump a bond to any state |

### Generating randomized test data

```
node mock/gen_bonds.js 20 > /tmp/bonds.json
```

Produces 20 randomized bonds cycling through DRAFT/ACTIVE/REDEEMABLE/EXPIRED
with realistic oracle panels, attestations, and collateral amounts.

### All-in-one: dev-hologram

`bin/dev-hologram` ties the full pipeline together:

```
bin/dev-hologram              # diagrams + mockoon + flutter
bin/dev-hologram --no-flutter # diagrams + mockoon only (before Flutter exists)
bin/dev-hologram --no-diagrams --no-flutter  # mockoon only
```

Press Ctrl+C to stop all background processes cleanly.

---

## Phase 1: Screen Designs (Penpot)

(to be added when Penpot setup is complete)

---

## Phase 3: Flutter UI + Widgetbook

(to be added when Flutter scaffold is created)
