"""Auxiliar do gate unsafe.

Uso (sempre via `python3 -I`):
  unsafe_scan.py allows <diretorio>   lista arquivos .rs que abrem unsafe_code (allow/expect/cfg_attr)
  unsafe_scan.py lints <Cargo.toml>   exit 0 se o crate herda os lints do workspace
"""

import re
import sys
import tomllib
from pathlib import Path

ATTRIBUTE_RE = re.compile(r"#!?\[([^\]]*)\]")
OPENS_RE = re.compile(r"\b(?:allow|expect)\s*\(")


def opens_unsafe(body):
    """True se o corpo do atributo relaxa unsafe_code (allow/expect, direto ou via cfg_attr)."""
    body = body.strip()
    if "unsafe_code" not in body:
        return False
    if body.startswith(("allow", "expect")):
        return True
    return body.startswith("cfg_attr") and OPENS_RE.search(body) is not None


def files_with_unsafe_allow(root):
    hits = []
    for path in sorted(Path(root).rglob("*.rs")):
        text = path.read_text(encoding="utf-8", errors="replace")
        if any(opens_unsafe(body) for body in ATTRIBUTE_RE.findall(text)):
            hits.append(path)
    return hits


def inherits_workspace_lints(manifest):
    data = tomllib.loads(Path(manifest).read_text(encoding="utf-8"))
    return data.get("lints", {}).get("workspace") is True


def main(argv):
    if len(argv) != 3 or argv[1] not in ("allows", "lints"):
        print(__doc__, file=sys.stderr)
        return 2
    if argv[1] == "allows":
        for path in files_with_unsafe_allow(argv[2]):
            print(path)
        return 0
    return 0 if inherits_workspace_lints(argv[2]) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
