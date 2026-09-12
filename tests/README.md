# NyxForge CLI test suite

Black-box behavioral tests for `nyxforge-cli`: each test runs the binary in a
fresh temporary directory and asserts on exit status, stdout and `--json` output.
No network, no on-disk fixtures.

## Running

```bash
cargo build                                   # build the CLI first
export NYX_BIN=./target/debug/nyxforge-cli    # or a binary named 'nyx' on PATH
python3 -m pytest tests/ -v
# equivalently, without pytest:
python3 -m unittest discover -s tests
```

`NYX_BIN` is the only configuration the suite reads (`conftest.py` defaults to
`nyx`). Every test creates its own tempdir and cleans it up, so the suite leaves
nothing behind and is safe to run repeatedly.

## Status: this is a specification, not a green suite

Measured 2026-09-12 against a binary built from this repo at `6df115f`:

    tests: 328   passed: 137   failed: 191

The suite describes the CLI surface **as designed**, which is wider than the
crate as it stands. Two causes of failure:

1. Eleven subcommands are not implemented yet -- `auction`, `claim`, `config`,
   `dex`, `key`, `ngo`, `node`, `oracle`, `swap`, `tax`, `wallet`. Today the
   binary exposes only `bounty`, `judge` and `mcp`.
2. Argument drift inside the implemented commands: e.g. `bounty create` no
   longer accepts `--subject` (`error: unexpected argument '--subject' found`),
   which the tests still pass.

| Test file | Drives | Passed | Failed |
| :--- | :--- | ---: | ---: |
| `test_auction.py` | `auction` | 7 | 9 |
| `test_bounty.py` | `bounty` | 5 | 12 |
| `test_claim.py` | `claim` | 10 | 12 |
| `test_config.py` | `config` | 13 | 15 |
| `test_dex.py` | `dex` | 12 | 14 |
| `test_judge.py` | `judge` | 11 | 7 |
| `test_key.py` | `key` | 11 | 20 |
| `test_ngo.py` | `ngo` | 10 | 16 |
| `test_node.py` | `node` | 5 | 15 |
| `test_oracle.py` | `oracle` | 15 (+4 subtests) | 24 |
| `test_swap.py` | `swap` | 16 | 9 |
| `test_tax.py` | `tax` | 11 | 20 |
| `test_wallet.py` | `wallet` | 11 | 18 |

Read the failures as the work list: each failing test names a command,
its arguments and the result it expects. Note that many of the passes are
usage/flag-validation assertions (`--help` exits zero, a missing required flag
errors) that hold for any CLI, so a high pass count in a file does not mean that
file's functionality works.

## Mock API (optional)

`mock/` holds a Mockoon environment and a data generator. Neither is used by this
suite -- the tests exercise the binary directly -- but they are useful for UI
work and manual flows against a stub node.

- `mock/nyxforge.mockoon.json` -- Mockoon environment "NyxForge Mock API",
  exposing a single JSON-RPC route at `http://127.0.0.1:8888/rpc`.
- `mock/gen_bonds.js` -- generates mock bounty data (plain Node, no dependencies).

Start it with the [Mockoon](https://mockoon.com) desktop app (open the
environment file) or the CLI:

```bash
npx @mockoon/cli@latest start --data mock/nyxforge.mockoon.json
```

Point the CLI at the stub with `--rpc http://127.0.0.1:8888/rpc` or via
`config set rpc.endpoint`. Note the default port 8888 may collide with other
local services.
