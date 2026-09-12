"""
Tests for: nyx swap  (Haveno-style fiat-to-XMR P2P exchange)
Commands: list, take, confirm-sent, dispute, status, create, history
"""
import json

from conftest import NyxTestCase, run

SECURITY_DEPOSIT_PCT = 0.15


class TestSwapList(NyxTestCase):

    def test_list_exits_zero(self):
        self.assertExitZero(["swap", "list"])

    def test_list_json_is_array(self):
        rc, out, _ = run(["swap", "list", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_list_item_keys(self):
        rc, out, _ = run(["swap", "list", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        if data:
            for key in ("offer_id", "payment_method", "price_usd",
                        "min_usd", "max_usd", "maker"):
                self.assertIn(key, data[0], f"Swap list item missing: {key}")

    def test_list_method_filter(self):
        rc, out, _ = run(["swap", "list", "--method", "zelle", "--json"])
        self.assertEqual(rc, 0)
        for offer in json.loads(out):
            self.assertEqual(offer.get("payment_method", "").lower(), "zelle")

    def test_list_sort_price(self):
        self.assertExitZero(["swap", "list", "--sort", "price"])

    def test_list_invalid_method_exits_nonzero(self):
        self.assertExitNonZero(["swap", "list", "--method", "telepathy"])


class TestSwapCreate(NyxTestCase):

    def test_create_requires_method(self):
        rc, _, err = run([
            "swap", "create",
            "--price-margin", "-1.5%",
            "--min", "100", "--max", "3000",
        ])
        self.assertNotEqual(rc, 0)
        self.assertIn("method", err.lower())

    def test_create_zero_max_rejected(self):
        self.assertExitNonZero([
            "swap", "create",
            "--method", "zelle",
            "--price-margin", "-1.5%",
            "--min", "100", "--max", "0",
        ])

    def test_create_min_greater_than_max_rejected(self):
        self.assertExitNonZero([
            "swap", "create",
            "--method", "zelle",
            "--price-margin", "-1.5%",
            "--min", "5000", "--max", "100",
        ])

    def test_create_dry_run_returns_offer(self):
        rc, out, _ = run([
            "swap", "create",
            "--method", "zelle",
            "--price-margin", "-1.5%",
            "--min", "100", "--max", "3000",
            "--dry-run", "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("offer_id", data)
            self.assertIn("payment_method", data)


class TestSwapTake(NyxTestCase):

    def test_take_nonexistent_offer_exits_nonzero(self):
        self.assertExitNonZero([
            "swap", "take", "SWP-NOTEXIST",
            "--amount-usd", "200",
        ])

    def test_take_zero_amount_rejected(self):
        self.assertExitNonZero([
            "swap", "take", "SWP-0001",
            "--amount-usd", "0",
        ])

    def test_take_below_minimum_rejected(self):
        # If offer min is $100, taking $10 should fail
        rc, _, err = run(["swap", "take", "SWP-0001", "--amount-usd", "1"])
        self.assertNotEqual(rc, 0)

    def test_take_above_maximum_rejected(self):
        rc, _, err = run(["swap", "take", "SWP-0001", "--amount-usd", "9999999"])
        self.assertNotEqual(rc, 0)

    def test_take_dry_run_shows_security_deposit(self):
        rc, out, _ = run([
            "swap", "take", "SWP-0001",
            "--amount-usd", "500",
            "--dry-run", "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("security_deposit_xmr", data)
            xmr = float(data.get("xmr_amount", 0))
            deposit = float(data.get("security_deposit_xmr", 0))
            if xmr > 0:
                ratio = deposit / xmr
                self.assertAlmostEqual(ratio, SECURITY_DEPOSIT_PCT, delta=0.005,
                                       msg=f"Security deposit should be ~15% of trade, got {ratio:.1%}")


class TestSwapStatus(NyxTestCase):

    def test_status_nonexistent_trade_exits_nonzero(self):
        self.assertExitNonZero(["swap", "status", "TRD-NOTEXIST"])

    def test_status_json_keys(self):
        rc, out, _ = run(["swap", "status", "TRD-P2P-0001", "--json"])
        if rc == 0:
            data = json.loads(out)
            for key in ("trade_id", "status", "time_remaining_s",
                        "payment_method", "xmr_amount"):
                self.assertIn(key, data, f"Status JSON missing: {key}")

    def test_status_values_valid(self):
        valid_statuses = {
            "AWAITING_PAYMENT", "PAYMENT_SENT", "PAYMENT_RECEIVED",
            "COMPLETE", "DISPUTED", "REFUNDED",
        }
        rc, out, _ = run(["swap", "status", "TRD-P2P-0001", "--json"])
        if rc == 0:
            status = json.loads(out).get("status")
            self.assertIn(status, valid_statuses, f"Unknown status: {status!r}")


class TestSwapConfirmSent(NyxTestCase):

    def test_confirm_sent_nonexistent_exits_nonzero(self):
        self.assertExitNonZero(["swap", "confirm-sent", "TRD-NOTEXIST"])

    def test_confirm_sent_already_confirmed_exits_nonzero(self):
        self.assertExitNonZero(["swap", "confirm-sent", "TRD-PAYMENT-SENT"])


class TestSwapDispute(NyxTestCase):

    def test_dispute_nonexistent_exits_nonzero(self):
        self.assertExitNonZero(["swap", "dispute", "TRD-NOTEXIST"])

    def test_dispute_completed_trade_exits_nonzero(self):
        self.assertExitNonZero(["swap", "dispute", "TRD-COMPLETE"])

    def test_dispute_requires_reason(self):
        rc, _, err = run(["swap", "dispute", "TRD-P2P-0001"])
        self.assertNotEqual(rc, 0)
        self.assertIn("reason", err.lower())


class TestSwapHistory(NyxTestCase):

    def test_history_exits_zero(self):
        self.assertExitZero(["swap", "history"])

    def test_history_json_is_array(self):
        rc, out, _ = run(["swap", "history", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)
