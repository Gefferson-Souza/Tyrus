import asyncio
import hashlib
import socket
import struct
import sys
import threading
import time
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HARNESS))
import pgproxy  # noqa: E402
from wire import trips_of  # noqa: E402


def free_port() -> int:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


class FakeUpstream:
    """Servidor mínimo: lê o que o proxy encaminha e responde só ReadyForQuery."""

    def __init__(self):
        self.port = free_port()
        self.received = bytearray()
        self.ready = threading.Event()
        self.loop = asyncio.new_event_loop()
        self.thread = threading.Thread(target=self._run, daemon=True)

    async def _handle(self, reader, writer):
        try:
            while data := await reader.read(4096):
                self.received += data
                writer.write(b"Z" + struct.pack("!i", 5) + b"I")
                await writer.drain()
        finally:
            writer.close()

    def _run(self):
        asyncio.set_event_loop(self.loop)
        self.server = self.loop.run_until_complete(asyncio.start_server(self._handle, "127.0.0.1", self.port))
        self.ready.set()
        self.loop.run_forever()

    def start(self):
        self.thread.start()
        self.ready.wait(5)

    def stop(self):
        async def shutdown():
            self.server.close()
            pending = [t for t in asyncio.all_tasks() if t is not asyncio.current_task()]
            for task in pending:
                task.cancel()
            await asyncio.gather(*pending, return_exceptions=True)
            self.loop.stop()

        asyncio.run_coroutine_threadsafe(shutdown(), self.loop)
        self.thread.join(5)


def startup_packet() -> bytes:
    body = struct.pack("!i", 196608) + b"user\x00postgres\x00\x00"
    return struct.pack("!i", len(body) + 4) + body


def query_message(sql: str) -> bytes:
    payload = sql.encode() + b"\x00"
    return b"Q" + struct.pack("!i", len(payload) + 4) + payload


def wait_until(condition, timeout_s: float = 5.0) -> None:
    deadline = time.monotonic() + timeout_s
    while not condition():
        if time.monotonic() > deadline:
            raise AssertionError("condição não atendida no prazo")
        time.sleep(0.01)


def send_through(proxy: pgproxy.WireProxy, upstream: FakeUpstream, messages: list[bytes]) -> None:
    """Só retorna quando o upstream recebeu todos os bytes: o primeiro `Z` responde ao startup, não ao `Q`."""
    expected_len = sum(len(message) for message in messages)
    with socket.create_connection(("127.0.0.1", proxy.listen_port), timeout=5) as client:
        for message in messages:
            client.sendall(message)
        wait_until(lambda: len(upstream.received) >= expected_len)


class WireProxyTest(unittest.TestCase):
    def setUp(self):
        self.upstream = FakeUpstream()
        self.upstream.start()
        self.addCleanup(self.upstream.stop)

    def run_proxy(self, rewrite=None):
        proxy = pgproxy.WireProxy(free_port(), self.upstream.port, rewrite=rewrite)
        proxy.start()
        self.addCleanup(proxy.stop)
        return proxy

    def test_frontend_message_carries_the_sha256_of_its_exact_bytes(self):
        proxy = self.run_proxy()
        message = query_message("SELECT 1")
        send_through(proxy, self.upstream, [startup_packet(), message])
        events = [e for e in proxy.events if e["dir"] == "F" and e["type"] == "Q"]
        self.assertEqual(events[0]["wire_sha256"], hashlib.sha256(message).hexdigest())
        self.assertEqual(trips_of(proxy.events)[0]["sql"], "SELECT 1")

    def test_rewrite_hook_changes_what_is_recorded_and_forwarded(self):
        proxy = self.run_proxy(rewrite=lambda kind, payload: payload.replace(b"SELECT", b"select"))
        send_through(proxy, self.upstream, [startup_packet(), query_message("SELECT 1")])
        recorded = [e for e in proxy.events if e["dir"] == "F" and e["type"] == "Q"][0]
        self.assertEqual(recorded["sql"], "select 1")
        self.assertIn(b"select 1", bytes(self.upstream.received))
        self.assertNotIn(b"SELECT 1", bytes(self.upstream.received))

    def test_without_rewrite_the_bytes_are_forwarded_untouched(self):
        proxy = self.run_proxy()
        send_through(proxy, self.upstream, [startup_packet(), query_message("SELECT 1")])
        self.assertIn(query_message("SELECT 1"), bytes(self.upstream.received))

    def test_password_messages_are_not_recorded_nor_hashed(self):
        proxy = self.run_proxy()
        password = b"p" + struct.pack("!i", 9) + b"abcd\x00"
        send_through(proxy, self.upstream, [startup_packet(), password])
        events = [e for e in proxy.events if e["type"] == "p"]
        self.assertEqual(events[0].get("note"), "password/SASL message not recorded")
        self.assertNotIn("wire_sha256", events[0])


if __name__ == "__main__":
    unittest.main()
