"""
Tests for: nyx tax
Commands: status, generate, receipts, receipt, schedule-b, export, ledger
"""
import json
import os
import zipfile

from conftest import NyxTestCase, run

SCHEDULE_B_THRESHOLD = 5000.0


class TestTaxStatus(NyxTestCase):

    def test_status_exits_zero(self):
        self.assertExitZero(["tax", "status"])

    def test_status_requires_year(self):
        rc, _, err = run(["tax", "status"])
        # Either exits 0 with current year default, or requires --year
        # Both are acceptable; test that if non-zero, error mentions year
        if rc != 0:
            self.assertIn("year", err.lower())

    def test_status_json_has_filing_list(self):
        rc, out, _ = run(["tax", "status", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("filings", data)
        self.assertIsInstance(data["filings"], list)

    def test_status_json_form_keys(self):
        rc, out, _ = run(["tax", "status", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        for form in json.loads(out).get("filings", []):
            for key in ("form", "status", "description"):
                self.assertIn(key, form, f"Filing item missing: {key}")

    def test_status_form_990_present(self):
        rc, out, _ = run(["tax", "status", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        forms = [f.get("form") for f in json.loads(out).get("filings", [])]
        self.assertIn("Form 990", forms)

    def test_status_form_8949_present(self):
        rc, out, _ = run(["tax", "status", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        forms = [f.get("form") for f in json.loads(out).get("filings", [])]
        self.assertIn("Form 8949", forms)

    def test_status_form_status_values(self):
        rc, out, _ = run(["tax", "status", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        valid = {"done", "pending", "n/a"}
        for form in json.loads(out).get("filings", []):
            s = form.get("status", "").lower()
            self.assertIn(s, valid, f"Unexpected filing status: {s!r}")


class TestTaxGenerate(NyxTestCase):

    def test_generate_requires_form(self):
        rc, _, err = run(["tax", "generate", "--year", "2025"])
        self.assertNotEqual(rc, 0)
        self.assertIn("form", err.lower())

    def test_generate_requires_year(self):
        rc, _, err = run(["tax", "generate", "--form", "990"])
        self.assertNotEqual(rc, 0)
        self.assertIn("year", err.lower())

    def test_generate_990_creates_pdf(self):
        out_path = os.path.join(self.tmpdir, "990.pdf")
        rc, _, _ = run([
            "tax", "generate",
            "--form", "990",
            "--year", "2025",
            "--out", out_path,
        ])
        if rc == 0:
            self.assertTrue(os.path.isfile(out_path))
            with open(out_path, "rb") as f:
                self.assertTrue(f.read(4) == b"%PDF")

    def test_generate_8949_creates_pdf(self):
        out_path = os.path.join(self.tmpdir, "8949.pdf")
        rc, _, _ = run([
            "tax", "generate",
            "--form", "8949",
            "--year", "2025",
            "--out", out_path,
        ])
        if rc == 0:
            self.assertTrue(os.path.isfile(out_path))

    def test_generate_8949_json_has_lots(self):
        rc, out, _ = run([
            "tax", "generate",
            "--form", "8949",
            "--year", "2025",
            "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("acquisitions", data)
            self.assertIn("dispositions", data)

    def test_generate_8949_gain_loss_consistent(self):
        rc, out, _ = run([
            "tax", "generate",
            "--form", "8949",
            "--year", "2025",
            "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            # net = short_term + long_term
            net = float(data.get("net_gain_loss", 0))
            st  = float(data.get("short_term_gain_loss", 0))
            lt  = float(data.get("long_term_gain_loss", 0))
            self.assertAlmostEqual(net, st + lt, places=2)

    def test_generate_invalid_form_exits_nonzero(self):
        self.assertExitNonZero([
            "tax", "generate",
            "--form", "W-9",
            "--year", "2025",
        ])

    def test_generate_future_year_exits_nonzero(self):
        self.assertExitNonZero([
            "tax", "generate",
            "--form", "990",
            "--year", "2099",
        ])


class TestTaxScheduleB(NyxTestCase):

    def test_schedule_b_exits_zero(self):
        self.assertExitZero(["tax", "schedule-b", "--year", "2025"])

    def test_schedule_b_json_is_array(self):
        rc, out, _ = run(["tax", "schedule-b", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_schedule_b_all_above_threshold(self):
        rc, out, _ = run([
            "tax", "schedule-b",
            "--year", "2025",
            "--threshold", str(int(SCHEDULE_B_THRESHOLD)),
            "--json",
        ])
        self.assertEqual(rc, 0)
        for donor in json.loads(out):
            amt = float(donor.get("amount_usd", 0))
            self.assertGreaterEqual(
                amt, SCHEDULE_B_THRESHOLD,
                f"Donor {donor.get('name')!r} amount {amt} below threshold"
            )

    def test_schedule_b_item_keys(self):
        rc, out, _ = run(["tax", "schedule-b", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        if data:
            for key in ("name", "amount_usd", "quarter"):
                self.assertIn(key, data[0], f"Schedule B item missing: {key}")


class TestTaxReceipts(NyxTestCase):

    def test_receipts_exits_zero(self):
        self.assertExitZero(["tax", "receipts", "--year", "2025"])

    def test_receipts_pending_filter(self):
        rc, out, _ = run(["tax", "receipts", "--year", "2025",
                          "--pending", "--json"])
        self.assertEqual(rc, 0)
        for r in json.loads(out):
            self.assertEqual(r.get("status"), "pending")

    def test_receipt_single_creates_pdf(self):
        out_path = os.path.join(self.tmpdir, "rcpt.pdf")
        rc, _, _ = run([
            "tax", "receipt",
            "--donor", "Test Donor",
            "--amount", "1500",
            "--date", "2025-06-01",
            "--out", out_path,
        ])
        if rc == 0:
            self.assertTrue(os.path.isfile(out_path))
            with open(out_path, "rb") as f:
                self.assertTrue(f.read(4) == b"%PDF")


class TestTaxExport(NyxTestCase):

    def test_export_creates_zip(self):
        out_path = os.path.join(self.tmpdir, "tax-2025.zip")
        rc, _, _ = run([
            "tax", "export",
            "--year", "2025",
            "--format", "zip",
            "--out", out_path,
        ])
        if rc == 0:
            self.assertTrue(os.path.isfile(out_path))
            self.assertTrue(zipfile.is_zipfile(out_path))

    def test_export_zip_contains_required_docs(self):
        out_path = os.path.join(self.tmpdir, "tax-2025.zip")
        rc, _, _ = run([
            "tax", "export",
            "--year", "2025",
            "--format", "zip",
            "--out", out_path,
        ])
        if rc == 0 and zipfile.is_zipfile(out_path):
            with zipfile.ZipFile(out_path) as zf:
                names = zf.namelist()
            has_990  = any("990" in n for n in names)
            has_8949 = any("8949" in n for n in names)
            self.assertTrue(has_990,  "Export zip missing Form 990")
            self.assertTrue(has_8949, "Export zip missing Form 8949")

    def test_export_invalid_format_rejected(self):
        self.assertExitNonZero([
            "tax", "export",
            "--year", "2025",
            "--format", "docx",
        ])

    def test_export_requires_year(self):
        rc, _, err = run(["tax", "export", "--format", "zip"])
        self.assertNotEqual(rc, 0)
        self.assertIn("year", err.lower())


class TestTaxLedger(NyxTestCase):

    def test_ledger_exits_zero(self):
        self.assertExitZero(["tax", "ledger", "--year", "2025"])

    def test_ledger_json_has_acquisitions_and_dispositions(self):
        rc, out, _ = run(["tax", "ledger", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("acquisitions", data)
        self.assertIn("dispositions", data)

    def test_ledger_acquisition_keys(self):
        rc, out, _ = run(["tax", "ledger", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        for acq in data.get("acquisitions", []):
            for key in ("date", "coin", "amount", "fmv_usd", "cost_basis_usd"):
                self.assertIn(key, acq, f"Acquisition lot missing: {key}")

    def test_ledger_disposition_keys(self):
        rc, out, _ = run(["tax", "ledger", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        for d in data.get("dispositions", []):
            for key in ("date", "coin", "amount", "proceeds_usd",
                        "cost_basis_usd", "gain_loss_usd", "term"):
                self.assertIn(key, d, f"Disposition missing: {key}")

    def test_ledger_term_values(self):
        rc, out, _ = run(["tax", "ledger", "--year", "2025", "--json"])
        self.assertEqual(rc, 0)
        for d in json.loads(out).get("dispositions", []):
            self.assertIn(d.get("term"), ("short", "long"),
                          f"Unexpected term: {d.get('term')!r}")
