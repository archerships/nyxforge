"""
Tests for: nyx auction
Commands: list, watch, bid, cancel, settle, create, history
"""
import json

from conftest import NyxTestCase, run


class TestAuctionList(NyxTestCase):

    def test_list_exits_zero(self):
        self.assertExitZero(["auction", "list"])

    def test_list_live_flag(self):
        self.assertExitZero(["auction", "list", "--live"])

    def test_list_json_is_array(self):
        rc, out, err = run(["auction", "list", "--json"])
        self.assertEqual(rc, 0, err)
        data = json.loads(out)
        self.assertIsInstance(data, list)

    def test_list_json_items_have_required_keys(self):
        rc, out, _ = run(["auction", "list", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        if data:
            item = data[0]
            for key in ("auction_id", "subject", "units_total", "units_remaining",
                        "current_price", "floor_price", "status"):
                self.assertIn(key, item, f"Auction list item missing key: {key}")


class TestAuctionCreate(NyxTestCase):

    def test_create_requires_floor_below_start(self):
        rc, _, err = run([
            "auction", "create",
            "--bounty-dir", self.tmpdir,
            "--units", "10",
            "--start-price", "50",
            "--floor-price", "100",
            "--duration", "24h",
        ])
        self.assertNotEqual(rc, 0, "Should reject floor > start price")

    def test_create_requires_positive_units(self):
        rc, _, _ = run([
            "auction", "create",
            "--bounty-dir", self.tmpdir,
            "--units", "0",
            "--start-price", "200",
            "--floor-price", "40",
            "--duration", "24h",
        ])
        self.assertNotEqual(rc, 0)

    def test_create_rejects_zero_duration(self):
        rc, _, _ = run([
            "auction", "create",
            "--bounty-dir", self.tmpdir,
            "--units", "5",
            "--start-price", "200",
            "--floor-price", "40",
            "--duration", "0h",
        ])
        self.assertNotEqual(rc, 0)

    def test_create_json_returns_auction_id(self):
        rc, out, _ = run([
            "auction", "create",
            "--bounty-dir", self.tmpdir,
            "--units", "5",
            "--start-price", "200",
            "--floor-price", "40",
            "--duration", "24h",
            "--json",
        ])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("auction_id", data)


class TestAuctionBid(NyxTestCase):

    def test_bid_nonexistent_auction_exits_nonzero(self):
        self.assertExitNonZero([
            "auction", "bid", "AUC-NOTEXIST",
            "--units", "1",
            "--max-price", "200",
        ])

    def test_bid_zero_units_rejected(self):
        self.assertExitNonZero([
            "auction", "bid", "AUC-0001",
            "--units", "0",
            "--max-price", "200",
        ])

    def test_bid_requires_max_price(self):
        rc, _, err = run(["auction", "bid", "AUC-0001", "--units", "1"])
        self.assertNotEqual(rc, 0)
        self.assertIn("max-price", err.lower())


class TestAuctionSettle(NyxTestCase):

    def test_settle_open_auction_exits_nonzero(self):
        # Cannot settle an auction that is still live
        rc, _, err = run(["auction", "settle", "AUC-OPEN-0001"])
        self.assertNotEqual(rc, 0)

    def test_settle_nonexistent_exits_nonzero(self):
        self.assertExitNonZero(["auction", "settle", "AUC-NOTEXIST"])


class TestAuctionHistory(NyxTestCase):

    def test_history_exits_zero(self):
        self.assertExitZero(["auction", "history"])

    def test_history_json_is_array(self):
        rc, out, _ = run(["auction", "history", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_history_limit_flag(self):
        rc, out, _ = run(["auction", "history", "--limit", "3", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertLessEqual(len(data), 3)
