import sys
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HARNESS))
import db_state  # noqa: E402


class StableDumpTest(unittest.TestCase):
    def test_only_the_random_restrict_lines_are_removed(self):
        dump = b"\\restrict AbC123\nINSERT INTO category VALUES (1);\nSELECT pg_catalog.setval('s', 5, true);\n\\unrestrict AbC123\n"
        expected = b"INSERT INTO category VALUES (1);\nSELECT pg_catalog.setval('s', 5, true);\n"
        self.assertEqual(db_state.stable_dump(dump), expected)

    def test_two_dumps_differing_only_in_the_token_are_equal(self):
        first = b"\\restrict one\nINSERT 1;\n\\unrestrict one\n"
        second = b"\\restrict two\nINSERT 1;\n\\unrestrict two\n"
        self.assertEqual(db_state.stable_dump(first), db_state.stable_dump(second))

    def test_a_data_difference_survives(self):
        self.assertNotEqual(db_state.stable_dump(b"INSERT 1;\n"), db_state.stable_dump(b"INSERT 2;\n"))

    def test_a_blind_dump_is_never_produced_from_real_content(self):
        self.assertNotEqual(db_state.stable_dump(b"INSERT 1;\nINSERT 2;\n"), b"")


if __name__ == "__main__":
    unittest.main()
