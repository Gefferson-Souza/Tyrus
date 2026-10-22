import json
import os
import socket
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPLICA = Path(__file__).resolve().parent.parent
RUNNER = REPLICA / "run_requests.py"
STUB = REPLICA / "test" / "support" / "stub_target.py"
SPEC = {"requests": [{"id": "01-x", "endpoint": "E1", "method": "GET", "path": "/x", "expect_status": 200}]}


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


class RunRequestsCliTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="replica-cli-"))
        self.spec = self.tmp / "requests.json"
        self.spec.write_text(json.dumps(SPEC), encoding="utf-8")

    def run_cli(self, mode: str, *extra: str, target: str | None = None) -> subprocess.CompletedProcess:
        env = {"PATH": os.environ["PATH"], "HOME": os.environ["HOME"]}
        command = [sys.executable, "-I", "-B", str(RUNNER), "run", "--target", target or f"{sys.executable} {STUB} {mode}",
                   "--reset-cmd", "true", "--requests", str(self.spec), "--out", str(self.tmp / "out.json"),
                   "--port", str(free_port()), *extra]
        return subprocess.run(command, env=env, capture_output=True, text=True, timeout=60)

    def test_stable_target_passes_twice_and_writes_the_report(self):
        result = self.run_cli("ok", "--twice")
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads((self.tmp / "out.json").read_text(encoding="utf-8"))
        self.assertEqual(report["meta"]["requests"], 1)
        self.assertEqual(report["results"][0]["status"], 200)

    def test_target_whose_body_changes_between_runs_fails_with_the_request_id(self):
        result = self.run_cli("drift", "--twice")
        self.assertEqual(result.returncode, 1)
        self.assertIn("01-x: corpo diferente", result.stderr)

    def test_unexpected_status_fails(self):
        result = self.run_cli("wrong-status")
        self.assertEqual(result.returncode, 1)
        self.assertIn("status 500 diferente do esperado 200", result.stderr)

    def test_wrong_runtime_version_is_refused_before_any_request(self):
        result = self.run_cli("ok", "--expect-node", "v0.0.0")
        self.assertEqual(result.returncode, 2)
        self.assertIn("esperado v0.0.0", result.stderr)

    def engine_without_version_flag(self) -> str:
        script = self.tmp / "engine.sh"
        script.write_text(f'#!/bin/sh\nif [ "$1" = "--version" ]; then exit 64; fi\nexec {sys.executable} {STUB} ok\n', encoding="utf-8")
        script.chmod(0o755)
        return str(script)

    def test_target_without_a_version_flag_needs_only_an_engine_id(self):
        result = self.run_cli("ok", "--engine-id", "fake-engine/1", target=self.engine_without_version_flag())
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads((self.tmp / "out.json").read_text(encoding="utf-8"))
        self.assertEqual(report["meta"]["node"], "fake-engine/1")

    def test_expect_node_on_an_engine_without_version_flag_is_a_clean_error(self):
        result = self.run_cli("ok", "--expect-node", "v26.11.1", target=self.engine_without_version_flag())
        self.assertEqual(result.returncode, 2)
        self.assertIn("sem resposta a --version", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_golden_must_match_the_run(self):
        self.assertEqual(self.run_cli("ok").returncode, 0)
        golden = self.tmp / "out.json"
        self.assertEqual(self.run_cli("ok", "--golden", str(golden)).returncode, 0)
        changed = json.loads(golden.read_text(encoding="utf-8"))
        changed["results"][0]["body_sha256"] = "0" * 64
        (self.tmp / "golden.json").write_text(json.dumps(changed), encoding="utf-8")
        result = self.run_cli("ok", "--golden", str(self.tmp / "golden.json"))
        self.assertEqual(result.returncode, 1)
        self.assertIn("golden: 01-x: corpo diferente", result.stderr)

    def test_compare_subcommand_detects_a_changed_report(self):
        self.assertEqual(self.run_cli("ok").returncode, 0)
        other = json.loads((self.tmp / "out.json").read_text(encoding="utf-8"))
        other["results"][0]["body_sha256"] = "0" * 64
        (self.tmp / "other.json").write_text(json.dumps(other), encoding="utf-8")
        same = subprocess.run([sys.executable, "-I", "-B", str(RUNNER), "compare", str(self.tmp / "out.json"),
                               str(self.tmp / "out.json")])
        changed = subprocess.run([sys.executable, "-I", "-B", str(RUNNER), "compare", str(self.tmp / "out.json"),
                                  str(self.tmp / "other.json")], capture_output=True)
        self.assertEqual((same.returncode, changed.returncode), (0, 1))


if __name__ == "__main__":
    unittest.main()
