"""
Shared fixtures and helpers for the NyxForge CLI test suite.
"""
import json
import os
import shutil
import sqlite3
import subprocess
import tempfile
import unittest

# Path to the nyx binary (must be on PATH or set NYX_BIN env var)
NYX = os.environ.get("NYX_BIN", "nyx")

# Fixtures path
FIXTURES = os.path.join(os.path.dirname(__file__), "fixtures")


def run(args, input_text=None, cwd=None, env=None):
    """Run `nyx <args>` and return (returncode, stdout, stderr)."""
    cmd = [NYX] + (args if isinstance(args, list) else args.split())
    result = subprocess.run(
        cmd,
        input=input_text,
        capture_output=True,
        text=True,
        cwd=cwd,
        env=env,
    )
    return result.returncode, result.stdout, result.stderr


def run_json(args):
    """Run `nyx <args> --json` and return parsed dict/list, or raise."""
    rc, out, err = run(args + ["--json"])
    if rc != 0:
        raise AssertionError(f"nyx exited {rc}: {err.strip()}")
    return json.loads(out)


class NyxTestCase(unittest.TestCase):
    """Base class: sets up a temp working dir and tears it down."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="nyx-test-")
        self.addCleanup(shutil.rmtree, self.tmpdir, ignore_errors=True)

    def assertExitZero(self, args, msg=None):
        rc, out, err = run(args, cwd=self.tmpdir)
        self.assertEqual(rc, 0, msg or f"Expected exit 0, got {rc}.\nstdout: {out}\nstderr: {err}")
        return out

    def assertExitNonZero(self, args, msg=None):
        rc, out, err = run(args, cwd=self.tmpdir)
        self.assertNotEqual(rc, 0, msg or f"Expected non-zero exit, got 0.\nstdout: {out}")
        return rc, out, err

    def assertOutputContains(self, args, substring, msg=None):
        rc, out, err = run(args, cwd=self.tmpdir)
        combined = out + err
        self.assertIn(substring, combined,
                      msg or f"Expected '{substring}' in output.\nGot: {combined!r}")
        return out

    def assertJsonKey(self, args, key, msg=None):
        data = run_json(args + (["--cwd", self.tmpdir] if "--cwd" not in args else []))
        self.assertIn(key, data, msg or f"JSON response missing key '{key}'")
        return data
