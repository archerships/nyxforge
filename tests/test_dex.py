"""
Tests for: nyx dex
Commands: list, take, create, cancel, trades, pairs, price
"""
import json

from conftest import NyxTestCase, run


class TestDexPairs(NyxTestCase):

    def test_pairs_exits_zero(self):
        self.assertExitZero(["dex", "pairs"])

    def test_pairs_json_is_array(self):
        rc, out, _ = run(["dex", "pairs", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_nyx_xmr_pair_exists(self):
        rc, out, _ = run(["dex", "pairs", "--json"])
        self.assertEqual(rc, 0)
        pairs = [p.get("pair", "") for p in json.loads(out)]
        self.assertIn("NYX/XMR", pairs)


class TestDexList(NyxTestCase):

    def test_list_requires_pair(self):
        rc, _, err = run(["dex", "list"])
        self.assertNotEqual(rc, 0)
        self.assertIn("pair", err.lower())

    def test_list_valid_pair_exits_zero(self):
        self.assertExitZero(["dex", "list", "--pair", "NYX/XMR"])

    def test_list_json_is_array(self):
        rc, out, _ = run(["dex", "list", "--pair", "NYX/XMR", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_list_json_item_keys(self):
        rc, out, _ = run(["dex", "list", "--pair", "NYX/XMR", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        if data:
            for key in ("offer_id", "side", "price", "amount", "min_amount", "maker"):
                self.assertIn(key, data[0], f"Offer list item missing: {key}")

    def test_list_sort_price(self):
        rc, out, _ = run(["dex", "list", "--pair", "NYX/XMR", "--sort", "price", "--json"])
        self.assertEqual(rc, 0)

    def test_list_invalid_pair_exits_nonzero(self):
        self.assertExitNonZero(["dex", "list", "--pair", "FAKE/PAIR"])


class TestDexTake(NyxTestCase):

    def test_take_nonexistent_offer_exits_nonzero(self):
        self.assertExitNonZero([
            "dex", "take", "OFF-NOTEXIST",
            "--amount", "1000",
        ])

    def test_take_zero_amount_rejected(self):
        self.assertExitNonZero([
            "dex", "take", "OFF-0001",
            "--amount", "0",
        ])

    def test_take_below_minimum_rejected(self):
        # If offer minimum is 1000, taking 10 should fail
        rc, _, err = run(["dex", "take", "OFF-0001", "--amount", "1"])
        self.assertNotEqual(rc, 0)

    def test_take_dry_run_returns_preview(self):
        rc, out, _ = run([
            "dex", "take", "OFF-0001",
            "--amount", "5000",
            "--dry-run", "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("total_cost", data)
            self.assertIn("settlement_type", data)


class TestDexCreate(NyxTestCase):

    def test_create_requires_pair(self):
        rc, _, err = run([
            "dex", "create",
            "--side", "sell",
            "--price", "0.00018",
            "--amount", "50000",
        ])
        self.assertNotEqual(rc, 0)

    def test_create_requires_side(self):
        rc, _, err = run([
            "dex", "create",
            "--pair", "NYX/XMR",
            "--price", "0.00018",
            "--amount", "50000",
        ])
        self.assertNotEqual(rc, 0)

    def test_create_invalid_side_rejected(self):
        rc, _, _ = run([
            "dex", "create",
            "--pair", "NYX/XMR",
            "--side", "sideways",
            "--price", "0.00018",
            "--amount", "50000",
        ])
        self.assertNotEqual(rc, 0)

    def test_create_zero_price_rejected(self):
        rc, _, _ = run([
            "dex", "create",
            "--pair", "NYX/XMR",
            "--side", "sell",
            "--price", "0",
            "--amount", "50000",
        ])
        self.assertNotEqual(rc, 0)

    def test_create_dry_run_returns_offer_preview(self):
        rc, out, _ = run([
            "dex", "create",
            "--pair", "NYX/XMR",
            "--side", "sell",
            "--price", "0.00019",
            "--amount", "100000",
            "--min", "5000",
            "--dry-run", "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("offer_id", data)
            self.assertIn("locked_amount", data)


class TestDexCancel(NyxTestCase):

    def test_cancel_nonexistent_exits_nonzero(self):
        self.assertExitNonZero(["dex", "cancel", "OFF-NOTEXIST"])

    def test_cancel_others_offer_exits_nonzero(self):
        # Cannot cancel an offer owned by a different key
        self.assertExitNonZero(["dex", "cancel", "OFF-OTHER-KEY"])


class TestDexPrice(NyxTestCase):

    def test_price_exits_zero(self):
        self.assertExitZero(["dex", "price", "--pair", "NYX/XMR"])

    def test_price_json_has_mid(self):
        rc, out, _ = run(["dex", "price", "--pair", "NYX/XMR", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("mid", data)
        self.assertIn("spread", data)
        self.assertIn("volume_24h", data)


class TestDexTrades(NyxTestCase):

    def test_trades_exits_zero(self):
        self.assertExitZero(["dex", "trades"])

    def test_trades_open_flag(self):
        self.assertExitZero(["dex", "trades", "--open"])

    def test_trades_history_json(self):
        rc, out, _ = run(["dex", "trades", "--history", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_trades_limit_respected(self):
        rc, out, _ = run(["dex", "trades", "--history", "--limit", "2", "--json"])
        self.assertEqual(rc, 0)
        self.assertLessEqual(len(json.loads(out)), 2)
