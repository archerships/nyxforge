"""
Tests for: nyx judge
Commands: list-claims, review, request-info, decide, appeal, history, register
"""
import json

from conftest import NyxTestCase, run


class TestJudgeListClaims(NyxTestCase):

    def test_list_claims_exits_zero(self):
        self.assertExitZero(["judge", "list-claims"])

    def test_list_claims_json_is_array(self):
        rc, out, _ = run(["judge", "list-claims", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_list_claims_json_item_keys(self):
        rc, out, _ = run(["judge", "list-claims", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        if data:
            for key in ("claim_id", "bounty_id", "claimant", "filed", "status", "deadline"):
                self.assertIn(key, data[0], f"Claim list item missing key: {key}")

    def test_list_claims_status_filter(self):
        rc, out, _ = run(["judge", "list-claims", "--status", "REVIEW", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        for item in data:
            self.assertEqual(item.get("status"), "REVIEW")


class TestJudgeReview(NyxTestCase):

    def test_review_nonexistent_claim_exits_nonzero(self):
        self.assertExitNonZero(["judge", "review", "CLM-NOTEXIST"])

    def test_review_json_has_evidence_list(self):
        rc, out, _ = run(["judge", "review", "CLM-0001", "--json"])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("evidence", data)
            self.assertIsInstance(data["evidence"], list)

    def test_review_evidence_items_have_hash(self):
        rc, out, _ = run(["judge", "review", "CLM-0001", "--json"])
        if rc == 0:
            data = json.loads(out)
            for item in data.get("evidence", []):
                self.assertIn("sha256", item)
                self.assertIn("verified", item)


class TestJudgeDecide(NyxTestCase):

    def test_decide_approve_requires_valid_claim(self):
        self.assertExitNonZero([
            "judge", "decide", "CLM-NOTEXIST",
            "--verdict", "approve",
            "--note", "Test approval",
        ])

    def test_decide_deny_requires_valid_claim(self):
        self.assertExitNonZero([
            "judge", "decide", "CLM-NOTEXIST",
            "--verdict", "deny",
            "--note", "Test denial",
        ])

    def test_decide_requires_verdict_flag(self):
        rc, _, err = run(["judge", "decide", "CLM-0001", "--note", "x"])
        self.assertNotEqual(rc, 0)
        self.assertIn("verdict", err.lower())

    def test_decide_rejects_invalid_verdict_value(self):
        rc, _, err = run([
            "judge", "decide", "CLM-0001",
            "--verdict", "maybe",
            "--note", "x",
        ])
        self.assertNotEqual(rc, 0)

    def test_decide_json_returns_verdict_tx(self):
        rc, out, _ = run([
            "judge", "decide", "CLM-0001",
            "--verdict", "approve",
            "--note", "Approved in test",
            "--json", "--dry-run",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("verdict_tx", data)
            self.assertIn("payout_tx", data)


class TestJudgeAppeal(NyxTestCase):

    def test_appeal_nonexistent_claim_exits_nonzero(self):
        self.assertExitNonZero(["judge", "appeal", "CLM-NOTEXIST"])

    def test_appeal_undecided_claim_exits_nonzero(self):
        # Can only appeal decided claims
        self.assertExitNonZero(["judge", "appeal", "CLM-PENDING"])


class TestJudgeRegister(NyxTestCase):

    def test_register_json_returns_judge_key(self):
        rc, out, _ = run(["judge", "register", "--json", "--dry-run"])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("judge_key", data)

    def test_register_dry_run_does_not_broadcast(self):
        rc, out, _ = run(["judge", "register", "--dry-run", "--json"])
        if rc == 0:
            data = json.loads(out)
            self.assertNotIn("tx_id", data,
                             "Dry run should not produce a broadcast TX")


class TestJudgeHistory(NyxTestCase):

    def test_history_exits_zero(self):
        self.assertExitZero(["judge", "history"])

    def test_history_json_is_array(self):
        rc, out, _ = run(["judge", "history", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)
