"""Gate corpus-immutable: oracle/corpus/seed/ bate com corpus.lock, sem sobra nem falta.

Uso: corpus_check.py [--root DIR] [--write]
O lock usa o formato do `sha256sum -c`, com caminhos relativos à raiz do repositório.
"""
import hashlib
import sys
from pathlib import Path

SEED_REL = Path("oracle/corpus/seed")
LOCK_NAME = "corpus.lock"


def sha256_of(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def seed_files(root):
    """Arquivos do corpus (exceto o lock) e symlinks encontrados, ambos como caminhos relativos."""
    seed = root / SEED_REL
    files, links = [], []
    for path in sorted(seed.rglob("*")):
        if path.is_symlink():
            links.append(path.relative_to(root).as_posix())
        elif path.is_file() and path != seed / LOCK_NAME:
            files.append(path.relative_to(root).as_posix())
    return sorted(files), links


def lock_lines(root):
    files, _ = seed_files(root)
    return [f"{sha256_of(root / rel)}  {rel}" for rel in files]


def write_lock(root):
    text = "\n".join(lock_lines(root)) + "\n"
    (root / SEED_REL / LOCK_NAME).write_text(text, encoding="utf-8", newline="\n")


def parse_lock(root):
    entries, errors = {}, []
    lock = root / SEED_REL / LOCK_NAME
    if not lock.is_file():
        return entries, [f"{SEED_REL / LOCK_NAME} ausente"]
    for number, line in enumerate(lock.read_text(encoding="utf-8").splitlines(), 1):
        digest, sep, rel = line.partition("  ")
        if not sep or len(digest) != 64:
            errors.append(f"corpus.lock:{number}: linha malformada")
        elif rel in entries:
            errors.append(f"corpus.lock:{number}: entrada duplicada para {rel}")
        else:
            entries[rel] = digest
    return entries, errors


def check(root):
    entries, errors = parse_lock(root)
    files, links = seed_files(root)
    errors += [f"symlink proibido no corpus: {rel}" for rel in links]
    errors += [f"arquivo fora do lock: {rel}" for rel in files if rel not in entries]
    for rel, digest in sorted(entries.items()):
        path = root / rel
        if not path.is_file():
            errors.append(f"arquivo do lock ausente: {rel}")
        elif sha256_of(path) != digest:
            errors.append(f"conteúdo alterado: {rel}")
    return errors


def main(argv):
    args = list(argv)
    root = Path(__file__).resolve().parents[2]
    if "--root" in args:
        index = args.index("--root")
        root = Path(args[index + 1]).resolve()
        del args[index:index + 2]
    if args == ["--write"]:
        write_lock(root)
        return 0
    errors = check(root)
    for message in errors:
        print(f"ERROR: {message}", file=sys.stderr)
    if errors:
        print("corpus-immutable: o corpus semente é imutável; só o dono o altera.", file=sys.stderr)
        return 1
    print("corpus-immutable: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
