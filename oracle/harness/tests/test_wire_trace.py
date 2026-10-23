import argparse
import http.server
import sys
import threading
import unittest
from unittest import mock
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HARNESS))
import wire_trace  # noqa: E402


def options(**overrides) -> argparse.Namespace:
    values = {"container": "tyrus-wire-pg", "port": 18101, "proxy_port": 55621, "upstream_port": 55521}
    return argparse.Namespace(**{**values, **overrides})


class ValidateOptionsTest(unittest.TestCase):
    def test_defaults_are_accepted(self):
        wire_trace.validate_options(options())

    def test_a_container_outside_the_tyrus_prefix_is_refused(self):
        with self.assertRaisesRegex(RuntimeError, "tyrus-"):
            wire_trace.validate_options(options(container="docker-database-central-pdv-1"))

    def test_each_owner_port_is_refused_on_every_port_option(self):
        for flag in ("port", "proxy_port", "upstream_port"):
            for owner_port in (3000, 3001, 3002):
                with self.subTest(flag=flag, owner_port=owner_port):
                    with self.assertRaisesRegex(RuntimeError, "3000 a 3002"):
                        wire_trace.validate_options(options(**{flag: owner_port}))

    def test_the_ports_next_to_the_owner_range_are_accepted(self):
        wire_trace.validate_options(options(port=2999))
        wire_trace.validate_options(options(port=3003))


class ResetCommandTest(unittest.TestCase):
    def test_reset_targets_the_same_container_and_port_as_the_proxy_and_the_dump(self):
        command = wire_trace.reset_command(options(container="tyrus-other", upstream_port=55530))
        self.assertIn("WIRE_PG_CONTAINER=tyrus-other", command)
        self.assertIn("WIRE_PG_PORT=55530", command)
        self.assertTrue(command.endswith("pg.sh reset"))


class Echo(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("content-length", "2")
        self.end_headers()
        self.wfile.write(b"ok")

    def log_message(self, *args):
        return


class SendThroughProxyTest(unittest.TestCase):
    def test_each_request_gets_only_the_events_after_its_own_mark(self):
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Echo)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        self.addCleanup(server.shutdown)

        class CountingProxy:
            def __init__(self):
                self.events: list[str] = []
                self.marks: list[int] = []

            def mark(self):
                self.marks.append(len(self.events))
                self.events.append("event")
                return self.marks[-1]

            def since(self, mark):
                return self.events[mark:]

        proxy = CountingProxy()
        requests = [{"id": f"r{i}", "method": "GET", "path": "/", "expect_status": 200} for i in range(3)]
        with mock.patch.object(wire_trace, "trips_of", lambda events: [{"seen": len(events)}]), \
                mock.patch.object(wire_trace, "summarize", lambda trip: trip):
            traced = wire_trace.send_through_proxy(server.server_address[1], proxy, requests)
        self.assertEqual(list(traced), ["r0", "r1", "r2"])
        self.assertEqual([t["http"]["status"] for t in traced.values()], [200, 200, 200])
        self.assertEqual(proxy.marks, [0, 1, 2])
        self.assertEqual([t["trips"] for t in traced.values()], [[{"seen": 1}]] * 3)


if __name__ == "__main__":
    unittest.main()
