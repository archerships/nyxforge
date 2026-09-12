"""
Tests for: nyx bounty
Commands: create, list, inspect, verify, transfer, claim
"""
import os
import sqlite3

from conftest import NyxTestCase, run


class TestBountyCreate(NyxTestCase):

    def test_create_produces_sqlite_file(self):
        out = self.assertExitZero([
            "bounty", "create",
            "--subject", "Test Subject",
            "--collateral", "10",
            "--denom", "XMR",
            "--maturity", "2035-01-01",
            "--judge", "9dFkABCDEFGH3rPx",
            "--out", os.path.join(self.tmpdir, "test.bounty"),
        ])
        path = os.path.join(self.tmpdir, "test.bounty")
        self.assertTrue(os.path.isfile(path), "Output .bounty file not created")

    def test_created_file_is_sqlite(self):
        path = os.path.join(self.tmpdir, "ts.bounty")
        self.assertExitZero([
            "bounty", "create",
            "--subject", "TS", "--collateral", "1", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", path,
        ])
        with sqlite3.connect(path) as conn:
            tables = {r[0] for r in conn.execute(
                "SELECT name FROM sqlite_master WHERE type='table'")}
        self.assertIn("bounty_spec", tables)
        self.assertIn("oracles", tables)
        self.assertIn("collateral", tables)
        self.assertIn("oracle_config", tables)
        self.assertIn("terms", tables)

    def test_create_requires_subject(self):
        self.assertExitNonZero([
            "bounty", "create",
            "--collateral", "10", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", os.path.join(self.tmpdir, "x.bounty"),
        ])

    def test_create_requires_positive_collateral(self):
        self.assertExitNonZero([
            "bounty", "create",
            "--subject", "X", "--collateral", "0", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", os.path.join(self.tmpdir, "x.bounty"),
        ])

    def test_create_rejects_past_maturity(self):
        self.assertExitNonZero([
            "bounty", "create",
            "--subject", "X", "--collateral", "10", "--denom", "XMR",
            "--maturity", "2000-01-01", "--judge", "ABCD1234",
            "--out", os.path.join(self.tmpdir, "x.bounty"),
        ])

    def test_create_rejects_maturity_beyond_10_years(self):
        # CRQC threat window: CLI enforces a 10-year hard cap on maturity
        self.assertExitNonZero([
            "bounty", "create",
            "--subject", "X", "--collateral", "10", "--denom", "XMR",
            "--maturity", "2045-01-01", "--judge", "ABCD1234",
            "--out", os.path.join(self.tmpdir, "x.bounty"),
        ])

    def test_json_output_contains_bounty_id(self):
        path = os.path.join(self.tmpdir, "j.bounty")
        rc, out, err = run([
            "bounty", "create",
            "--subject", "JSON Test", "--collateral", "5", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", path, "--json",
        ])
        self.assertEqual(rc, 0)
        import json
        data = json.loads(out)
        self.assertIn("bounty_id", data)
        self.assertIn("file", data)


class TestBountyInspect(NyxTestCase):

    def _make_bounty(self, name="t.bounty"):
        path = os.path.join(self.tmpdir, name)
        self.assertExitZero([
            "bounty", "create",
            "--subject", "Inspector Test", "--collateral", "20", "--denom", "XMR",
            "--maturity", "2036-01-01", "--judge", "ABCD1234",
            "--out", path,
        ])
        return path

    def test_inspect_prints_subject(self):
        path = self._make_bounty()
        self.assertOutputContains(["bounty", "inspect", path], "Inspector Test")

    def test_inspect_prints_collateral(self):
        path = self._make_bounty()
        self.assertOutputContains(["bounty", "inspect", path], "20")

    def test_inspect_shows_dleq_valid(self):
        path = self._make_bounty()
        self.assertOutputContains(["bounty", "inspect", path], "valid")

    def test_inspect_nonexistent_file_exits_nonzero(self):
        self.assertExitNonZero(["bounty", "inspect", "/tmp/no-such.bounty"])

    def test_inspect_json_keys(self):
        path = self._make_bounty()
        rc, out, _ = run(["bounty", "inspect", path, "--json"])
        self.assertEqual(rc, 0)
        import json
        data = json.loads(out)
        for key in ("bounty_id", "subject", "collateral", "currency", "maturity",
                    "judge_key", "dleq_valid", "bearer_key"):
            self.assertIn(key, data, f"JSON missing key: {key}")


class TestBountyVerify(NyxTestCase):

    def test_verify_passes_on_fresh_file(self):
        path = os.path.join(self.tmpdir, "v.bounty")
        self.assertExitZero([
            "bounty", "create",
            "--subject", "Verify Test", "--collateral", "5", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", path,
        ])
        self.assertExitZero(["bounty", "verify", path])

    def test_verify_fails_on_tampered_file(self):
        path = os.path.join(self.tmpdir, "tamper.bounty")
        self.assertExitZero([
            "bounty", "create",
            "--subject", "Tamper Test", "--collateral", "5", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", path,
        ])
        # Corrupt the file
        with open(path, "r+b") as f:
            f.seek(512)
            f.write(b"\x00" * 64)
        self.assertExitNonZero(["bounty", "verify", path])


class TestBountyList(NyxTestCase):

    def test_list_shows_created_bounty(self):
        path = os.path.join(self.tmpdir, "list.bounty")
        self.assertExitZero([
            "bounty", "create",
            "--subject", "List Subject", "--collateral", "7", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", path,
        ])
        self.assertOutputContains(
            ["bounty", "list", "--dir", self.tmpdir],
            "List Subject",
        )

    def test_list_empty_dir_exits_zero(self):
        self.assertExitZero(["bounty", "list", "--dir", self.tmpdir])


class TestBountyTransfer(NyxTestCase):

    def test_transfer_changes_bearer_key(self):
        path = os.path.join(self.tmpdir, "xfer.bounty")
        self.assertExitZero([
            "bounty", "create",
            "--subject", "Transfer Test", "--collateral", "3", "--denom", "XMR",
            "--maturity", "2035-01-01", "--judge", "ABCD1234",
            "--out", path,
        ])
        rc, out, _ = run(["bounty", "inspect", path, "--json"])
        import json
        original_key = json.loads(out)["bearer_key"]

        new_key = "NEWBEARER00000001"
        self.assertExitZero(["bounty", "transfer", path, "--to", new_key])

        rc2, out2, _ = run(["bounty", "inspect", path, "--json"])
        updated_key = json.loads(out2)["bearer_key"]
        self.assertNotEqual(original_key, updated_key)
        self.assertEqual(updated_key, new_key)
