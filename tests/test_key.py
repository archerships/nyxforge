"""
Tests for `nyx key` -- Ed25519 keypair management, keystore, rotation.
"""
import json
import os
import unittest
from conftest import NyxTestCase, run


class TestKeyGenerate(NyxTestCase):

    def test_generate_exits_zero(self):
        self.assertExitZero([
            "key", "generate",
            "--label", "test-key",
            "--passphrase", "hunter2",
        ])

    def test_generate_requires_label(self):
        self.assertExitNonZero(["key", "generate", "--passphrase", "hunter2"])

    def test_generate_requires_passphrase(self):
        self.assertExitNonZero(["key", "generate", "--label", "test-key"])

    def test_generate_creates_keystore_file(self):
        out_path = os.path.join(self.tmpdir, "test.nyx")
        self.assertExitZero([
            "key", "generate",
            "--label", "test-key",
            "--passphrase", "hunter2",
            "--out", out_path,
        ])
        self.assertTrue(os.path.exists(out_path))

    def test_generate_json_has_fingerprint(self):
        data = self.assertJsonKey([
            "key", "generate",
            "--label", "test-key",
            "--passphrase", "hunter2",
        ], "fingerprint")
        self.assertIsInstance(data["fingerprint"], str)
        self.assertGreater(len(data["fingerprint"]), 8)

    def test_generate_uses_ed25519(self):
        data = self.assertJsonKey([
            "key", "generate",
            "--label", "test-key",
            "--passphrase", "hunter2",
        ], "algorithm")
        self.assertEqual(data["algorithm"], "Ed25519")

    def test_generate_alg_epoch_is_zero(self):
        data = self.assertJsonKey([
            "key", "generate",
            "--label", "test-key",
            "--passphrase", "hunter2",
        ], "alg_epoch")
        self.assertEqual(data["alg_epoch"], 0)

    def test_weak_passphrase_warns(self):
        rc, out, err = run([
            "key", "generate",
            "--label", "test-key",
            "--passphrase", "abc",
        ])
        combined = out + err
        self.assertIn("weak", combined.lower())


class TestKeyList(NyxTestCase):

    def test_list_exits_zero(self):
        self.assertExitZero(["key", "list"])

    def test_list_json_is_list(self):
        rc, out, _ = run(["key", "list", "--json"])
        if rc == 0:
            data = json.loads(out)
            self.assertIsInstance(data, list)

    def test_list_filter_by_usage(self):
        for usage in ("bounty", "judge", "wallet", "any"):
            with self.subTest(usage=usage):
                self.assertExitZero(["key", "list", "--usage", usage])


class TestKeyImport(NyxTestCase):

    def test_import_pem_exits_zero(self):
        self.assertExitZero([
            "key", "import",
            "--format", "pem",
            "--file", "/dev/null",
            "--passphrase", "hunter2",
        ])

    def test_import_requires_format(self):
        self.assertExitNonZero(["key", "import", "--file", "/dev/null"])

    def test_import_invalid_format_rejected(self):
        self.assertExitNonZero([
            "key", "import",
            "--format", "weirdformat",
            "--file", "/dev/null",
        ])

    def test_import_valid_formats(self):
        for fmt in ("pem", "nyx", "raw"):
            with self.subTest(fmt=fmt):
                self.assertExitZero([
                    "key", "import",
                    "--format", fmt,
                    "--file", "/dev/null",
                    "--dry-run",
                ])


class TestKeyExport(NyxTestCase):

    def test_export_requires_fingerprint(self):
        self.assertExitNonZero(["key", "export", "--format", "pem"])

    def test_export_invalid_format_rejected(self):
        self.assertExitNonZero([
            "key", "export",
            "--fingerprint", "AABBCCDD",
            "--format", "badformat",
        ])

    def test_export_valid_formats(self):
        for fmt in ("pem", "nyx", "qr"):
            with self.subTest(fmt=fmt):
                self.assertExitZero([
                    "key", "export",
                    "--fingerprint", "AABBCCDD",
                    "--format", fmt,
                    "--dry-run",
                ])


class TestKeyRotate(NyxTestCase):

    def test_rotate_requires_old_fingerprint(self):
        self.assertExitNonZero(["key", "rotate", "--passphrase", "hunter2"])

    def test_rotate_dry_run_exits_zero(self):
        self.assertExitZero([
            "key", "rotate",
            "--old-fingerprint", "AABBCCDD",
            "--passphrase", "hunter2",
            "--dry-run",
        ])

    def test_rotate_creates_new_key(self):
        data = self.assertJsonKey([
            "key", "rotate",
            "--old-fingerprint", "AABBCCDD",
            "--passphrase", "hunter2",
            "--dry-run",
        ], "new_fingerprint")
        self.assertNotEqual(data["new_fingerprint"], "AABBCCDD")


if __name__ == "__main__":
    unittest.main()
