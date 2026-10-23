#!/usr/bin/env python3
"""Equivalência de HTTP, fio do Postgres e estado final de um alvo qualquer (`--target-cmd`).

Uso (sempre com `python3 -I`):
  equiv_wire.py record  --target-cmd "node dist/bundle/main.js" --out trace.json
  equiv_wire.py check   --target-cmd "<cmd>" --golden trace.json [--out report.json]
  equiv_wire.py twice   --target-cmd "<cmd>" --out report.json     (duas execuções do mesmo alvo)
  equiv_wire.py compare golden.json actual.json                      (sem Docker)
Saída 0: todas as camadas verdes; 1: alguma camada DIFFERENT; 2: erro de configuração do harness.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import normalizers  # noqa: E402
import wire_compare  # noqa: E402

DEFAULT_UPSTREAM_PORT = 55521
DEFAULT_PROXY_PORT = 55621


def add_trace_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--target-cmd", dest="target", required=True)
    parser.add_argument("--cwd", default=str(HERE.parent / "apps" / "replica"))
    parser.add_argument("--adapter", choices=["express", "fastify"], default="express")
    parser.add_argument("--port", type=int, default=18101)
    parser.add_argument("--proxy-port", type=int, default=DEFAULT_PROXY_PORT)
    parser.add_argument("--upstream-port", type=int, default=int(os.environ.get("WIRE_PG_PORT", DEFAULT_UPSTREAM_PORT)))
    parser.add_argument("--container", default=os.environ.get("WIRE_PG_CONTAINER", "tyrus-wire-pg"))
    parser.add_argument("--requests", default=str(HERE.parent / "apps" / "replica" / "requests.json"))
    parser.add_argument("--engine-id", default=None)
    parser.add_argument("--expect-node", default=None)
    parser.add_argument("--inject-mutation", choices=["lowercase-select"], default=None,
                        help="controle negativo: o proxy reescreve um SELECT antes de gravar e encaminhar")
    parser.add_argument("--inject-state-mutation", choices=["bump-order-item"], default=None,
                        help="controle negativo: escrita direta no banco entre o tráfego e a coleta do estado")


def add_compare_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--wire-level", choices=list(wire_compare.WIRE_LEVELS), default="bytes")
    parser.add_argument("--known", default=str(HERE / "known_divergences.json"))
    parser.add_argument("--out", default=None)


def write_json(path: str | None, payload: dict[str, Any]) -> None:
    if path:
        Path(path).write_text(json.dumps(payload, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")


def message_count(trace: dict[str, Any]) -> int:
    return sum(len(t["client_sha256"]) for r in trace["requests"].values() for t in r["trips"])


def verdict(golden: dict, actual: dict, args: argparse.Namespace) -> dict[str, Any]:
    known = normalizers.load_known(Path(args.known))
    options = {"engine": actual["meta"]["engine"], "wire_level": args.wire_level}
    report = wire_compare.compare_traces(golden, actual, known, options)
    report["client_messages"] = {"reference": message_count(golden), "target": message_count(actual)}
    return report


def print_report(report: dict[str, Any]) -> None:
    for name, layer in report["layers"].items():
        suffix = f" ({', '.join(layer['normalizers'])})" if layer["normalizers"] else ""
        print(f"{name}: {layer['status']}{suffix}")
        for line in layer["fails"][:20]:
            print(f"  FAIL {line}")
    print(f"mensagens do cliente: {report['client_messages']}")


def finish(report: dict[str, Any], args: argparse.Namespace) -> int:
    write_json(args.out, report)
    print_report(report)
    return 0 if report["green"] else 1


def command_record(args: argparse.Namespace) -> int:
    import wire_trace
    trace = wire_trace.collect_trace(args)
    write_json(args.out, trace)
    print(f"traço gravado: {len(trace['requests'])} entradas, {message_count(trace)} mensagens do cliente")
    return 0


def command_check(args: argparse.Namespace) -> int:
    import wire_trace
    golden = json.loads(Path(args.golden).read_text(encoding="utf-8"))
    return finish(verdict(golden, wire_trace.collect_trace(args), args), args)


def command_twice(args: argparse.Namespace) -> int:
    import wire_trace
    first, second = wire_trace.collect_trace(args), wire_trace.collect_trace(args)
    return finish(verdict(first, second, args), args)


def command_compare(args: argparse.Namespace) -> int:
    golden, actual = (json.loads(Path(p).read_text(encoding="utf-8")) for p in (args.golden, args.actual))
    return finish(verdict(golden, actual, args), args)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    for name, handler in (("record", command_record), ("check", command_check), ("twice", command_twice)):
        cmd = sub.add_parser(name)
        add_trace_args(cmd)
        add_compare_args(cmd)
        if name == "check":
            cmd.add_argument("--golden", required=True)
        cmd.set_defaults(handler=handler)
    compare = sub.add_parser("compare")
    compare.add_argument("golden")
    compare.add_argument("actual")
    add_compare_args(compare)
    compare.set_defaults(handler=command_compare)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    try:
        return args.handler(args)
    except (normalizers.HarnessError, RuntimeError) as error:
        print(f"equiv_wire: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
