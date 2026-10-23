"""Lint de governança: docs x gates.toml, nomenclatura, ADR e orçamento de LOC.

Uso (sempre via `python3 -I`):
  rules_lint.py static [--root DIR] [--toml ARQUIVO]
  rules_lint.py adr-trigger --base REF [--head REF] [--root DIR] [--toml ARQUIVO]
O corpo do PR vem de TYRUS_RULES_LINT_PR_BODY. Saída: 0 ok, 1 reprovado, 2 erro de uso.
"""

import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
RULE_ROW_RE = re.compile(r"^\|\s*(R\d+)\s*\|", re.MULTILINE)
ITERATION_RE = re.compile(r"(?<![\w@/.\-])v[123](?!\w|\.\d)")
ITERATION_WORD_RE = re.compile(r"\bitera(?:ção|ções|cao|coes|tion|tions)\s+[123]\b", re.IGNORECASE)
CRATE_TOKEN_RE = re.compile(r"(?:^|[_-])v[123](?:$|[_-])")
STATUS_RE = re.compile(r"^Status:\s*(.*)$", re.MULTILINE)
KNOWN_STATUS = ("accepted", "proposto", "proposed", "superseded", "rejected", "rejeitado", "deprecated")
ADR_FILE_RE = re.compile(r"^(\d{4})-.+\.md$")
ADR_LINK_RE = re.compile(r"decisions/(\d{4}-[\w.\-]+\.md)")
ADR_REF_RE = re.compile(r"\bADR[\s\-]?\d{4}\b")
PUBLIC_TRAIT_RE = re.compile(r"^\+\s*pub\s+(?:unsafe\s+)?trait\s+\w+")
CRATE_PATH_RE = re.compile(r"^crates/([^/]+)/")
SKIP_DIRS = {".git", "target", "node_modules", "vendor", "dist", "__pycache__"}
MIN_CRATES_FOR_ADR = 3


def clean_git_env():
    """Ambiente sem GIT_*: o hook pre-commit de um worktree exporta GIT_DIR/GIT_INDEX_FILE absolutos."""
    return {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}


def load_config(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)


def walk_files(root):
    """Arquivos versionados (git) ou, fora de um repositório, todos exceto diretórios de build."""
    root = Path(root)
    if (root / ".git").exists():
        out = subprocess.run(["git", "-C", str(root), "ls-files", "-z"], capture_output=True, check=True, env=clean_git_env())
        return [root / rel for rel in out.stdout.decode().split("\0") if rel and (root / rel).is_file()]
    found = []
    for base, dirs, names in os.walk(root):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        found.extend(Path(base) / name for name in names)
    return found


def rel(root, path):
    return Path(path).relative_to(root).as_posix()


# --- (a) regras da Constituição x gates.toml ---------------------------------

def constitution_rules(root, data):
    text = (Path(root) / data["rules_lint"]["constitution"]).read_text(encoding="utf-8")
    return RULE_ROW_RE.findall(text)


def describe_rule(name, entry, gates):
    """Devolve (erro, nota) para uma regra mapeada."""
    planned, names = entry.get("planned_in"), entry.get("gates")
    if bool(planned) == bool(names):
        return f"{name}: informar exatamente um entre 'gates' e 'planned_in'", None
    if planned:
        return None, f"{name}: SKIPPED({planned})"
    missing = [g for g in names if g not in gates]
    if missing:
        return f"{name}: gate inexistente em gates.toml: {', '.join(missing)}", None
    states = [gates[g] for g in names]
    skipped = [g.get("activates_in") for g in states if g["state"] == "SKIPPED"]
    if len(skipped) == len(states):
        return None, f"{name}: SKIPPED({skipped[0]})"
    return None, f"{name}: {', '.join(names)} (ACTIVE)"


def check_constitution(root, data):
    mapping = data.get("constitution_rules", {})
    gates = data.get("gates", {})
    doc_rules = constitution_rules(root, data)
    errors = [f"{r}: regra da Constituição sem gate em gates.toml [constitution_rules]" for r in doc_rules if r not in mapping]
    errors += [f"{r}: entrada em gates.toml sem regra na Constituição" for r in sorted(mapping) if r not in doc_rules]
    notes = []
    for name in doc_rules:
        if name in mapping:
            error, note = describe_rule(name, mapping[name], gates)
            errors.extend([error] if error else [])
            notes.extend([note] if note else [])
    return errors, notes


# --- (b) nomenclatura de iteração -------------------------------------------

def naming_files(root, data):
    root = Path(root)
    for entry in data["rules_lint"]["naming_paths"]:
        target = root / entry
        if target.is_file():
            yield target
        elif target.is_dir():
            yield from sorted(p for p in target.rglob("*.md") if p.is_file())


def scan_line(line, allow):
    for phrase in allow:
        line = line.replace(phrase, "")
    return bool(ITERATION_RE.search(line) or ITERATION_WORD_RE.search(line))


def crate_names(root):
    crates = Path(root) / "crates"
    return sorted(p.name for p in crates.iterdir() if p.is_dir()) if crates.is_dir() else []


def check_naming(root, data):
    allow = data["rules_lint"].get("naming_allow", [])
    errors = []
    for path in naming_files(root, data):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if scan_line(line, allow):
                errors.append(f"{rel(root, path)}:{number}: nome de iteração (v1/v2/v3 ou 'iteração N'): {line.strip()[:90]}")
    errors += [f"crates/{n}: nome de crate carrega nome de iteração" for n in crate_names(root) if CRATE_TOKEN_RE.search(n)]
    return errors


# --- (c) ADR: status e índice -----------------------------------------------

def adr_files(root, data):
    folder = Path(root) / data["rules_lint"]["adr_dir"]
    return sorted(p for p in folder.glob("*.md") if ADR_FILE_RE.match(p.name)) if folder.is_dir() else []


def check_adr_status(path):
    match = STATUS_RE.search(path.read_text(encoding="utf-8"))
    number = path.name[:4]
    if not match:
        return [f"ADR {number}: linha 'Status:' ausente"]
    status = match.group(1).strip()
    if not status.lower().startswith(KNOWN_STATUS):
        return [f"ADR {number}: Status desconhecido: {status[:60]}"]
    if status.lower().startswith("accepted") and re.search(r"\bsuperseded\b", status, re.IGNORECASE):
        return [f"ADR {number}: Status Accepted e Superseded ao mesmo tempo: {status[:80]}"]
    return []


def check_adr_index(root, data, files):
    index = (Path(root) / data["rules_lint"]["adr_index"]).read_text(encoding="utf-8")
    linked = set(ADR_LINK_RE.findall(index))
    present = {p.name for p in files}
    errors = [f"ADR {n[:4]}: indexado em {data['rules_lint']['adr_index']} mas o arquivo não existe" for n in sorted(linked - present)]
    errors += [f"ADR {n[:4]}: arquivo existe mas não está no índice" for n in sorted(present - linked)]
    return errors


def check_adrs(root, data):
    files = adr_files(root, data)
    errors = []
    for path in files:
        errors += check_adr_status(path)
    return errors + check_adr_index(root, data, files)


# --- (e) orçamento de LOC ----------------------------------------------------

def count_lines(path):
    try:
        content = path.read_bytes()
    except OSError:
        return 0
    return content.count(b"\n") + (1 if content and not content.endswith(b"\n") else 0)


def category_of(relative, categories):
    for name, prefixes in categories.items():
        if any(relative.startswith(prefix) for prefix in prefixes):
            return name
    return None


def loc_stats(root, data):
    budget = data["loc_budget"]
    extensions = tuple(budget["extensions"])
    excluded = tuple(budget.get("exclude", []))
    counts = {name: 0 for name in budget["categories"]}
    total = 0
    for path in walk_files(root):
        if not path.name.endswith(extensions) or rel(root, path).startswith(excluded):
            continue
        lines = count_lines(path)
        total += lines
        category = category_of(rel(root, path), budget["categories"])
        if category:
            counts[category] += lines
    tooling = counts.get("tooling", 0)
    wide = sum(counts.values())
    return {"tooling": tooling, "total": total, "counts": counts,
            "ratio": tooling / total if total else 0.0, "wide_ratio": wide / total if total else 0.0}


def check_loc(root, data):
    limit = data["loc_budget"]["tooling_max_ratio"]
    stats = loc_stats(root, data)
    note = (f"ferramental {stats['tooling']}/{stats['total']} LOC = {stats['ratio']:.1%} (teto {limit:.0%}); "
            f"larga com oracle (informativo, não imposto): {stats['wide_ratio']:.1%}")
    notes = [note]
    if stats["wide_ratio"] > limit:
        notes.append(f"AVISO: a leitura larga ({stats['wide_ratio']:.1%}) passa de {limit:.0%}; "
                     "o K8 do RFC pode estar disparado (ADR antes de continuar) - decisão do dono")
    if stats["ratio"] > limit:
        return [f"orçamento de LOC: ferramental em {stats['ratio']:.1%}, acima de {limit:.0%}"], notes
    return [], notes


# --- (d) gatilho de ADR sobre o diff (só CI) ---------------------------------

def run_git(root, *args):
    out = subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True, check=True, env=clean_git_env())
    return out.stdout


def adr_trigger_reasons(root, base, head):
    names = run_git(root, "diff", "--name-only", f"{base}...{head}").splitlines()
    crates = sorted({m.group(1) for n in names if (m := CRATE_PATH_RE.match(n))})
    reasons = []
    if len(crates) >= MIN_CRATES_FOR_ADR:
        reasons.append(f"toca {len(crates)} crates ({', '.join(crates)})")
    patch = run_git(root, "diff", "-U0", f"{base}...{head}", "--", "*.rs")
    if any(PUBLIC_TRAIT_RE.match(line) for line in patch.splitlines()):
        reasons.append("adiciona um pub trait")
    return reasons, names


def has_adr_reference(root, base, head, names, pr_body):
    if any(re.match(r"docs/architecture/decisions/\d{4}-", n) for n in names):
        return True
    messages = run_git(root, "log", "--format=%B", f"{base}..{head}")
    return bool(ADR_REF_RE.search(messages) or ADR_REF_RE.search(pr_body or ""))


def check_adr_trigger(root, base, head, pr_body):
    reasons, names = adr_trigger_reasons(root, base, head)
    if not reasons or has_adr_reference(root, base, head, names, pr_body):
        return []
    return [f"gatilho de ADR (Constituição R12): o diff {' e '.join(reasons)} e não referencia nenhum ADR "
            "(arquivo em docs/architecture/decisions/, ou 'ADR NNNN' em commit/corpo do PR)"]


# --- CLI ---------------------------------------------------------------------

def option(args, flag, default=None):
    if flag in args:
        index = args.index(flag)
        value = args[index + 1]
        del args[index:index + 2]
        return value
    return default


def run_static(root, data):
    failures = 0
    for label, (errors, notes) in (
        ("constitution", check_constitution(root, data)),
        ("adr", (check_adrs(root, data), [])),
        ("naming", (check_naming(root, data), [])),
        ("loc-budget", check_loc(root, data)),
    ):
        print(f"--- rules-lint: {label}")
        for note in notes:
            print(f"  {note}")
        for error in errors:
            print(f"  FAIL: {error}")
        failures += len(errors)
    print("--- rules-lint: adr-trigger")
    print("  SKIPPED(ci-only): roda com 'adr-trigger --base REF' sobre o diff do PR")
    return 1 if failures else 0


def run_trigger(root, data, base, head):
    errors = check_adr_trigger(root, base, head, os.environ.get("TYRUS_RULES_LINT_PR_BODY", ""))
    print("--- rules-lint: adr-trigger")
    for error in errors:
        print(f"  FAIL: {error}")
    if not errors:
        print(f"  ok: {base}...{head} sem gatilho de ADR pendente")
    return 1 if errors else 0


def main(argv):
    args = list(argv[1:])
    root = Path(option(args, "--root", str(REPO))).resolve()
    toml = option(args, "--toml", os.environ.get("TYRUS_GATES_TOML", str(root / "gates.toml")))
    base, head = option(args, "--base"), option(args, "--head", "HEAD")
    if len(args) != 1 or args[0] not in ("static", "adr-trigger") or (args[0] == "adr-trigger" and not base):
        print(__doc__, file=sys.stderr)
        return 2
    data = load_config(toml)
    if args[0] == "static":
        return run_static(root, data)
    return run_trigger(root, data, base, head)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
