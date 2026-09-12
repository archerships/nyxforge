"""
Tests for `nyx node` -- P2P node management, peers, sync, health.
"""
import json
import unittest
from conftest import NyxTestCase, run


class TestNodeStatus(NyxTestCase):

    def test_status_exits_zero(self):
        self.assertExitZero(["node", "status"])

    def test_status_json_has_keys(self):
        data = self.assertJsonKey(["node", "status"], "sync_status")
        self.assertIn("peers", data)
        self.assertIn("block_height", data)
        self.assertIn("version", data)

    def test_sync_status_valid_value(self):
        data = self.assertJsonKey(["node", "status"], "sync_status")
        valid = {"synced", "syncing", "stalled", "disconnected"}
        self.assertIn(data["sync_status"], valid)

    def test_block_height_non_negative(self):
        data = self.assertJsonKey(["node", "status"], "block_height")
        self.assertGreaterEqual(data["block_height"], 0)

    def test_peer_count_non_negative(self):
        data = self.assertJsonKey(["node", "status"], "peers")
        self.assertGreaterEqual(data["peers"], 0)


class TestNodePeers(NyxTestCase):

    def test_peers_exits_zero(self):
        self.assertExitZero(["node", "peers"])

    def test_peers_json_is_list(self):
        rc, out, _ = run(["node", "peers", "--json"])
        if rc == 0:
            data = json.loads(out)
            self.assertIsInstance(data, list)

    def test_add_peer_requires_address(self):
        self.assertExitNonZero(["node", "peers", "add"])

    def test_add_peer_exits_zero(self):
        self.assertExitZero([
            "node", "peers", "add",
            "--address", "192.0.2.1:18080",
        ])

    def test_remove_peer_requires_address(self):
        self.assertExitNonZero(["node", "peers", "remove"])

    def test_ban_peer_exits_zero(self):
        self.assertExitZero([
            "node", "peers", "ban",
            "--address", "192.0.2.1",
            "--duration", "24h",
        ])

    def test_ban_invalid_duration_rejected(self):
        self.assertExitNonZero([
            "node", "peers", "ban",
            "--address", "192.0.2.1",
            "--duration", "notaduration",
        ])


class TestNodeSync(NyxTestCase):

    def test_sync_status_exits_zero(self):
        self.assertExitZero(["node", "sync", "status"])

    def test_sync_json_has_keys(self):
        data = self.assertJsonKey(["node", "sync", "status"], "current_height")
        self.assertIn("target_height", data)
        self.assertIn("percent", data)

    def test_sync_percent_range(self):
        data = self.assertJsonKey(["node", "sync", "status"], "percent")
        self.assertGreaterEqual(data["percent"], 0.0)
        self.assertLessEqual(data["percent"], 100.0)

    def test_sync_resync_requires_confirm(self):
        self.assertExitNonZero(["node", "sync", "resync"])

    def test_sync_resync_dry_run_exits_zero(self):
        self.assertExitZero(["node", "sync", "resync", "--dry-run"])


class TestNodeHealth(NyxTestCase):

    def test_health_exits_zero(self):
        self.assertExitZero(["node", "health"])

    def test_health_json_has_status(self):
        data = self.assertJsonKey(["node", "health"], "status")
        valid = {"ok", "degraded", "critical"}
        self.assertIn(data["status"], valid)

    def test_health_includes_uptime(self):
        data = self.assertJsonKey(["node", "health"], "uptime_seconds")
        self.assertGreaterEqual(data["uptime_seconds"], 0)


if __name__ == "__main__":
    unittest.main()
