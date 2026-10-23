"""Leitor e validador de gates.toml (fonte única de limiares e estado dos gates).

Uso (sempre via `python3 -I`):
  cfg.py get <chave.pontilhada> [arquivo]
  cfg.py validate [arquivo]
  cfg.py parity [arquivo] [--gates-sh caminho] [--ci caminho]
  cfg.py deny-config [arquivo]
Saída: 0 ok, 1 reprovado/ausente, 2 erro de uso.
"""

import os
import re
import sys
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
NAME_RE = re.compile(r"^[a-z][a-z0-9-]*$")
GATE_FN_RE = re.compile(r"^gate_([a-z_]+)\(\)", re.MULTILINE)
CI_STEP_RE = re.compile(r"run:\s+\./scripts/gates\.sh\s+([a-z][a-z0-9-]*)")
STATES = ("ACTIVE", "SKIPPED")
NUMERIC_THRESHOLDS = (
    "file_soft_cap", "file_hard_cap", "fn_max_lines", "fn_max_params",
    "nesting_max", "coverage_min", "disk_min_gb_all", "disk_min_gb_step",
    "g1_margin", "g1b_upper", "g2_promote_max", "g2_demote_min",
    "g2_demote_nights", "m2_rss_ratio_max", "m2_startup_ratio_max",
    "ci_level", "g1_min_rounds",
)


def is_number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def is_str_list(value):
    return isinstance(value, list) and all(isinstance(item, str) for item in value)


def load(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)


def check_thresholds(data, errors):
    thresholds = data.get("thresholds")
    if not isinstance(thresholds, dict):
        errors.append("thresholds: tabela ausente")
        return
    for key in NUMERIC_THRESHOLDS:
        if key not in thresholds:
            errors.append(f"thresholds.{key}: ausente")
        elif not is_number(thresholds[key]) or thresholds[key] <= 0:
            errors.append(f"thresholds.{key}: deve ser número positivo")
    if not is_str_list(thresholds.get("filesize_soft_allowlist")):
        errors.append("thresholds.filesize_soft_allowlist: lista de strings ausente")
    margin = thresholds.get("g1_margin")
    if is_number(margin) and margin < 1.0:
        errors.append(f"thresholds.g1_margin: {margin} < 1.0 (ADR 0018)")
    soft, hard = thresholds.get("file_soft_cap"), thresholds.get("file_hard_cap")
    if is_number(soft) and is_number(hard) and soft > hard:
        errors.append("thresholds.file_soft_cap: maior que file_hard_cap")


def check_gate(name, gate, errors):
    if not NAME_RE.match(name):
        errors.append(f"gates.{name}: nome inválido")
    if not isinstance(gate, dict) or gate.get("state") not in STATES:
        errors.append(f"gates.{name}.state: deve ser ACTIVE ou SKIPPED")
        return
    phase = gate.get("activates_in")
    if gate["state"] == "SKIPPED" and not (isinstance(phase, str) and phase):
        errors.append(f"gates.{name}.activates_in: obrigatório quando SKIPPED")
    if gate["state"] == "ACTIVE" and phase is not None:
        errors.append(f"gates.{name}.activates_in: só vale para SKIPPED")


def check_lists(data, errors):
    legacy = data.get("legacy", {}).get("crates")
    if not is_str_list(legacy) or not legacy or len(set(legacy)) != len(legacy):
        errors.append("legacy.crates: lista não vazia, sem repetição")
        legacy = []
    allow = data.get("unsafe_allowlist", {})
    crates = allow.get("crates")
    if not is_str_list(crates):
        errors.append("unsafe_allowlist.crates: lista de strings ausente")
        crates = []
    for crate in set(crates) & set(legacy):
        errors.append(f"unsafe_allowlist.crates: {crate} também é legado")
    modules = allow.get("modules", {})
    for crate in crates:
        if not is_str_list(modules.get(crate)):
            errors.append(f"unsafe_allowlist.modules.{crate}: lista ausente")
    dependents = data.get("direction", {}).get("allowed_dependents")
    if not isinstance(dependents, dict) or not all(
        is_str_list(v) for v in dependents.values()
    ):
        errors.append("direction.allowed_dependents: tabela de listas ausente")


def validate_data(data):
    errors = []
    if data.get("meta", {}).get("schema_version") != 1:
        errors.append("meta.schema_version: deve ser 1")
    check_thresholds(data, errors)
    check_lists(data, errors)
    gates = data.get("gates")
    if not isinstance(gates, dict) or not gates:
        errors.append("gates: tabela ausente")
    else:
        for name, gate in gates.items():
            check_gate(name, gate, errors)
    if not is_str_list(data.get("claims", {}).get("paths")):
        errors.append("claims.paths: lista de strings ausente")
    return errors


def script_gate_names(gates_sh):
    text = Path(gates_sh).read_text(encoding="utf-8")
    return {name.replace("_", "-") for name in GATE_FN_RE.findall(text)}


def ci_gate_names(ci_yml):
    return set(CI_STEP_RE.findall(Path(ci_yml).read_text(encoding="utf-8")))


def parity_errors(data, gates_sh, ci_yml):
    declared = data.get("gates", {})
    in_script = script_gate_names(gates_sh)
    in_ci = ci_gate_names(ci_yml) - {"all", "help"}
    errors = [f"gate '{n}' está em gates.sh e não em gates.toml" for n in sorted(in_script - set(declared))]
    errors += [f"gate '{n}' está em gates.toml e não em gates.sh" for n in sorted(set(declared) - in_script)]
    for name, gate in sorted(declared.items()):
        if name not in in_ci and "ci_exempt" not in gate:
            errors.append(f"gate '{name}' sem passo em ci.yml (ou ci_exempt)")
    errors += [f"ci.yml chama gate inexistente '{n}'" for n in sorted(in_ci - set(declared))]
    return errors


def deny_config(data):
    """Gera um bans.deny com wrappers: crate guardado só aceita os dependentes permitidos."""
    legacy = data["legacy"]["crates"]
    rules = {crate: list(legacy) for crate in legacy}
    for crate, allowed in data["direction"]["allowed_dependents"].items():
        rules[crate] = list(allowed)
    lines = ['[bans]', 'multiple-versions = "allow"', 'wildcards = "allow"', 'deny = [']
    for crate, wrappers in sorted(rules.items()):
        quoted = ", ".join(f'"{w}"' for w in wrappers)
        lines.append(f'    {{ crate = "{crate}", wrappers = [{quoted}] }},')
    lines.append("]")
    return "\n".join(lines) + "\n"


def lookup(data, dotted):
    node = data
    for part in dotted.split("."):
        if not isinstance(node, dict) or part not in node:
            raise KeyError(dotted)
        node = node[part]
    return node


def render(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list):
        return "\n".join(render(item) for item in value)
    if isinstance(value, dict):
        raise TypeError("a chave aponta para uma tabela")
    return str(value)


def default_toml():
    return os.environ.get("TYRUS_GATES_TOML") or str(REPO / "gates.toml")


def take_option(args, flag, default):
    if flag in args:
        index = args.index(flag)
        value = args[index + 1] if index + 1 < len(args) else None
        del args[index:index + 2]
        return value or default
    return default


def cmd_get(args):
    if not args:
        raise SystemExit(2)
    path = args[1] if len(args) > 1 else default_toml()
    try:
        print(render(lookup(load(path), args[0])))
    except (KeyError, TypeError) as error:
        print(f"ERROR: chave ausente ou inválida: {error}", file=sys.stderr)
        return 1
    return 0


def report(errors, label):
    for message in errors:
        print(f"ERROR: {message}", file=sys.stderr)
    if not errors:
        print(f"{label}: ok")
    return 1 if errors else 0


def cmd_validate(args):
    return report(validate_data(load(args[0] if args else default_toml())), "validate")


def cmd_parity(args):
    args = list(args)
    gates_sh = take_option(args, "--gates-sh", str(REPO / "scripts" / "gates.sh"))
    ci_yml = take_option(args, "--ci", str(REPO / ".github" / "workflows" / "ci.yml"))
    data = load(args[0] if args else default_toml())
    return report(parity_errors(data, gates_sh, ci_yml), "parity")


def cmd_deny_config(args):
    sys.stdout.write(deny_config(load(args[0] if args else default_toml())))
    return 0


COMMANDS = {"get": cmd_get, "validate": cmd_validate, "parity": cmd_parity, "deny-config": cmd_deny_config}


def main(argv):
    if len(argv) < 2 or argv[1] not in COMMANDS:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        return COMMANDS[argv[1]](argv[2:])
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
