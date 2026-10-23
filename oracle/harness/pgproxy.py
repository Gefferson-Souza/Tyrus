"""Decoding PostgreSQL wire proxy (protocol v3). Sits between the Nest app and Postgres and records every
frontend/backend message, so the SQL a driver really sends is read from the WIRE, not from ORM logs.

Usage as a module:  proxy = WireProxy(listen_port, upstream_port); proxy.start(); ...; proxy.events; proxy.stop()
Passwords (type 'p' messages) are never recorded.
Adaptations: every decoded frontend message also carries `wire_sha256` (digest of its exact bytes); the optional
`rewrite` hook mutates Q/P messages (negative control); `stop()` cancels the pending tasks and joins the thread.
"""
from __future__ import annotations

import asyncio
import hashlib
import struct
import threading
import time
from collections.abc import Callable


def _cstr(buf: bytes, pos: int) -> tuple[str, int]:
    end = buf.index(b"\x00", pos)
    return buf[pos:end].decode("utf-8", "replace"), end + 1


def decode_frontend(kind: str, payload: bytes) -> dict:
    ev: dict = {"type": kind}
    if kind == "Q":
        ev["sql"] = _cstr(payload, 0)[0]
    elif kind == "P":
        name, pos = _cstr(payload, 0)
        sql, pos = _cstr(payload, pos)
        (n,) = struct.unpack_from("!h", payload, pos); pos += 2
        oids = list(struct.unpack_from(f"!{n}i", payload, pos)) if n else []
        ev.update(stmt=name, sql=sql, param_oids=oids)
    elif kind == "B":
        portal, pos = _cstr(payload, 0)
        stmt, pos = _cstr(payload, pos)
        (nf,) = struct.unpack_from("!h", payload, pos); pos += 2
        fmts = list(struct.unpack_from(f"!{nf}h", payload, pos)) if nf else []; pos += 2 * nf
        (np_,) = struct.unpack_from("!h", payload, pos); pos += 2
        params, params_hex = [], {}
        for i in range(np_):
            (ln,) = struct.unpack_from("!i", payload, pos); pos += 4
            if ln == -1:
                params.append(None)
            else:
                raw = payload[pos:pos + ln]
                params.append(raw.decode("utf-8", "replace"))
                if any(b > 127 for b in raw):
                    params_hex[i] = {"declared_length": ln, "bytes": raw.hex()}
                pos += ln
        (nr,) = struct.unpack_from("!h", payload, pos); pos += 2
        rfmts = list(struct.unpack_from(f"!{nr}h", payload, pos)) if nr else []
        ev.update(portal=portal, stmt=stmt, param_formats=fmts, params=params, result_formats=rfmts)
        if params_hex:
            ev["non_ascii_params_raw"] = params_hex
    elif kind in ("D", "C"):
        target = chr(payload[0])
        ev.update(target={"S": "statement", "P": "portal"}.get(target, target), name=_cstr(payload, 1)[0])
    elif kind == "E":
        portal, pos = _cstr(payload, 0)
        (maxrows,) = struct.unpack_from("!i", payload, pos)
        ev.update(portal=portal, max_rows=maxrows)
    return ev


def decode_backend(kind: str, payload: bytes) -> dict:
    ev: dict = {"type": kind}
    if kind == "Z":
        ev["tx_status"] = chr(payload[0])
    elif kind == "C":
        ev["tag"] = _cstr(payload, 0)[0]
    elif kind == "D":
        (n,) = struct.unpack_from("!h", payload, 0)
        ev["columns"] = n
    elif kind == "T":
        (n,) = struct.unpack_from("!h", payload, 0)
        pos, cols = 2, []
        for _ in range(n):
            name, pos = _cstr(payload, pos)
            _tbl, _attr, oid, _sz, _mod, fmt = struct.unpack_from("!ihihih", payload, pos); pos += 18
            cols.append({"name": name, "oid": oid, "format": fmt})
        ev["row_description"] = cols
    elif kind in ("E", "N"):
        fields, pos = {}, 0
        while pos < len(payload) and payload[pos] != 0:
            code = chr(payload[pos]); pos += 1
            val, pos = _cstr(payload, pos)
            fields[code] = val
        ev["fields"] = {k: fields[k] for k in ("S", "C", "M", "D", "n", "t", "c") if k in fields}
    elif kind == "t":
        (n,) = struct.unpack_from("!h", payload, 0)
        ev["param_oids"] = list(struct.unpack_from(f"!{n}i", payload, 2)) if n else []
    return ev


class WireProxy:
    def __init__(self, listen_port: int, upstream_port: int, host: str = "127.0.0.1",
                 rewrite: Callable[[str, bytes], bytes] | None = None):
        self.listen_port, self.upstream_port, self.host = listen_port, upstream_port, host
        self.rewrite = rewrite  # controle negativo: reescreve Q/P antes de gravar e de encaminhar
        self.events: list[dict] = []
        self._lock = threading.Lock()
        self._conn_seq = 0
        self._loop: asyncio.AbstractEventLoop | None = None
        self._thread: threading.Thread | None = None
        self._server: asyncio.base_events.Server | None = None
        self._ready = threading.Event()

    def _record(self, ev: dict) -> None:
        ev["t"] = time.monotonic()
        with self._lock:
            ev["seq"] = len(self.events)
            self.events.append(ev)

    def mark(self) -> int:
        with self._lock:
            return len(self.events)

    def since(self, mark: int) -> list[dict]:
        with self._lock:
            return list(self.events[mark:])

    async def _pipe_frontend(self, conn: int, reader, writer) -> None:
        # Startup packet: int32 length, no type byte.
        head = await reader.readexactly(4)
        (length,) = struct.unpack("!i", head)
        rest = await reader.readexactly(length - 4)
        (code,) = struct.unpack_from("!i", rest, 0)
        parts = rest[4:].split(b"\x00")
        params = {parts[i].decode(): parts[i + 1].decode() for i in range(0, len(parts) - 2, 2) if parts[i]}
        self._record({"conn": conn, "dir": "F", "type": "startup", "protocol": code, "params": params})
        writer.write(head + rest)
        await writer.drain()
        while True:
            hdr = await reader.readexactly(5)
            kind = chr(hdr[0])
            (ln,) = struct.unpack("!i", hdr[1:])
            payload = await reader.readexactly(ln - 4) if ln > 4 else b""
            if self.rewrite is not None and kind in ("Q", "P"):
                payload = self.rewrite(kind, payload)
                hdr = hdr[:1] + struct.pack("!i", len(payload) + 4)
            if kind != "p":
                try:
                    ev = decode_frontend(kind, payload)
                except Exception as exc:  # keep proxying even when a message cannot be decoded
                    ev = {"type": kind, "decode_error": repr(exc)}
                ev["wire_sha256"] = hashlib.sha256(hdr + payload).hexdigest()
            else:
                ev = {"type": "p", "note": "password/SASL message not recorded"}
            ev.update(conn=conn, dir="F")
            self._record(ev)
            writer.write(hdr + payload)
            await writer.drain()
            if kind == "X":
                return

    async def _pipe_backend(self, conn: int, reader, writer) -> None:
        while True:
            hdr = await reader.readexactly(5)
            kind = chr(hdr[0])
            (ln,) = struct.unpack("!i", hdr[1:])
            payload = await reader.readexactly(ln - 4) if ln > 4 else b""
            if kind != "R":
                try:
                    ev = decode_backend(kind, payload)
                except Exception as exc:
                    ev = {"type": kind, "decode_error": repr(exc)}
                ev.update(conn=conn, dir="B")
                self._record(ev)
            writer.write(hdr + payload)
            await writer.drain()

    async def _handle(self, c_reader, c_writer) -> None:
        self._conn_seq += 1
        conn = self._conn_seq
        u_reader, u_writer = await asyncio.open_connection(self.host, self.upstream_port)
        tasks = [
            asyncio.ensure_future(self._pipe_frontend(conn, c_reader, u_writer)),
            asyncio.ensure_future(self._pipe_backend(conn, u_reader, c_writer)),
        ]
        try:
            await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
        finally:
            for t in tasks:
                t.cancel()
            for w in (c_writer, u_writer):
                w.close()

    def start(self) -> None:
        def run() -> None:
            self._loop = asyncio.new_event_loop()
            asyncio.set_event_loop(self._loop)
            self._server = self._loop.run_until_complete(
                asyncio.start_server(self._handle, self.host, self.listen_port))
            self._ready.set()
            self._loop.run_forever()

        self._thread = threading.Thread(target=run, daemon=True)
        self._thread.start()
        if not self._ready.wait(10):
            raise RuntimeError("wire proxy did not start")

    def stop(self) -> None:
        if self._loop is None:
            return

        async def shutdown() -> None:
            if self._server:
                self._server.close()
            pending = [t for t in asyncio.all_tasks() if t is not asyncio.current_task()]
            for task in pending:
                task.cancel()
            await asyncio.gather(*pending, return_exceptions=True)
            self._loop.stop()

        asyncio.run_coroutine_threadsafe(shutdown(), self._loop)
        if self._thread:
            self._thread.join(10)
