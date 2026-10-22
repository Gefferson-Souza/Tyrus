"""Arquiva a evidência externa do Gate 0 em docs/evidence/gate0 com hashes."""
import hashlib
import json
import os
import re
import shutil
import sys
from pathlib import Path

HOME = Path.home()
DEFAULT_SRC = (HOME / "Desktop/personal-projects/TypeRust/.claude/plan/oracle-seed"
               / "ceiling/results-2026-10-20")
CEILING_ROOT = HOME / ".local/share/tyrus-ceiling"
RUNTIME_TUPLE = {
    "node": "v26.11.1",
    "v8": "14.6.202.34-node.37",
    "icu": "78.3",
    "source": "summary.md, tabela de runtimes (linha node26)",
}
RUST_BINARY_RE = re.compile(
    r'"(rust-ceiling(?:-full)?)": \{"path": "([^"]+)", "bytes": (\d+), "sha256": "([0-9a-f]{64})"\}'
)


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_files(src: Path) -> list[Path]:
    names = ["summary.md", "equivalence.md", "raw.jsonl"]
    names += sorted(p.name for p in src.glob("sql-equiv-*.md"))
    return [src / name for name in names]


def rust_binaries(summary: Path) -> list[dict]:
    entries = []
    for name, path, size, declared in RUST_BINARY_RE.findall(summary.read_text()):
        binary = Path(path)
        if not binary.is_file():
            status = "absent"
        else:
            status = "match" if sha256_of(binary) == declared else "mismatch"
        entries.append({"name": name, "path": path, "bytes": int(size),
                        "sha256_declared": declared, "local_check": status})
    return entries


def main() -> int:
    src = Path(os.environ.get("GATE0_SRC", DEFAULT_SRC))
    out = Path(__file__).resolve().parents[2] / "docs/evidence/gate0"
    out.mkdir(parents=True, exist_ok=True)
    files = []
    for origin in source_files(src):
        shutil.copyfile(origin, out / origin.name)
        files.append({"file": origin.name, "origin": str(origin),
                      "bytes": origin.stat().st_size, "sha256": sha256_of(origin)})
    manifest = {
        "kind": "external_evidence",
        "gate": "M0 Gate 0",
        "closed_on": "2026-10-20",
        "runtime_tuple": RUNTIME_TUPLE,
        "rust_ceiling_binaries": rust_binaries(src / "summary.md"),
        "ceiling_work_area": str(CEILING_ROOT),
        "files": files,
    }
    (out / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
    lines = [f"{e['sha256']}  {e['file']}\n" for e in files]
    (out / "MANIFEST.sha256").write_text("".join(lines))
    print(f"{len(files)} arquivos arquivados em {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
