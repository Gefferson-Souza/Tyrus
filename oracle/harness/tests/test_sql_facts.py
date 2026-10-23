import sys
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HARNESS))
import sql_facts  # noqa: E402


class PredictorsTest(unittest.TestCase):
    def test_key_order_follows_entity_order_not_request_order(self):
        body = {"categoryId": 1, "name": "n", "sku": "s", "price": 2, "stock": 3}
        self.assertEqual(sql_facts.predict_key_order(body),
                         ["sku", "name", "price", "stock", "categoryId", "description", "id", "active",
                          "createdAt", "updatedAt"])

    def test_insert_uses_default_for_absent_description_and_active(self):
        sql, params = sql_facts.predict_insert({"sku": "s", "name": "n", "price": 100.0, "stock": 1, "categoryId": 4})
        self.assertIn("VALUES ($1, $2, DEFAULT, $3, $4, DEFAULT, $5, DEFAULT, DEFAULT)", sql)
        self.assertEqual(params, ["s", "n", "100", "1", "4"])

    def test_null_description_is_bound_as_null(self):
        _, params = sql_facts.predict_insert({"sku": "s", "name": "n", "description": None, "price": 1, "stock": 1,
                                              "categoryId": 1, "active": False})
        self.assertEqual(params[2], None)
        self.assertEqual(params[5], "false")

    def test_normalize_sql_extracts_inlined_literals(self):
        sql, literals = sql_facts.normalize_sql('SELECT x LIMIT 5 OFFSET 37 WHERE id IN (1, 2, 3)')
        self.assertEqual(literals, {"limit": 5, "offset": 37, "ids": "1, 2, 3"})
        self.assertIn("LIMIT <limit> OFFSET <offset>", sql)


if __name__ == "__main__":
    unittest.main()
