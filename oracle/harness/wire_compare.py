"""Comparação de dois traços (HTTP, fio do Postgres, estado final) em vereditos por camada.

Camadas: `http` usa `replica_core.diff_runs` da réplica; `wire` compara as idas e voltas por requisição (nível
`bytes`: também o sha256 exato de cada mensagem do cliente; nível `sql`: só protocolo, texto e parâmetros, com as
diferenças aceitas de drivers que não são o node-pg); `state` compara o estado final do banco.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "apps" / "replica" / "harness"))
import replica_core  # noqa: E402
from normalizers import excused  # noqa: E402

WIRE_LEVELS = ("bytes", "sql")
IDENTICAL, NORMALIZED, DIFFERENT = "IDENTICAL", "NORMALIZED", "DIFFERENT"
ACCEPTED_KEYS = ("binary result format", "describe statement instead of portal", "parse OIDs present",
                 "binary parameters (not compared)")


def compare_http(golden: dict[str, Any], actual: dict[str, Any]) -> list[str]:
    left = [r["http"] for r in golden["requests"].values() if r["http"] is not None]
    right = [r["http"] for r in actual["requests"].values() if r["http"] is not None]
    problems = [f"{r['id']}: {p}" for r in right for p in r["problems"]]
    return replica_core.diff_runs(left, right, "strict") + problems


def _trip_fails(rid: str, index: int, pair: tuple[dict, dict], level: str) -> list[str]:
    gold, seen = pair
    fails: list[str] = []
    if gold["protocol"] != seen["protocol"]:
        fails.append(f"{rid} trip {index}: protocol {gold['protocol']} expected, {seen['protocol']} observed")
    if gold["sql"] != seen["sql"]:
        fails.append(f"{rid} trip {index}: statement text differs\n      expected: {str(gold['sql'])[:200]}"
                     f"\n      observed: {str(seen['sql'])[:200]}")
    if all_text(gold, seen) and gold["params"] != seen["params"]:
        fails.append(f"{rid} trip {index}: parameters differ: expected {gold['params']} observed {seen['params']}")
    if gold["errors"] != seen["errors"]:
        fails.append(f"{rid} trip {index}: backend errors {gold['errors']} expected, {seen['errors']} observed")
    if level == "bytes" and gold["client_sha256"] != seen["client_sha256"]:
        fails.append(f"{rid} trip {index}: client bytes differ")
    return fails


def all_text(gold: dict, seen: dict) -> bool:
    return all(f == 0 for f in gold["param_formats"] + seen["param_formats"])


def _accepted(seen: dict, gold: dict, counts: dict[str, int]) -> None:
    if any(f != 0 for f in seen["result_formats"]):
        counts["binary result format"] += 1
    if seen["describe"] == "statement":
        counts["describe statement instead of portal"] += 1
    if any(seen["parse_oids"]):
        counts["parse OIDs present"] += 1
    if not all_text(gold, seen):
        counts["binary parameters (not compared)"] += 1


def compare_wire(golden: dict[str, Any], actual: dict[str, Any], level: str = "bytes") -> dict[str, Any]:
    if level not in WIRE_LEVELS:
        raise ValueError(f"nível de fio desconhecido: {level}")
    fails: list[str] = []
    accepted = dict.fromkeys(ACCEPTED_KEYS, 0)
    for rid, gold in golden["requests"].items():
        seen = actual["requests"].get(rid)
        if seen is None:
            fails.append(f"{rid}: request missing from the contestant trace")
        elif len(gold["trips"]) != len(seen["trips"]):
            fails.append(f"{rid}: {len(gold['trips'])} round trips expected, {len(seen['trips'])} observed "
                         f"({[str(t['sql'])[:40] for t in seen['trips']]})")
        else:
            for index, pair in enumerate(zip(gold["trips"], seen["trips"]), 1):
                fails += _trip_fails(rid, index, pair, level)
                _accepted(pair[1], pair[0], accepted)
    return {"fails": fails, "accepted": accepted, "requests": len(golden["requests"])}


def compare_state(golden: dict[str, Any], actual: dict[str, Any]) -> list[str]:
    fails = []
    for key in sorted(set(golden) | set(actual)):
        if golden.get(key) != actual.get(key):
            fails.append(f"state `{key}`: {_short(golden.get(key))} expected, {_short(actual.get(key))} observed")
    return fails


def _short(value: Any) -> str:
    text = repr(value)
    return text if len(text) <= 160 else text[:157] + "..."


def layer_verdict(fails: list[str], known: list[dict], engine: str, layer: str) -> dict[str, Any]:
    """IDENTICAL sem falhas; NORMALIZED só se toda falha é perdoada por texto exato e id do registro."""
    excuses = excused(known, engine, layer)
    left = [f for f in fails if f not in excuses]
    ids = sorted({excuses[f] for f in fails if f in excuses})
    if left:
        status = DIFFERENT
    else:
        status = NORMALIZED if ids else IDENTICAL
    return {"status": status, "normalizers": ids, "fails": left, "excused": [f for f in fails if f in excuses]}


def compare_traces(golden: dict, actual: dict, known: list[dict], options: dict[str, str]) -> dict[str, Any]:
    engine, level = options["engine"], options.get("wire_level", "bytes")
    wire = compare_wire(golden, actual, level)
    layers = {
        "http": layer_verdict(compare_http(golden, actual), known, engine, "http"),
        "wire": layer_verdict(wire["fails"], known, engine, "wire"),
        "state": layer_verdict(compare_state(golden["state"], actual["state"]), known, engine, "state"),
    }
    green = all(v["status"] in (IDENTICAL, NORMALIZED) for v in layers.values())
    return {"green": green, "wire_level": level, "engine": engine, "requests": wire["requests"],
            "accepted_wire_differences": wire["accepted"], "layers": layers}
