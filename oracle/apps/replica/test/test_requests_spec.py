import json
import subprocess
import sys
import unittest
from pathlib import Path

REPLICA = Path(__file__).resolve().parent.parent
SPEC = json.loads((REPLICA / "requests.json").read_text(encoding="utf-8"))["requests"]
REQUIRED_STATUSES = {200, 201, 400, 404, 429}
GATE0_COUNT = 54


class RequestsSpecTest(unittest.TestCase):
    def test_has_at_least_sixty_requests(self):
        self.assertGreaterEqual(len(SPEC), 60)

    def test_ids_are_unique_and_numbered_in_order(self):
        ids = [r["id"] for r in SPEC]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertEqual([int(i.split("-")[0]) for i in ids], list(range(1, len(ids) + 1)))

    def test_status_set_covers_the_milestone_classes(self):
        self.assertTrue(REQUIRED_STATUSES <= {r["expect_status"] for r in SPEC})

    def test_keeps_the_gate0_block_first(self):
        gate0 = SPEC[:GATE0_COUNT]
        self.assertEqual(gate0[0]["id"], "01-health")
        self.assertEqual(gate0[-1]["id"], "54-post-fk-500")
        self.assertTrue(all(r["endpoint"] in {"E1", "E2", "E3", "E4", "E5", "E6"} for r in gate0))

    def test_covers_malformed_json_and_router_404(self):
        malformed = [r for r in SPEC if r["endpoint"] == "MJ"]
        self.assertGreaterEqual(len(malformed), 3)
        self.assertTrue(all(r["expect_status"] == 400 and "body_raw" in r for r in malformed))
        self.assertTrue(any(r["endpoint"] == "NF" and r["expect_status"] == 404 for r in SPEC))

    def test_throttled_429_requests_declare_the_volatile_retry_after(self):
        blocked = [r for r in SPEC if r["expect_status"] == 429]
        self.assertGreaterEqual(len(blocked), 2)
        self.assertTrue(all("retry-after" in r["volatile_headers"] for r in blocked))

    def test_successful_posts_request_timestamp_masking(self):
        created = [r for r in SPEC if r["method"] == "POST" and r["expect_status"] == 201]
        self.assertGreaterEqual(len(created), 10)
        self.assertTrue(all(r.get("mask_timestamps") for r in created))

    def test_committed_file_is_what_the_generator_produces(self):
        result = subprocess.run([sys.executable, "-I", "-B", str(REPLICA / "make_requests.py"), "--check"])
        self.assertEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
