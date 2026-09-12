"""
Tests for: nyx ngo
Commands: status, donations, convert, treasury, receipt, processor, config
"""
import json
import os

from conftest import NyxTestCase, run

AUTO_CONVERT_THRESHOLD = 500.0


class TestNgoStatus(NyxTestCase):

    def test_status_exits_zero(self):
        self.assertExitZero(["ngo", "status"])

    def test_status_json_has_processors(self):
        rc, out, _ = run(["ngo", "status", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("processors", data)
        self.assertIsInstance(data["processors"], list)

    def test_status_json_has_treasury(self):
        rc, out, _ = run(["ngo", "status", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("treasury", data)
        self.assertIn("xmr_balance", data["treasury"])

    def test_status_processor_keys(self):
        rc, out, _ = run(["ngo", "status", "--json"])
        self.assertEqual(rc, 0)
        for proc in json.loads(out).get("processors", []):
            for key in ("name", "status", "balance_usd"):
                self.assertIn(key, proc, f"Processor missing: {key}")

    def test_status_auto_convert_field(self):
        rc, out, _ = run(["ngo", "status", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("auto_convert", data)
        self.assertIn("enabled", data["auto_convert"])
        self.assertIn("threshold_usd", data["auto_convert"])


class TestNgoDonations(NyxTestCase):

    def test_donations_exits_zero(self):
        self.assertExitZero(["ngo", "donations"])

    def test_donations_json_is_array(self):
        rc, out, _ = run(["ngo", "donations", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_donations_item_keys(self):
        rc, out, _ = run(["ngo", "donations", "--limit", "1", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        if data:
            for key in ("date", "processor", "amount_usd", "donor", "convert_status"):
                self.assertIn(key, data[0], f"Donation item missing: {key}")

    def test_donations_limit_respected(self):
        rc, out, _ = run(["ngo", "donations", "--limit", "3", "--json"])
        self.assertEqual(rc, 0)
        self.assertLessEqual(len(json.loads(out)), 3)

    def test_donations_convert_status_values(self):
        rc, out, _ = run(["ngo", "donations", "--json"])
        self.assertEqual(rc, 0)
        valid = {"converted", "pooling", "pending", "failed"}
        for d in json.loads(out):
            s = d.get("convert_status", "").lower()
            self.assertIn(s, valid, f"Unknown convert_status: {s!r}")


class TestNgoConvert(NyxTestCase):

    def test_convert_requires_amount(self):
        rc, _, err = run(["ngo", "convert", "--route", "kraken"])
        self.assertNotEqual(rc, 0)
        self.assertIn("amount", err.lower())

    def test_convert_zero_amount_rejected(self):
        self.assertExitNonZero([
            "ngo", "convert",
            "--amount-usd", "0",
            "--route", "kraken",
        ])

    def test_convert_below_threshold_rejected(self):
        # Pool must be >= threshold before manual convert is allowed
        rc, _, err = run([
            "ngo", "convert",
            "--amount-usd", "10",
            "--route", "kraken",
        ])
        self.assertNotEqual(rc, 0)

    def test_convert_invalid_route_rejected(self):
        self.assertExitNonZero([
            "ngo", "convert",
            "--amount-usd", "600",
            "--route", "fakeroute",
        ])

    def test_convert_dry_run_returns_estimate(self):
        rc, out, _ = run([
            "ngo", "convert",
            "--amount-usd", "600",
            "--route", "kraken",
            "--dry-run", "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("est_xmr", data)
            self.assertIn("fee_usd", data)
            self.assertIn("route", data)


class TestNgoTreasury(NyxTestCase):

    def test_treasury_exits_zero(self):
        self.assertExitZero(["ngo", "treasury"])

    def test_treasury_json_keys(self):
        rc, out, _ = run(["ngo", "treasury", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        for key in ("address", "xmr_balance", "inflows_30d", "outflows_30d"):
            self.assertIn(key, data, f"Treasury JSON missing: {key}")

    def test_treasury_balance_non_negative(self):
        rc, out, _ = run(["ngo", "treasury", "--json"])
        self.assertEqual(rc, 0)
        bal = float(json.loads(out).get("xmr_balance", -1))
        self.assertGreaterEqual(bal, 0.0)


class TestNgoReceipt(NyxTestCase):

    def test_receipt_requires_donor(self):
        rc, _, err = run([
            "ngo", "receipt",
            "--amount", "1000",
            "--date", "2026-01-01",
            "--out", f"{self.tmpdir}/r.pdf",
        ])
        self.assertNotEqual(rc, 0)

    def test_receipt_requires_amount(self):
        rc, _, err = run([
            "ngo", "receipt",
            "--donor", "Test Donor",
            "--date", "2026-01-01",
            "--out", f"{self.tmpdir}/r.pdf",
        ])
        self.assertNotEqual(rc, 0)

    def test_receipt_zero_amount_rejected(self):
        self.assertExitNonZero([
            "ngo", "receipt",
            "--donor", "Test Donor",
            "--amount", "0",
            "--date", "2026-01-01",
            "--out", f"{self.tmpdir}/r.pdf",
        ])

    def test_receipt_creates_pdf(self):
        out_path = os.path.join(self.tmpdir, "receipt.pdf")
        rc, _, _ = run([
            "ngo", "receipt",
            "--donor", "Test Donor",
            "--amount", "500",
            "--date", "2026-01-01",
            "--out", out_path,
        ])
        if rc == 0:
            self.assertTrue(os.path.isfile(out_path), "Receipt PDF not created")
            with open(out_path, "rb") as f:
                self.assertTrue(f.read(4) == b"%PDF", "Output is not a valid PDF")


class TestNgoConfig(NyxTestCase):

    def test_config_exits_zero(self):
        self.assertExitZero(["ngo", "config", "--show"])

    def test_config_json_has_threshold(self):
        rc, out, _ = run(["ngo", "config", "--show", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("auto_convert_threshold_usd", data)

    def test_config_set_threshold_dry_run(self):
        rc, out, _ = run([
            "ngo", "config",
            "--set-threshold", "750",
            "--dry-run", "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertEqual(float(data.get("auto_convert_threshold_usd", 0)), 750.0)

    def test_config_negative_threshold_rejected(self):
        self.assertExitNonZero(["ngo", "config", "--set-threshold", "-100"])
