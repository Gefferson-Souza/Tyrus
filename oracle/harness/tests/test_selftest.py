import sys
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HARNESS))
import equiv_selftest  # noqa: E402
import selftest_fixtures as fx  # noqa: E402
import wire_compare  # noqa: E402


class SelftestTest(unittest.TestCase):
    def test_every_control_behaves_as_expected(self):
        failed = [name for name, ok, _ in equiv_selftest.run_all(None) if not ok]
        self.assertEqual(failed, [])

    def test_the_selftest_has_at_least_thirty_controls(self):
        self.assertGreaterEqual(len(equiv_selftest.run_all(None)), 30)

    def test_client_byte_hash_is_the_only_difference_caught_at_bytes_level(self):
        base = fx.base_trace()
        other = fx.mutated(base, equiv_selftest.patch_trip("02-product-1", 0, client_sha256=["0" * 64]))
        options = {"engine": "e", "wire_level": "bytes"}
        self.assertFalse(wire_compare.compare_traces(base, other, [], options)["green"])
        options["wire_level"] = "sql"
        self.assertTrue(wire_compare.compare_traces(base, other, [], options)["green"])

    def test_unknown_wire_level_is_a_programming_error(self):
        base = fx.base_trace()
        with self.assertRaises(ValueError):
            wire_compare.compare_wire(base, base, "everything")

    def test_normalized_needs_every_failure_excused(self):
        known = [{"engine": "e", "normalizer": "W-BUN-LONE-SURROGATE-LEN", "failures": {"wire": ["a"]}}]
        covered = wire_compare.layer_verdict(["a"], known, "e", "wire")
        partial = wire_compare.layer_verdict(["a", "b"], known, "e", "wire")
        self.assertEqual(covered["status"], wire_compare.NORMALIZED)
        self.assertEqual(partial["status"], wire_compare.DIFFERENT)
        self.assertEqual(wire_compare.layer_verdict([], known, "e", "wire")["status"], wire_compare.IDENTICAL)


if __name__ == "__main__":
    unittest.main()
