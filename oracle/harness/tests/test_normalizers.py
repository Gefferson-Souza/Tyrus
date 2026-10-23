import json
import sys
import tempfile
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HARNESS))
import normalizers  # noqa: E402

ENTRY = {"engine": "bun-1.2.22", "normalizer": "W-BUN-LONE-SURROGATE-LEN", "failures": {"wire": ["x"]}}


class NormalizersTest(unittest.TestCase):
    def test_id_outside_the_closed_registry_aborts(self):
        with self.assertRaises(normalizers.HarnessError):
            normalizers.validate_known([dict(ENTRY, normalizer="W-NOT-IN-REGISTRY")])

    def test_missing_normalizer_aborts(self):
        with self.assertRaises(normalizers.HarnessError):
            normalizers.validate_known([{"engine": "e", "failures": {}}])

    def test_unknown_layer_aborts(self):
        with self.assertRaises(normalizers.HarnessError):
            normalizers.validate_known([dict(ENTRY, failures={"dns": ["x"]})])

    def test_entry_without_engine_aborts(self):
        with self.assertRaises(normalizers.HarnessError):
            normalizers.validate_known([{k: v for k, v in ENTRY.items() if k != "engine"}])

    def test_excused_is_scoped_to_engine_and_layer(self):
        known = normalizers.validate_known([ENTRY])
        self.assertEqual(normalizers.excused(known, "bun-1.2.22", "wire"), {"x": "W-BUN-LONE-SURROGATE-LEN"})
        self.assertEqual(normalizers.excused(known, "node", "wire"), {})
        self.assertEqual(normalizers.excused(known, "bun-1.2.22", "state"), {})

    def test_shipped_known_file_loads(self):
        known = normalizers.load_known(HARNESS / "known_divergences.json")
        self.assertGreaterEqual(len(known), 1)

    def test_load_known_rejects_a_file_with_a_foreign_id(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "known.json"
            path.write_text(json.dumps([dict(ENTRY, normalizer="W-FOREIGN")]), encoding="utf-8")
            with self.assertRaises(normalizers.HarnessError):
                normalizers.load_known(path)


if __name__ == "__main__":
    unittest.main()
