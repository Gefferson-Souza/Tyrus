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


def engine_env(time_zone, extra, home):
    env = {
        "PATH": os.environ.get("PATH", ""),
        "HOME": home,
        "LANG": "en_US.UTF-8",
        "LC_ALL": "en_US.UTF-8",
        "TZ": time_zone,
        "NO_COLOR": "1",
    }
    env.update(extra)
    return env


def run_script(engine_cmd, script, env, timeout=PROBE_TIMEOUT_SECONDS):
    script = os.path.abspath(script)
    if not os.path.isfile(script):
        raise HarnessError(f"script ausente: {script}")
    command = shlex.split(engine_cmd) + [script]
    try:
        done = subprocess.run(
            command, env=env, cwd=os.path.dirname(script), capture_output=True,
            timeout=timeout, check=False,
        )
    except subprocess.TimeoutExpired:
        return {"exit": "timeout", "stdout": b""}
    except OSError as error:
        raise HarnessError(f"motor nao executavel: {error}") from error
    return {"exit": done.returncode, "stdout": done.stdout}


def probe_tuple(engine_cmd, env):
    result = run_script(engine_cmd, os.path.join(ROOT, "fingerprint", "tuple.js"), env)
    if result["exit"] != 0:
        raise HarnessError(f"sonda de tupla falhou (exit {result['exit']})")
    return json.loads(result["stdout"])


def probe_plan(index):
    plan = [(name, name, "UTC") for name in index["fingerprint"]["probes"]]
    local = index["fingerprint"]["local_probe"]
    return plan + [(local, f"{local}@{zone}", zone) for zone in index["fingerprint"]["local_time_zones"]]


def fingerprint_classes(engine_cmd, index, env_for, needed_probes):
    """Roda as sondas necessarias. Devolve ({class_id: dados}, {probe_id: erro})."""
    classes, failed = {}, {}
    for script_name, probe_id, zone in probe_plan(index):
        if needed_probes is not None and probe_id not in needed_probes:
            continue
        result = run_script(engine_cmd, os.path.join(ROOT, "fingerprint", script_name + ".js"), env_for(zone))
        try:
            if result["exit"] != 0:
                raise ValueError(f"exit {result['exit']}")
            payload = json.loads(result["stdout"])
        except ValueError as error:
            failed[probe_id] = str(error)
            continue
        suffix = f"@{zone}" if script_name == index["fingerprint"]["local_probe"] else ""
        for name, values in payload.items():
            if not name.startswith("__"):
                classes[f"fingerprint.{name}{suffix}"] = {"kind": "fingerprint", "probe": probe_id, "n": len(values), "sha256": canonical_sha256(values)}
    return classes, failed


def script_classes(engine_cmd, folder, kind, env, selection):
    classes, outputs = {}, {}
    wanted = selection.wanted
    for name in sorted(os.listdir(folder)):
        stem, extension = os.path.splitext(name)
        class_id = f"{kind}.{stem}"
        if extension not in (".js", ".mjs") or not os.path.isfile(os.path.join(folder, name)):
            continue
        if wanted is not None and class_id not in wanted:
            continue
        runs = [run_script(engine_cmd, os.path.join(folder, name), env, LITMUS_TIMEOUT_SECONDS) for _ in range(selection.stability)]
        digests = {(r["exit"], hashlib.sha256(r["stdout"]).hexdigest()) for r in runs}
        if len(digests) > 1:
            classes[class_id] = {"kind": kind, "unstable": True}
            continue
        classes[class_id] = {"kind": kind, "exit": runs[0]["exit"], "sha256": hashlib.sha256(runs[0]["stdout"]).hexdigest()}
        outputs[class_id] = runs[0]["stdout"]
    return classes, outputs


def collect(engine_cmd, wanted, golden_classes, stability, extra_env):
    """wanted = None grava tudo; senao, so as classes pedidas (e as sondas que as produzem)."""
    index = load_json(os.path.join(ROOT, "index.json"))
    needed = None
    if wanted is not None:
        needed = {golden_classes[c]["probe"] for c in wanted if golden_classes[c]["kind"] == "fingerprint"}
    with tempfile.TemporaryDirectory() as home:
        def env_for(zone):
            return engine_env(zone, extra_env, home)

        tuple_info = probe_tuple(engine_cmd, env_for("UTC"))
        classes, failed = fingerprint_classes(engine_cmd, index, env_for, needed)
        outputs = {}
        for directory in ("litmus", "inspect"):
            found, out = script_classes(engine_cmd, os.path.join(ROOT, directory), directory, env_for("UTC"), Selection(stability, wanted))
            classes.update(found)
            outputs.update(out)
    return tuple_info, classes, failed, outputs, index
