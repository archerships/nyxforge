"""
Tests for `nyx oracle` -- oracle/judge registry: list, register, rate, invite.
"""
import json
import unittest
from conftest import NyxTestCase, run


class TestOracleList(NyxTestCase):

    def test_list_exits_zero(self):
        self.assertExitZero(["oracle", "list"])

    def test_list_json_is_list(self):
        rc, out, _ = run(["oracle", "list", "--json"])
        if rc == 0:
            data = json.loads(out)
            self.assertIsInstance(data, list)

    def test_list_filter_by_specialty(self):
        for spec in ("longevity", "actuarial", "legal", "medical", "financial"):
            with self.subTest(spec=spec):
                self.assertExitZero(["oracle", "list", "--specialty", spec])

    def test_list_sort_by_rating(self):
        self.assertExitZero(["oracle", "list", "--sort", "rating"])

    def test_list_sort_by_decisions(self):
        self.assertExitZero(["oracle", "list", "--sort", "decisions"])

    def test_list_invalid_sort_rejected(self):
        self.assertExitNonZero(["oracle", "list", "--sort", "nonsense"])

    def test_list_json_entry_has_keys(self):
        rc, out, _ = run(["oracle", "list", "--json"])
        if rc == 0:
            data = json.loads(out)
            if data:
                entry = data[0]
                for key in ("oracle_id", "rating", "decisions", "specialty"):
                    self.assertIn(key, entry)


class TestOracleProfile(NyxTestCase):

    def test_profile_exits_zero(self):
        self.assertExitZero(["oracle", "profile", "--id", "ORK-0001"])

    def test_profile_requires_id(self):
        self.assertExitNonZero(["oracle", "profile"])

    def test_profile_json_has_keys(self):
        data = self.assertJsonKey(["oracle", "profile", "--id", "ORK-0001"], "oracle_id")
        self.assertIn("rating", data)
        self.assertIn("decisions", data)
        self.assertIn("specialty", data)

    def test_profile_rating_range(self):
        data = self.assertJsonKey(["oracle", "profile", "--id", "ORK-0001"], "rating")
        self.assertGreaterEqual(data["rating"], 0.0)
        self.assertLessEqual(data["rating"], 5.0)

    def test_profile_verdict_history_is_list(self):
        data = self.assertJsonKey(["oracle", "profile", "--id", "ORK-0001"], "verdict_history")
        self.assertIsInstance(data["verdict_history"], list)


class TestOracleRegister(NyxTestCase):

    def test_register_exits_zero(self):
        self.assertExitZero([
            "oracle", "register",
            "--key", "AABBCCDD",
            "--specialty", "longevity",
            "--fee", "50",
        ])

    def test_register_requires_key(self):
        self.assertExitNonZero([
            "oracle", "register",
            "--specialty", "longevity",
            "--fee", "50",
        ])

    def test_register_requires_specialty(self):
        self.assertExitNonZero([
            "oracle", "register",
            "--key", "AABBCCDD",
            "--fee", "50",
        ])

    def test_register_negative_fee_rejected(self):
        self.assertExitNonZero([
            "oracle", "register",
            "--key", "AABBCCDD",
            "--specialty", "longevity",
            "--fee", "-10",
        ])

    def test_register_invalid_specialty_rejected(self):
        self.assertExitNonZero([
            "oracle", "register",
            "--key", "AABBCCDD",
            "--specialty", "notaspecialty",
            "--fee", "50",
        ])

    def test_register_dry_run_exits_zero(self):
        self.assertExitZero([
            "oracle", "register",
            "--key", "AABBCCDD",
            "--specialty", "longevity",
            "--fee", "50",
            "--dry-run",
        ])


class TestOracleRate(NyxTestCase):

    def test_rate_exits_zero(self):
        self.assertExitZero([
            "oracle", "rate",
            "--id", "ORK-0001",
            "--claim", "CLM-0001",
            "--score", "4",
        ])

    def test_rate_requires_oracle_id(self):
        self.assertExitNonZero([
            "oracle", "rate",
            "--claim", "CLM-0001",
            "--score", "4",
        ])

    def test_rate_score_out_of_range_rejected(self):
        for score in ("0", "6", "-1", "100"):
            with self.subTest(score=score):
                self.assertExitNonZero([
                    "oracle", "rate",
                    "--id", "ORK-0001",
                    "--claim", "CLM-0001",
                    "--score", score,
                ])

    def test_rate_valid_scores(self):
        for score in ("1", "2", "3", "4", "5"):
            with self.subTest(score=score):
                self.assertExitZero([
                    "oracle", "rate",
                    "--id", "ORK-0001",
                    "--claim", "CLM-0001",
                    "--score", score,
                ])


class TestOracleInvite(NyxTestCase):

    def test_invite_exits_zero(self):
        self.assertExitZero([
            "oracle", "invite",
            "--oracle", "ORK-0001",
            "--series", "NYX-SERIES-A",
            "--role", "primary",
        ])

    def test_invite_requires_oracle(self):
        self.assertExitNonZero([
            "oracle", "invite",
            "--series", "NYX-SERIES-A",
            "--role", "primary",
        ])

    def test_invite_invalid_role_rejected(self):
        self.assertExitNonZero([
            "oracle", "invite",
            "--oracle", "ORK-0001",
            "--series", "NYX-SERIES-A",
            "--role", "badrole",
        ])

    def test_invite_valid_roles(self):
        for role in ("primary", "backup", "panel"):
            with self.subTest(role=role):
                self.assertExitZero([
                    "oracle", "invite",
                    "--oracle", "ORK-0001",
                    "--series", "NYX-SERIES-A",
                    "--role", role,
                ])


if __name__ == "__main__":
    unittest.main()
