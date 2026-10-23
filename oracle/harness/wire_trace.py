"""Executa `requests.json` da réplica contra um `--target-cmd` com o Postgres atrás do proxy de fio.

Um traço guarda, por requisição: o registro HTTP (`replica_core.build_record`) e as idas e voltas do Postgres com o
sha256 de cada mensagem do cliente; no fim, o estado do banco (`db_state`). Nada de executor, gerador ou
normalização próprios: tudo isso vem de `oracle/apps/replica`.
"""
from __future__ import annotations

import argparse
import fcntl
import http.client
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
REPLICA = HERE.parent / "apps" / "replica"
sys.path.insert(0, str(REPLICA))
sys.path.insert(0, str(REPLICA / "harness"))
sys.path.insert(0, str(HERE))
import db_state  # noqa: E402
import replica_core  # noqa: E402
import run_requests  # noqa: E402
from pgproxy import WireProxy  # noqa: E402
from wire import trips_of  # noqa: E402

STARTUP_ID = "00-startup"
PROXY_PASSWORD = "replica"


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


MUTATIONS = {
    "lowercase-select": lambda kind, payload: payload.replace(b"SELECT", b"select", 1),
}


def server_args(options: argparse.Namespace, proxy_port: int) -> argparse.Namespace:
    url = f"postgres://postgres:{PROXY_PASSWORD}@127.0.0.1:{proxy_port}/{db_state.DATABASE}"
    return argparse.Namespace(database_url=url, port=options.port, adapter=options.adapter,
                              engine_id=options.engine_id, expect_node=options.expect_node)


def send_through_proxy(port: int, proxy: WireProxy, requests: list[dict[str, Any]]) -> dict[str, Any]:
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=run_requests.REQUEST_TIMEOUT_S)
    traced: dict[str, Any] = {}
    for request in requests:
        mark = proxy.mark()
        headers = dict(request.get("headers", {}))
        payload = run_requests.request_body(request)
        if payload is not None:
            headers["content-type"] = "application/json"
        connection.request(request["method"], request["path"], body=payload, headers=headers)
        response = connection.getresponse()
        raw = response.read()
        http_record = replica_core.build_record(request, response.status, response.getheaders(), raw)
        traced[request["id"]] = {"http": http_record,
                                 "trips": [summarize(t) for t in trips_of(proxy.since(mark))]}
    connection.close()
    return traced


def run_traced(options: argparse.Namespace, requests: list[dict[str, Any]]) -> dict[str, Any]:
    subprocess.run(["sh", str(HERE / "tools" / "pg.sh"), "reset"], check=True, timeout=120)
    rewrite = MUTATIONS.get(getattr(options, "inject_mutation", None) or "")
    proxy = WireProxy(options.proxy_port, options.upstream_port, rewrite=rewrite)
    proxy.start()
    args = server_args(options, options.proxy_port)
    proc = subprocess.Popen(shlex.split(options.target), cwd=options.cwd, env=run_requests.server_env(args),
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        run_requests.wait_ready(options.port, proc)
        startup = [summarize(t) for t in trips_of(proxy.since(0))]
        traced = send_through_proxy(options.port, proxy, requests)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        proxy.stop()
    return {STARTUP_ID: {"http": None, "trips": startup}, **traced}


def collect_trace(options: argparse.Namespace) -> dict[str, Any]:
    requests = json.loads(Path(options.requests).read_text(encoding="utf-8"))["requests"]
    target = shlex.split(options.target)
    version = run_requests.engine_id(options, target)
    if options.expect_node and version != options.expect_node:
        raise RuntimeError(f"Node {version or 'sem resposta a --version'}, esperado {options.expect_node}")
    with open(run_requests.lock_path(), "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        traced = run_traced(options, requests)
        state = db_state.collect_state(options.container)
    meta = {"target": options.target, "engine": options.engine_id or version or "unspecified",
            "adapter": options.adapter, "requests": len(requests)}
    return {"meta": meta, "requests": traced, "state": state}
