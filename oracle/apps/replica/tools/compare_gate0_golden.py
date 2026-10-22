#!/usr/bin/env python3
"""Confere as requisições 01 a 54 de uma execução da réplica contra o golden do Gate 0 (verificação local).

Uso: compare_gate0_golden.py run.json   (exige TYRUS_GATE0_DIR apontando para ~/.local/share/tyrus-ceiling)
O golden foi gravado no Node 22.21.1; diferenças de motor aparecem aqui e não são defeito da réplica.
"""
from __future__ import annotations

import base64
import json
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "harness"))
import replica_core  # noqa: E402


def golden_records(golden_path: Path, requests: dict[str, dict]) -> list[dict]:
    golden = json.loads(golden_path.read_text(encoding="utf-8"))
    records = []
    for item in golden["requests"]:
        headers = [(name, item["headers"][name.lower()][0]) for name in item["header_order"]]
        body = base64.b64decode(item["body_b64"])
        records.append(replica_core.build_record(requests[item["id"]], item["status"], headers, body))
    return records


def main() -> int:
    run = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))["results"]
    root = Path(os.environ["TYRUS_GATE0_DIR"])
    spec = json.loads((HERE.parent / "requests.json").read_text(encoding="utf-8"))["requests"]
    expected = golden_records(root / "golden" / "golden.json", {r["id"]: r for r in spec})
    wanted = {r["id"] for r in expected}
    actual = [r for r in run if r["id"] in wanted]
    diffs = replica_core.diff_runs(expected, actual, "strict")
    for line in diffs:
        print(line, file=sys.stderr)
    print(f"{len(expected)} requisições do Gate 0 comparadas, {len(diffs)} diferenças")
    return 1 if diffs else 0


if __name__ == "__main__":
    sys.exit(main())
