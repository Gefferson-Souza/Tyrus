import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
SCRIPT = REPO / "scripts" / "check_replica_parity.sh"
REPLICA = REPO / "oracle" / "apps" / "replica"
ROUTE_FILE = "src/products/products.controller.ts"


def run_script(replica_dir: Path, **extra_env: str) -> subprocess.CompletedProcess:
    env = {"PATH": os.environ["PATH"], "REPLICA_DIR": str(replica_dir), **extra_env}
    return subprocess.run(["sh", str(SCRIPT)], env=env, capture_output=True, text=True)


class ParityScriptTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="replica-parity-"))
        self.copy = self.tmp / "replica"
        shutil.copytree(REPLICA, self.copy, ignore=shutil.ignore_patterns("node_modules", "dist", "__pycache__"))

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_pristine_copy_passes(self):
        self.assertEqual(run_script(self.copy).returncode, 0)

    def test_changing_one_byte_of_a_route_file_fails_and_names_it(self):
        with open(self.copy / ROUTE_FILE, "a", encoding="utf-8") as handle:
            handle.write("// x\n")
        result = run_script(self.copy)
        self.assertEqual(result.returncode, 1)
        self.assertIn(ROUTE_FILE, result.stdout + result.stderr)

    def test_removing_a_route_file_fails(self):
        (self.copy / ROUTE_FILE).unlink()
        self.assertEqual(run_script(self.copy).returncode, 1)

    def test_missing_manifest_fails(self):
        (self.copy / "gate0-parity.sha256").unlink()
        self.assertEqual(run_script(self.copy).returncode, 1)

    def test_origin_comparison_catches_a_manifest_that_was_regenerated(self):
        origin = self.tmp / "gate0"
        (origin / "nest-app").mkdir(parents=True)
        shutil.copytree(self.copy / "src", origin / "nest-app" / "src")
        shutil.copytree(self.copy / "db", origin / "db")
        self.assertEqual(run_script(self.copy, TYRUS_GATE0_DIR=str(origin)).returncode, 0)
        with open(origin / "nest-app" / ROUTE_FILE, "a", encoding="utf-8") as handle:
            handle.write("// origem mudou\n")
        self.assertEqual(run_script(self.copy, TYRUS_GATE0_DIR=str(origin)).returncode, 1)


if __name__ == "__main__":
    unittest.main()
