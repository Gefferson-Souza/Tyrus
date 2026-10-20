import sys
import unittest
from pathlib import Path

REPLICA = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPLICA / "harness"))
import replica_core as core  # noqa: E402

HEALTH = {"id": "01-health", "expect_status": 200}
POST_201 = {"id": "45-post", "expect_status": 201, "mask_timestamps": True}
THROTTLED = {"id": "75-throttled", "expect_status": 429, "volatile_headers": {"retry-after": [1, 60]}}
STAMP = b'"createdAt":"2026-10-20T10:00:00.123Z","updatedAt":"2026-10-20T10:00:00.123Z"'


def record(request, status, headers, body):
    return core.build_record(request, status, headers, body)


class WeakEtagTest(unittest.TestCase):
    def test_matches_the_value_express_sent_for_health(self):
        self.assertEqual(core.weak_etag(b'{"status":"ok"}'), 'W/"f-VaSQ4oDUiZblZNAEkkN+sX+q3Sg"')

    def test_length_is_hexadecimal(self):
        self.assertTrue(core.weak_etag(b"x" * 255).startswith('W/"ff-'))


class MaskTimestampsTest(unittest.TestCase):
    def test_masks_both_fields_and_nothing_else(self):
        masked = core.mask_timestamps(b'{"id":1,' + STAMP + b'}')
        self.assertEqual(masked, b'{"id":1,"createdAt":"<TS>","updatedAt":"<TS>"}')

    def test_leaves_other_iso_strings_alone(self):
        body = b'{"from":"2026-03-01T00:00:00.000Z"}'
        self.assertEqual(core.mask_timestamps(body), body)


class BuildRecordTest(unittest.TestCase):
    def test_drops_date_connection_keep_alive_and_lowercases_names(self):
        headers = [("X-Powered-By", "Express"), ("Date", "x"), ("Connection", "keep-alive"), ("Keep-Alive", "timeout=5")]
        result = record(HEALTH, 200, headers, b"{}")
        self.assertEqual(result["headers"], [["x-powered-by", "Express"]])

    def test_flags_unexpected_status(self):
        result = record(HEALTH, 500, [], b"{}")
        self.assertEqual(len(result["problems"]), 1)

    def test_flags_etag_that_does_not_match_the_body(self):
        result = record(HEALTH, 200, [("ETag", 'W/"f-AAAA"')], b'{"status":"ok"}')
        self.assertEqual(result["problems"], ["etag não bate com o corpo"])

    def test_masked_response_checks_etag_on_raw_body_then_drops_it(self):
        raw = b'{"id":1,' + STAMP + b"}"
        result = record(POST_201, 201, [("ETag", core.weak_etag(raw)), ("Content-Length", str(len(raw)))], raw)
        self.assertEqual(result["problems"], [])
        self.assertEqual(result["headers"], [["content-length", str(len(raw))]])
        self.assertIn("<TS>", result["body"])

    def test_masking_only_applies_when_the_request_asks_for_it(self):
        raw = b'{"id":1,' + STAMP + b"}"
        result = record(HEALTH, 200, [], raw)
        self.assertIn("2026-10-20", result["body"])

    def test_volatile_header_is_replaced_after_range_check(self):
        result = record(THROTTLED, 429, [("Retry-After", "60")], b"{}")
        self.assertEqual(result["headers"], [["retry-after", "<volatile>"]])
        self.assertEqual(result["problems"], [])

    def test_volatile_header_out_of_range_is_a_problem(self):
        result = record(THROTTLED, 429, [("Retry-After", "600")], b"{}")
        self.assertEqual(len(result["problems"]), 1)

    def test_missing_volatile_header_is_a_problem(self):
        result = record(THROTTLED, 429, [], b"{}")
        self.assertEqual(len(result["problems"]), 1)


class DiffRunsTest(unittest.TestCase):
    def setUp(self):
        self.base = [record(HEALTH, 200, [("X-Powered-By", "Express")], b"{}")]

    def test_identical_runs_have_no_diff(self):
        self.assertEqual(core.diff_runs(self.base, list(self.base), "strict"), [])

    def test_body_change_is_a_diff_at_both_levels(self):
        other = [record(HEALTH, 200, [("X-Powered-By", "Express")], b"[]")]
        self.assertEqual(len(core.diff_runs(self.base, other, "body")), 1)
        self.assertEqual(len(core.diff_runs(self.base, other, "strict")), 1)

    def test_header_change_is_a_diff_only_in_strict(self):
        other = [record(HEALTH, 200, [], b"{}")]
        self.assertEqual(core.diff_runs(self.base, other, "body"), [])
        self.assertEqual(len(core.diff_runs(self.base, other, "strict")), 1)

    def test_status_change_is_a_diff(self):
        other = [record(HEALTH, 201, [("X-Powered-By", "Express")], b"{}")]
        self.assertTrue(any("status" in line for line in core.diff_runs(self.base, other, "body")))

    def test_different_request_sets_are_refused(self):
        other = [record({"id": "02-other", "expect_status": 200}, 200, [], b"{}")]
        self.assertEqual(core.diff_runs(self.base, other, "strict"), ["conjuntos de requisições diferentes"])

    def test_unknown_level_raises(self):
        with self.assertRaises(ValueError):
            core.diff_runs(self.base, self.base, "loose")


if __name__ == "__main__":
    unittest.main()
