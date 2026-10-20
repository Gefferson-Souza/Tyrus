"""Bateria de conformidade do motor: fingerprint, litmus de ordem e litmus de inspect.

Uso:
  python3 -I run.py --engine-cmd "$TYRUS_NODE_REFERENCE" --compare goldens/node-26.11.1
  python3 -I run.py --engine-cmd "$TYRUS_NODE_REFERENCE" --record goldens/node-26.11.1
Saida: relatorio JSON deterministico em stdout. Codigos: 0 = tudo igual, 1 = ha classe
diferente/com erro/instavel, 2 = erro de uso ou do harness.
"""
import argparse
import difflib
import fnmatch
import hashlib
import json
import os
import shlex
import subprocess
import sys
import tempfile
from typing import NamedTuple

ROOT = os.path.dirname(os.path.abspath(__file__))
PROBE_TIMEOUT_SECONDS = 120
LITMUS_TIMEOUT_SECONDS = 20
DEFAULT_RECORD_STABILITY = 5
EXIT_SAME, EXIT_DIFF, EXIT_USAGE = 0, 1, 2


class HarnessError(Exception):
    """Falha do harness (nao do motor): vira saida 2."""


class Measurement(NamedTuple):
    """O que o motor produziu: tupla, classes e sondas com erro."""

    tuple_info: dict
    actual: dict
    failed_probes: dict


class Selection(NamedTuple):
    """Quantas repeticoes por script e quais classes rodar (None = todas)."""

    stability: int
    wanted: object


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def canonical_sha256(values):
    text = json.dumps(values, ensure_ascii=True, separators=(",", ":"))
    return hashlib.sha256(text.encode("ascii")).hexdigest()


def matches(class_id, patterns):
    return patterns is None or any(fnmatch.fnmatchcase(class_id, p) for p in patterns)


def compare_class(golden, actual, failed_probes):
    if golden["kind"] == "fingerprint" and golden["probe"] in failed_probes:
        return "error"
    if actual is None:
        return "missing"
    if actual.get("unstable"):
        return "unstable"
    keys = ("n", "sha256") if golden["kind"] == "fingerprint" else ("exit", "sha256")
    return "same" if all(golden.get(k) == actual.get(k) for k in keys) else "diff"


def build_report(engine_cmd, golden, measurement, patterns):
    statuses = {
        class_id: compare_class(expected, measurement.actual.get(class_id), measurement.failed_probes)
        for class_id, expected in golden["classes"].items()
        if matches(class_id, patterns)
    }
    bad = sorted(c for c, s in statuses.items() if s != "same")
    return {
        "engine_cmd": engine_cmd,
        "engine_tuple": measurement.tuple_info,
        "golden_tuple": golden["tuple"],
        "tuple_matches": measurement.tuple_info == golden["tuple"],
        "counts": {"total": len(statuses), "same": len(statuses) - len(bad), "not_same": len(bad)},
        "diff_classes": bad,
        "failed_probes": measurement.failed_probes,
        "statuses": statuses,
    }


def section_rollup(report, index):
    rollup = {}
    for section, spec in index["sections"].items():
        members = [f"litmus.{m}" for m in spec["members"]]
        bad = [m for m in members if report["statuses"].get(m, "same") != "same"]
        rollup[section] = {"title": spec["title"], "status": "diff" if bad else "same", "not_same": bad}
    return rollup


def failure_diffs(report, goldens_dir, outputs):
    shown = {}
    for class_id in report["diff_classes"]:
        path = os.path.join(goldens_dir, "outputs", class_id.replace(".", "/", 1) + ".out")
        if class_id in outputs and os.path.exists(path):
            with open(path, "rb") as handle:
                expected = handle.read().decode("utf-8", "replace").splitlines()
            got = outputs[class_id].decode("utf-8", "replace").splitlines()
            shown[class_id] = list(difflib.unified_diff(expected, got, "golden", "engine", lineterm="", n=1))[:40]
    return shown
