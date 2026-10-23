#!/usr/bin/env python3
"""Executa requests.json contra um alvo (comando que sobe a réplica) e compara execuções.

Uso (sempre com `python3 -I`):
  run_requests.py run --target "node dist/bundle/main.js" --out run.json [--twice] [--golden g.json] [--adapter express|fastify]
  run_requests.py burst --target "node dist/bundle/main.js" --path-template "/products/{n}" --count 1000
  run_requests.py compare a.json b.json [--level strict|body]
"""
from __future__ import annotations

import argparse
import collections
import contextlib
import fcntl
import http.client
import json
import os
import shlex
import socket
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Callable, Iterator

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE / "harness"))
import replica_core  # noqa: E402

DEFAULT_DATABASE_URL = "postgres://postgres:replica@127.0.0.1:55493/replica"
STARTUP_TIMEOUT_S = 30.0
REQUEST_TIMEOUT_S = 30.0


def lock_path() -> Path:
    base = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share")) / "tyrus-replica"
    base.mkdir(parents=True, exist_ok=True)
    return base / "db.lock"


def engine_id(args: argparse.Namespace, target: list[str]) -> str:
    """`--engine-id` vence; a sonda `--version` só roda com `--expect-node` e nunca derruba o runner."""
    if args.engine_id:
        return args.engine_id
    if not args.expect_node:
        return "unspecified"
    probe = subprocess.run([target[0], "--version"], capture_output=True, text=True, check=False, timeout=10)
    return probe.stdout.strip() if probe.returncode == 0 else ""


def server_env(args: argparse.Namespace) -> dict[str, str]:
    return {
        "PATH": os.environ.get("PATH", ""),
        "HOME": os.environ.get("HOME", ""),
        "TZ": "UTC",
        "DATABASE_URL": args.database_url,
        "PORT": str(args.port),
        "LISTEN_HOST": "127.0.0.1",
        "ADAPTER": args.adapter,
    }


def wait_ready(port: int, proc: subprocess.Popen[bytes]) -> None:
    deadline = time.monotonic() + STARTUP_TIMEOUT_S
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(f"o alvo terminou com código {proc.returncode} antes de ficar pronto")
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                break
        except OSError:
            time.sleep(0.1)
    else:
        raise RuntimeError("o alvo não abriu a porta a tempo")
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=REQUEST_TIMEOUT_S)
    connection.request("GET", "/health")
    response = connection.getresponse()
    response.read()
    connection.close()
    if response.status != 200:
        raise RuntimeError(f"/health respondeu {response.status}")


def request_body(request: dict[str, Any]) -> bytes | None:
    if "body_raw" in request:
        return request["body_raw"].encode("utf-8")
    if "body" in request:
        return json.dumps(request["body"], ensure_ascii=False).encode("utf-8")
    return None


def send_all(port: int, requests: list[dict[str, Any]],
             before: Callable[[dict[str, Any]], None] | None = None,
             after: Callable[[dict[str, Any], dict[str, Any]], None] | None = None) -> list[dict[str, Any]]:
    """`before(request)` e `after(request, record)` deixam outro harness observar cada requisição sem reimplementar o envio."""
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=REQUEST_TIMEOUT_S)
    records = []
    for request in requests:
        if before:
            before(request)
        payload = request_body(request)
        headers = dict(request.get("headers", {}))
        if payload is not None:
            headers["content-type"] = "application/json"
        connection.request(request["method"], request["path"], body=payload, headers=headers)
        response = connection.getresponse()
        raw = response.read()
        record = replica_core.build_record(request, response.status, response.getheaders(), raw)
        records.append(record)
        if after:
            after(request, record)
    connection.close()
    return records


@contextlib.contextmanager
def serving(args: argparse.Namespace) -> Iterator[None]:
    """Reseta o banco, sobe o alvo, espera ficar pronto e o derruba ao sair."""
    reset = shlex.split(args.reset_cmd) if args.reset_cmd else ["sh", str(HERE / "db" / "reset.sh")]
    subprocess.run(reset, check=True, timeout=120)
    proc = subprocess.Popen(shlex.split(args.target), cwd=args.cwd, env=server_env(args),
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        wait_ready(args.port, proc)
        yield
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()


def run_once(args: argparse.Namespace, requests: list[dict[str, Any]]) -> list[dict[str, Any]]:
    with serving(args):
        return send_all(args.port, requests)


def command_run(args: argparse.Namespace) -> int:
    requests = json.loads(Path(args.requests).read_text(encoding="utf-8"))["requests"]
    target = shlex.split(args.target)
    version = engine_id(args, target)
    if args.expect_node and version != args.expect_node:
        print(f"run_requests: Node {version or 'sem resposta a --version'}, esperado {args.expect_node}", file=sys.stderr)
        return 2
    runs = []
    with open(lock_path(), "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        for _ in range(2 if args.twice else 1):
            runs.append(run_once(args, requests))
    problems = [f"{r['id']}: {p}" for run in runs for r in run for p in r["problems"]]
    report = {"meta": {"target": args.target, "node": version, "adapter": args.adapter,
                       "requests": len(requests)}, "results": runs[0]}
    Path(args.out).write_text(json.dumps(report, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    diffs = replica_core.diff_runs(runs[0], runs[1], args.level) if args.twice else []
    if args.golden:
        golden = json.loads(Path(args.golden).read_text(encoding="utf-8"))["results"]
        diffs += [f"golden: {line}" for line in replica_core.diff_runs(golden, runs[0], args.level)]
    for line in problems + diffs:
        print(line, file=sys.stderr)
    return 1 if problems or diffs else 0


def command_burst(args: argparse.Namespace) -> int:
    """Envia `count` GETs sequenciais a `path_template` (com {n}) e exige que todos tenham o status esperado."""
    counts: collections.Counter[int] = collections.Counter()
    with open(lock_path(), "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        with serving(args):
            connection = http.client.HTTPConnection("127.0.0.1", args.port, timeout=REQUEST_TIMEOUT_S)
            for n in range(args.count):
                connection.request("GET", args.path_template.format(n=args.first + n % args.span))
                response = connection.getresponse()
                response.read()
                counts[response.status] += 1
            connection.close()
    print(json.dumps(dict(sorted(counts.items()))))
    return 0 if set(counts) == {args.expect_status} else 1


def command_compare(args: argparse.Namespace) -> int:
    left, right = (json.loads(Path(p).read_text(encoding="utf-8"))["results"] for p in (args.left, args.right))
    diffs = replica_core.diff_runs(left, right, args.level)
    for line in diffs:
        print(line, file=sys.stderr)
    return 1 if diffs else 0


def add_server_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--target", required=True)
    parser.add_argument("--cwd", default=str(HERE))
    parser.add_argument("--adapter", choices=["express", "fastify"], default="express")
    parser.add_argument("--port", type=int, default=18101)
    parser.add_argument("--database-url", default=os.environ.get("REPLICA_DATABASE_URL", DEFAULT_DATABASE_URL))
    parser.add_argument("--reset-cmd", default=None, help="substitui db/reset.sh (testes sem Docker)")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    run = sub.add_parser("run")
    add_server_args(run)
    run.add_argument("--out", required=True)
    run.add_argument("--requests", default=str(HERE / "requests.json"))
    run.add_argument("--expect-node", default=None)
    run.add_argument("--engine-id", default=None, help="identificador do motor no relatório; alvos sem --version (qualquer motor) usam só isto")
    run.add_argument("--level", choices=list(replica_core.LEVELS), default="strict")
    run.add_argument("--twice", action="store_true")
    run.add_argument("--golden", default=None, help="relatório gravado antes; a execução deve ser idêntica a ele")
    run.set_defaults(handler=command_run)
    burst = sub.add_parser("burst")
    add_server_args(burst)
    burst.add_argument("--path-template", required=True)
    burst.add_argument("--count", type=int, required=True)
    burst.add_argument("--first", type=int, default=1000)
    burst.add_argument("--span", type=int, default=9000)
    burst.add_argument("--expect-status", type=int, default=200)
    burst.set_defaults(handler=command_burst)
    compare = sub.add_parser("compare")
    compare.add_argument("left")
    compare.add_argument("right")
    compare.add_argument("--level", choices=list(replica_core.LEVELS), default="strict")
    compare.set_defaults(handler=command_compare)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    return args.handler(args)


if __name__ == "__main__":
    sys.exit(main())
