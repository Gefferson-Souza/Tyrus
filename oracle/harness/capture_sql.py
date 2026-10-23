#!/usr/bin/env python3
"""Captura de SQL no fio de um alvo qualquer e verificação dos fatos documentados (`sql_facts.py`).

  capture_sql.py --target-cmd "node dist/bundle/main.js" --out-dir DIR
Grava `DIR/trace.json` (traço completo) e `DIR/facts.json`; sai 1 se algum fato deixar de valer.
Fatos de leitura, não de equivalência: a equivalência é `equiv_wire.py`.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import equiv_wire  # noqa: E402
import sql_facts  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    equiv_wire.add_trace_args(parser)
    parser.add_argument("--out-dir", required=True)
    args = parser.parse_args()
    import wire_trace
    trace = wire_trace.collect_trace(args)
    definitions = json.loads(Path(args.requests).read_text(encoding="utf-8"))["requests"]
    facts = sql_facts.check_facts(trace, definitions)
    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    (out / "trace.json").write_text(json.dumps(trace, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    (out / "facts.json").write_text(json.dumps([{"fact": n, "ok": ok} for n, ok in facts], indent=1) + "\n",
                                    encoding="utf-8")
    failed = [name for name, ok in facts if not ok]
    print(f"{len(facts) - len(failed)}/{len(facts)} fatos verificados")
    for name in failed:
        print(f"FAIL {name}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
