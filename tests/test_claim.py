"""
Tests for `nyx claim` — file and track claims against .bounty issuers.
"""
import os
import unittest
from conftest import NyxTestCase, run


class TestClaimFile(NyxTestCase):

    def test_file_exits_zero(self):
        self.assertExitZero([
            "claim", "file",
            "--bounty", "NYX-0001",
            "--type", "maturity",
            "--narrative", "Subject reached maturity.",
        ])

    def test_file_requires_bounty_id(self):
        self.assertExitNonZero(["claim", "file", "--type", "maturity"])

    def test_file_requires_claim_type(self):
        self.assertExitNonZero(["claim", "file", "--bounty", "NYX-0001"])

    def test_file_invalid_type_rejected(self):
        self.assertExitNonZero([
            "claim", "file",
            "--bounty", "NYX-0001",
            "--type", "fakeType",
        ])

    def test_file_returns_claim_id(self):
        out = self.assertExitZero([
            "claim", "file",
            "--bounty", "NYX-0001",
            "--type", "maturity",
            "--narrative", "Maturity payout.",
        ])
        self.assertRegex(out, r"CLM-[0-9A-F]{4}")

    def test_valid_claim_types(self):
        for claim_type in ("maturity", "partial", "default", "expired"):
            with self.subTest(type=claim_type):
                self.assertExitZero([
                    "claim", "file",
                    "--bounty", "NYX-0001",
                    "--type", claim_type,
                ])


class TestClaimEvidence(NyxTestCase):

    def test_attach_exits_zero(self):
        self.assertExitZero([
            "claim", "attach",
            "--claim", "CLM-0001",
            "--file", "/dev/null",
            "--desc", "Bio cert",
        ])

    def test_attach_requires_claim_id(self):
        self.assertExitNonZero(["claim", "attach", "--file", "/dev/null"])

    def test_attach_requires_file(self):
        self.assertExitNonZero(["claim", "attach", "--claim", "CLM-0001"])

    def test_attach_missing_file_rejected(self):
        self.assertExitNonZero([
            "claim", "attach",
            "--claim", "CLM-0001",
            "--file", "/tmp/no-such-evidence-xyzabc.pdf",
        ])


class TestClaimStatus(NyxTestCase):

    def test_status_exits_zero(self):
        self.assertExitZero(["claim", "status", "--claim", "CLM-0001"])

    def test_status_json_has_keys(self):
        data = self.assertJsonKey(["claim", "status", "--claim", "CLM-0001"], "status")
        self.assertIn("claim_id", data)
        self.assertIn("bounty_id", data)

    def test_status_values_valid(self):
        data = self.assertJsonKey(["claim", "status", "--claim", "CLM-0001"], "status")
        valid = {"FILED", "EVIDENCE", "REVIEW", "DECIDED", "APPEAL",
                 "WITHDRAWN", "APPROVED", "DENIED"}
        self.assertIn(data["status"], valid)

    def test_list_exits_zero(self):
        self.assertExitZero(["claim", "list"])

    def test_list_json_is_list(self):
        rc, out, _ = run(["claim", "list", "--json"])
        if rc == 0:
            import json
            data = json.loads(out)
            self.assertIsInstance(data, list)


class TestClaimWithdraw(NyxTestCase):

    def test_withdraw_requires_claim(self):
        self.assertExitNonZero(["claim", "withdraw"])

    def test_withdraw_dry_run_exits_zero(self):
        self.assertExitZero([
            "claim", "withdraw",
            "--claim", "CLM-0001",
            "--dry-run",
        ])

    def test_withdraw_decided_claim_rejected(self):
        self.assertExitNonZero([
            "claim", "withdraw",
            "--claim", "CLM-DECIDED",
        ])


if __name__ == "__main__":
    unittest.main()
