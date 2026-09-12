"""
Tests for: nyx wallet
Commands: balance, receive, send, history, mine, export
"""
import json
import re

from conftest import NyxTestCase, run


class TestWalletBalance(NyxTestCase):

    def test_balance_exits_zero(self):
        self.assertExitZero(["wallet", "balance"])

    def test_balance_json_has_coins(self):
        rc, out, _ = run(["wallet", "balance", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("XMR", data)
        self.assertIn("NYX", data)
        self.assertIn("TARI", data)

    def test_balance_values_are_non_negative(self):
        rc, out, _ = run(["wallet", "balance", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        for coin in ("XMR", "NYX", "TARI"):
            self.assertGreaterEqual(float(data[coin]["amount"]), 0.0,
                                    f"{coin} balance should be >= 0")

    def test_balance_coin_filter(self):
        rc, out, _ = run(["wallet", "balance", "--coin", "xmr", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("XMR", data)

    def test_balance_invalid_coin_exits_nonzero(self):
        self.assertExitNonZero(["wallet", "balance", "--coin", "FAKECOIN"])


class TestWalletReceive(NyxTestCase):

    def test_receive_xmr_exits_zero(self):
        self.assertExitZero(["wallet", "receive", "--coin", "xmr"])

    def test_receive_xmr_address_format(self):
        rc, out, _ = run(["wallet", "receive", "--coin", "xmr", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        addr = data.get("address", "")
        # XMR mainnet addresses start with 4 and are 95 chars
        self.assertTrue(addr.startswith("4") and len(addr) == 95,
                        f"Unexpected XMR address: {addr!r}")

    def test_receive_nyx_exits_zero(self):
        self.assertExitZero(["wallet", "receive", "--coin", "nyx"])

    def test_receive_tari_exits_zero(self):
        self.assertExitZero(["wallet", "receive", "--coin", "tari"])

    def test_receive_unknown_coin_exits_nonzero(self):
        self.assertExitNonZero(["wallet", "receive", "--coin", "dogecoin"])

    def test_receive_json_has_qr_path(self):
        rc, out, _ = run(["wallet", "receive", "--coin", "xmr", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        self.assertIn("qr_path", data)


class TestWalletSend(NyxTestCase):

    def test_send_requires_to_address(self):
        rc, _, err = run(["wallet", "send", "--coin", "xmr", "--amount", "0.1"])
        self.assertNotEqual(rc, 0)
        self.assertIn("to", err.lower())

    def test_send_requires_amount(self):
        rc, _, err = run(["wallet", "send", "--coin", "xmr", "--to", "4xAddr..."])
        self.assertNotEqual(rc, 0)
        self.assertIn("amount", err.lower())

    def test_send_zero_amount_rejected(self):
        self.assertExitNonZero([
            "wallet", "send",
            "--coin", "xmr",
            "--to", "4xTestAddr",
            "--amount", "0",
        ])

    def test_send_negative_amount_rejected(self):
        self.assertExitNonZero([
            "wallet", "send",
            "--coin", "xmr",
            "--to", "4xTestAddr",
            "--amount", "-1",
        ])

    def test_send_insufficient_balance_exits_nonzero(self):
        self.assertExitNonZero([
            "wallet", "send",
            "--coin", "xmr",
            "--to", "4xTestAddr",
            "--amount", "999999999",
        ])

    def test_send_dry_run_returns_fee(self):
        rc, out, _ = run([
            "wallet", "send",
            "--coin", "xmr",
            "--to", "4xTestAddr",
            "--amount", "0.001",
            "--dry-run", "--json",
        ])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("fee", data)
            self.assertIn("total", data)


class TestWalletHistory(NyxTestCase):

    def test_history_exits_zero(self):
        self.assertExitZero(["wallet", "history"])

    def test_history_json_is_array(self):
        rc, out, _ = run(["wallet", "history", "--json"])
        self.assertEqual(rc, 0)
        self.assertIsInstance(json.loads(out), list)

    def test_history_coin_filter(self):
        rc, out, _ = run(["wallet", "history", "--coin", "xmr", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        for tx in data:
            self.assertEqual(tx.get("coin", "XMR"), "XMR")

    def test_history_limit_respected(self):
        rc, out, _ = run(["wallet", "history", "--limit", "3", "--json"])
        self.assertEqual(rc, 0)
        self.assertLessEqual(len(json.loads(out)), 3)

    def test_history_item_keys(self):
        rc, out, _ = run(["wallet", "history", "--limit", "1", "--json"])
        self.assertEqual(rc, 0)
        data = json.loads(out)
        if data:
            for key in ("date", "amount", "direction", "type", "balance_after"):
                self.assertIn(key, data[0], f"History item missing: {key}")

    def test_history_direction_values(self):
        rc, out, _ = run(["wallet", "history", "--json"])
        self.assertEqual(rc, 0)
        for tx in json.loads(out):
            self.assertIn(tx.get("direction"), ("in", "out"),
                          f"Unexpected direction: {tx.get('direction')!r}")


class TestWalletMine(NyxTestCase):

    def test_mine_requires_coins_flag(self):
        rc, _, err = run(["wallet", "mine"])
        self.assertNotEqual(rc, 0)

    def test_mine_invalid_coin_rejected(self):
        self.assertExitNonZero(["wallet", "mine", "--coins", "fakecoin"])

    def test_mine_status_dry_run(self):
        rc, out, _ = run(["wallet", "mine", "--coins", "xmr,nyx,tari",
                          "--dry-run", "--json"])
        if rc == 0:
            data = json.loads(out)
            self.assertIn("pools", data)
            self.assertIn("threads", data)

    def test_mine_stop_when_not_running_exits_nonzero(self):
        self.assertExitNonZero(["wallet", "mine", "stop"])


class TestWalletExport(NyxTestCase):

    def test_export_requires_passphrase(self):
        rc, _, err = run(["wallet", "export",
                          "--out", f"{self.tmpdir}/wallet.enc"])
        self.assertNotEqual(rc, 0)

    def test_export_dry_run_exits_zero(self):
        rc, out, _ = run(["wallet", "export",
                          "--out", f"{self.tmpdir}/wallet.enc",
                          "--passphrase", "test-pass-1234",
                          "--dry-run"])
        self.assertEqual(rc, 0)
