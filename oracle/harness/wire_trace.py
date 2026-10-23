"""Executa `requests.json` da réplica contra um `--target-cmd` com o Postgres atrás do proxy de fio.

Um traço guarda, por requisição: o registro HTTP (`replica_core.build_record`) e as idas e voltas do Postgres com o
sha256 de cada mensagem do cliente; no fim, o estado do banco (`db_state`). Nada de executor, gerador ou
normalização próprios: tudo isso vem de `oracle/apps/replica`.
"""
from __future__ import annotations

import argparse
import fcntl
import json
import shlex
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
REPLICA = HERE.parent / "apps" / "replica"
sys.path.insert(0, str(REPLICA))
sys.path.insert(0, str(REPLICA / "harness"))
sys.path.insert(0, str(HERE))
import db_state  # noqa: E402
import run_requests  # noqa: E402
from pgproxy import WireProxy  # noqa: E402
from wire import trips_of  # noqa: E402

STARTUP_ID = "00-startup"
PROXY_PASSWORD = "replica"
CONTAINER_PREFIX = "tyrus-"
OWNER_PORTS = range(3000, 3003)


def summarize(trip: dict[str, Any]) -> dict[str, Any]:
    bind, parse, describe = trip["bind"], trip["parse"], trip["describe"]
    return {
        "protocol": trip["protocol"], "sql": trip["sql"], "params": trip["params"], "kinds": trip["kinds"],
        "param_formats": bind["param_formats"] if bind else [],
        "result_formats": bind["result_formats"] if bind else [],
        "describe": describe["target"] if describe else None,
        "parse_oids": parse["param_oids"] if parse else [],
        "client_sha256": [message["wire_sha256"] for message in trip["front"]],
        "errors": [e["fields"].get("C") for e in trip["back"] if e["type"] == "E"],
        "connection": trip["front"][0]["conn"],
    }


STATE_MUTATIONS = {
    "bump-order-item": "UPDATE order_item SET quantity = quantity + 1 WHERE id = 1",
}
MUTATIONS = {
    "lowercase-select": lambda kind, payload: payload.replace(b"SELECT", b"select", 1),
}


def server_args(options: argparse.Namespace, proxy_port: int) -> argparse.Namespace:
    url = f"postgres://postgres:{PROXY_PASSWORD}@127.0.0.1:{proxy_port}/{db_state.DATABASE}"
    return argparse.Namespace(database_url=url, port=options.port, adapter=options.adapter,
                              engine_id=options.engine_id, expect_node=options.expect_node)


def reset_command(options: argparse.Namespace) -> str:
    pg_sh = shlex.quote(str(HERE / "tools" / "pg.sh"))
    return (f"env WIRE_PG_CONTAINER={shlex.quote(options.container)} WIRE_PG_PORT={options.upstream_port} "
            f"sh {pg_sh} reset")


def send_through_proxy(port: int, proxy: WireProxy, requests: list[dict[str, Any]]) -> dict[str, Any]:
    traced: dict[str, Any] = {}
    marks: list[int] = []

    def record_trips(request: dict[str, Any], http_record: dict[str, Any]) -> None:
        trips = [summarize(t) for t in trips_of(proxy.since(marks.pop()))]
        traced[request["id"]] = {"http": http_record, "trips": trips}

    run_requests.send_all(port, requests, before=lambda _: marks.append(proxy.mark()), after=record_trips)
    return traced


def run_traced(options: argparse.Namespace, requests: list[dict[str, Any]]) -> dict[str, Any]:
    rewrite = MUTATIONS.get(getattr(options, "inject_mutation", None) or "")
    proxy = WireProxy(options.proxy_port, options.upstream_port, rewrite=rewrite)
    proxy.start()
    args = server_args(options, options.proxy_port)
    args.target, args.cwd, args.reset_cmd = options.target, options.cwd, reset_command(options)
    try:
        with run_requests.serving(args):
            startup = [summarize(t) for t in trips_of(proxy.since(0))]
            traced = send_through_proxy(options.port, proxy, requests)
    finally:
        proxy.stop()
    return {STARTUP_ID: {"http": None, "trips": startup}, **traced}


def validate_options(options: argparse.Namespace) -> None:
    """Restrições do dono: só contêineres `tyrus-` e nenhuma das portas 3000 a 3002 (valem para a CLI, não só para pg.sh)."""
    if not options.container.startswith(CONTAINER_PREFIX):
        raise RuntimeError(f"o contêiner deve começar com {CONTAINER_PREFIX} (recebido {options.container})")
    for flag in ("port", "proxy_port", "upstream_port"):
        if getattr(options, flag) in OWNER_PORTS:
            raise RuntimeError(f"portas 3000 a 3002 são do dono (--{flag.replace('_', '-')} {getattr(options, flag)})")


def collect_trace(options: argparse.Namespace) -> dict[str, Any]:
    validate_options(options)
    requests = json.loads(Path(options.requests).read_text(encoding="utf-8"))["requests"]
    target = shlex.split(options.target)
    version = run_requests.engine_id(options, target)
    if options.expect_node and version != options.expect_node:
        raise RuntimeError(f"Node {version or 'sem resposta a --version'}, esperado {options.expect_node}")
    with open(run_requests.lock_path(), "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        traced = run_traced(options, requests)
        state_mutation = STATE_MUTATIONS.get(getattr(options, "inject_state_mutation", None) or "")
        if state_mutation:
            db_state.execute(options.container, state_mutation)
        state = db_state.collect_state(options.container)
    meta = {"target": options.target, "engine": options.engine_id or version or "unspecified",
            "adapter": options.adapter, "requests": len(requests)}
    return {"meta": meta, "requests": traced, "state": state}
