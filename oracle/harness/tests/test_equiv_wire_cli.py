import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HARNESS))
import selftest_fixtures as fx  # noqa: E402

CLI = HARNESS / "equiv_wire.py"


def run_compare(golden: dict, actual: dict, *extra: str) -> subprocess.CompletedProcess:
    with tempfile.TemporaryDirectory() as tmp:
        paths = []
        for name, trace in (("golden", golden), ("actual", actual)):
            path = Path(tmp) / f"{name}.json"
            path.write_text(json.dumps(trace), encoding="utf-8")
            paths.append(str(path))
        return subprocess.run([sys.executable, "-I", "-B", str(CLI), "compare", *paths, *extra],
                              capture_output=True, text=True, check=False)


class CompareCommandTest(unittest.TestCase):
    def test_identical_traces_exit_zero(self):
        base = fx.base_trace()
        done = run_compare(base, base)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertIn("wire: IDENTICAL", done.stdout)

    def test_a_rewritten_select_exits_one_and_names_the_layer(self):
        base = fx.base_trace()
        other = fx.mutated(base, lambda t: t["requests"]["02-product-1"]["trips"][0].update(sql="select 1"))
        done = run_compare(base, other)
        self.assertEqual(done.returncode, 1)
        self.assertIn("wire: DIFFERENT", done.stdout)
        self.assertIn("http: IDENTICAL", done.stdout)

    def test_known_file_with_foreign_id_exits_two(self):
        base = fx.base_trace()
        with tempfile.TemporaryDirectory() as tmp:
            known = Path(tmp) / "known.json"
            known.write_text(json.dumps([{"engine": "e", "normalizer": "W-FOREIGN", "failures": {}}]), encoding="utf-8")
            done = run_compare(base, base, "--known", str(known))
        self.assertEqual(done.returncode, 2)
        self.assertIn("fora do registro fechado", done.stderr)

    def test_state_difference_exits_one(self):
        base = fx.base_trace()
        other = fx.mutated(base, lambda t: t["state"].update(dump_sha256="c" * 64))
        done = run_compare(base, other)
        self.assertEqual(done.returncode, 1)
        self.assertIn("state: DIFFERENT", done.stdout)

    def test_capture_message_count_is_reported(self):
        base = fx.base_trace()
        done = run_compare(base, base)
        self.assertIn("mensagens do cliente", done.stdout)


class TraceOptionsTest(unittest.TestCase):
    def record(self, *extra: str) -> subprocess.CompletedProcess:
        return subprocess.run([sys.executable, "-I", "-B", str(CLI), "record", "--target-cmd", "true", "--out", "/dev/null",
                               *extra], capture_output=True, text=True, check=False)

    def test_a_container_outside_the_tyrus_prefix_exits_two_before_touching_docker(self):
        done = self.record("--container", "docker-database-cloud-1")
        self.assertEqual(done.returncode, 2)
        self.assertIn("tyrus-", done.stderr)

    def test_an_owner_port_exits_two(self):
        for flag in ("--port", "--proxy-port", "--upstream-port"):
            with self.subTest(flag=flag):
                done = self.record(flag, "3001")
                self.assertEqual(done.returncode, 2)
                self.assertIn("3000 a 3002", done.stderr)


if __name__ == "__main__":
    unittest.main()
