import hashlib
import json
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
HEX64 = set("0123456789abcdef")


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class ProvenanceTest(unittest.TestCase):
    def setUp(self):
        self.entries = json.loads((HARNESS / "PROVENANCE.json").read_text(encoding="utf-8"))["files"]

    def test_every_imported_file_has_an_origin_hash(self):
        for entry in self.entries:
            self.assertEqual(len(entry["origin_sha256"]), 64, entry["path"])
            self.assertTrue(set(entry["origin_sha256"]) <= HEX64, entry["path"])

    def test_recorded_hash_matches_the_file_in_the_repository(self):
        for entry in self.entries:
            self.assertEqual(sha256_of(HARNESS / entry["path"]), entry["sha256"], entry["path"])

    def test_verbatim_files_keep_the_origin_hash(self):
        verbatim = [e for e in self.entries if e["mode"] == "verbatim"]
        self.assertGreaterEqual(len(verbatim), 1)
        for entry in verbatim:
            self.assertEqual(entry["sha256"], entry["origin_sha256"], entry["path"])

    def test_the_five_wire_origins_are_all_declared(self):
        origins = {e["origin"] for e in self.entries}
        for name in ("harness/wire.py", "tools/pgproxy.py", "harness/capture_sql.py",
                     "harness/known_divergences.json", "harness/equiv_selftest.py"):
            self.assertIn(name, origins)

    def test_equiv_core_and_request_generator_are_not_imported(self):
        for forbidden in ("equiv.py", "equiv_core.py", "make_requests.py"):
            self.assertFalse((HARNESS / forbidden).exists(), forbidden)


if __name__ == "__main__":
    unittest.main()
