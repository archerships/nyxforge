"""
Tests for `nyx config` -- get/set/reset configuration values, profiles, network.
"""
import json
import unittest
from conftest import NyxTestCase, run


class TestConfigGet(NyxTestCase):

    def test_get_exits_zero(self):
        self.assertExitZero(["config", "get", "network"])

    def test_get_unknown_key_exits_nonzero(self):
        self.assertExitNonZero(["config", "get", "no.such.key.xyz"])

    def test_get_json_has_key_and_value(self):
        data = self.assertJsonKey(["config", "get", "network"], "key")
        self.assertIn("value", data)

    def test_get_all_exits_zero(self):
        self.assertExitZero(["config", "get", "--all"])

    def test_get_all_json_is_dict(self):
        rc, out, _ = run(["config", "get", "--all", "--json"])
        if rc == 0:
            data = json.loads(out)
            self.assertIsInstance(data, dict)
            self.assertGreater(len(data), 0)


class TestConfigSet(NyxTestCase):

    def test_set_network_mainnet(self):
        self.assertExitZero(["config", "set", "network", "mainnet"])

    def test_set_network_testnet(self):
        self.assertExitZero(["config", "set", "network", "testnet"])

    def test_set_invalid_network_rejected(self):
        self.assertExitNonZero(["config", "set", "network", "fakenet"])

    def test_set_requires_key_and_value(self):
        self.assertExitNonZero(["config", "set", "network"])

    def test_set_rpc_endpoint(self):
        self.assertExitZero([
            "config", "set", "rpc.endpoint", "http://127.0.0.1:18081",
        ])

    def test_set_invalid_rpc_url_rejected(self):
        self.assertExitNonZero([
            "config", "set", "rpc.endpoint", "not-a-url",
        ])

    def test_set_log_level(self):
        for level in ("debug", "info", "warn", "error"):
            with self.subTest(level=level):
                self.assertExitZero(["config", "set", "log.level", level])

    def test_set_invalid_log_level_rejected(self):
        self.assertExitNonZero(["config", "set", "log.level", "hyperverbose"])


class TestConfigReset(NyxTestCase):

    def test_reset_key_exits_zero(self):
        self.assertExitZero(["config", "reset", "network"])

    def test_reset_all_requires_confirm(self):
        self.assertExitNonZero(["config", "reset", "--all"])

    def test_reset_all_dry_run_exits_zero(self):
        self.assertExitZero(["config", "reset", "--all", "--dry-run"])

    def test_reset_unknown_key_exits_nonzero(self):
        self.assertExitNonZero(["config", "reset", "no.such.key.xyz"])


class TestConfigProfile(NyxTestCase):

    def test_profile_list_exits_zero(self):
        self.assertExitZero(["config", "profile", "list"])

    def test_profile_create_exits_zero(self):
        self.assertExitZero([
            "config", "profile", "create",
            "--name", "my-testnet",
            "--network", "testnet",
        ])

    def test_profile_create_requires_name(self):
        self.assertExitNonZero(["config", "profile", "create"])

    def test_profile_use_exits_zero(self):
        self.assertExitZero([
            "config", "profile", "use", "default",
        ])

    def test_profile_use_unknown_exits_nonzero(self):
        self.assertExitNonZero([
            "config", "profile", "use", "no-such-profile-xyz",
        ])

    def test_profile_delete_requires_name(self):
        self.assertExitNonZero(["config", "profile", "delete"])

    def test_profile_default_cannot_be_deleted(self):
        self.assertExitNonZero([
            "config", "profile", "delete", "default",
        ])


if __name__ == "__main__":
    unittest.main()
