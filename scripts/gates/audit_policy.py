#!/usr/bin/env python3
"""Política de exceções do cargo audit (D26): valida audit.toml e deriva os --ignore.

Uso: audit_policy.py check [--audit F] [--deny F] | args [--audit F]
Env: TYRUS_AUDIT_TODAY=AAAA-MM-DD fixa "hoje" (testes).
"""
import datetime
import os
import pathlib
import re
import sys
import tomllib

REPO = pathlib.Path(__file__).resolve().parents[2]
REACH_VALUES = ("absent", "build-only", "present")
MAX_DAYS = 90
ID_PATTERN = re.compile(r"^RUSTSEC-\d{4}-\d{4}$")
TEXT_FIELDS = ("owner", "justification")


def today():
    raw = os.environ.get("TYRUS_AUDIT_TODAY")
    return datetime.date.fromisoformat(raw) if raw else datetime.date.today()


def load(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)


def entry_errors(entry, now):
    label = entry.get("id", "<sem id>")
    errors = []
    if not isinstance(entry.get("id"), str) or not ID_PATTERN.match(entry["id"]):
        errors.append(f"{label}: id ausente ou fora do formato RUSTSEC-AAAA-NNNN")
    if entry.get("reach") not in REACH_VALUES:
        errors.append(f"{label}: reach ausente ou inválido (use {', '.join(REACH_VALUES)})")
    for field in TEXT_FIELDS:
        value = entry.get(field)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"{label}: {field} ausente ou vazio")
    errors.extend(expiry_errors(label, entry.get("expires"), now))
    return errors


def expiry_errors(label, expires, now):
    if not isinstance(expires, datetime.date):
        return [f"{label}: expires ausente ou não é data AAAA-MM-DD"]
    if expires < now:
        return [f"{label}: exceção expirada em {expires}"]
    if (expires - now).days > MAX_DAYS:
        return [f"{label}: expires {expires} passa de {MAX_DAYS} dias"]
    return []


def policy_errors(data, now):
    entries = data.get("exception", [])
    errors = []
    seen = set()
    for entry in entries:
        errors.extend(entry_errors(entry, now))
        if entry.get("id") in seen:
            errors.append(f"{entry['id']}: exceção duplicada")
        seen.add(entry.get("id"))
    return errors


def ids(data):
    return [entry["id"] for entry in data.get("exception", [])]


def sync_errors(audit_ids, deny_data):
    deny_ids = set(deny_data.get("advisories", {}).get("ignore", []))
    errors = [f"{i}: está em audit.toml e falta em deny.toml [advisories].ignore"
              for i in sorted(set(audit_ids) - deny_ids)]
    errors += [f"{i}: está em deny.toml [advisories].ignore e falta em audit.toml"
               for i in sorted(deny_ids - set(audit_ids))]
    return errors


def option(args, flag, default):
    if flag in args:
        index = args.index(flag)
        return args[index + 1]
    return default


def cmd_check(args):
    audit = load(option(args, "--audit", str(REPO / "audit.toml")))
    deny = load(option(args, "--deny", str(REPO / "deny.toml")))
    errors = policy_errors(audit, today()) + sync_errors(ids(audit), deny)
    for message in errors:
        print(f"ERROR: {message}", file=sys.stderr)
    if not errors:
        print(f"audit-policy: ok ({len(ids(audit))} exceções)")
    return 1 if errors else 0


def cmd_args(args):
    audit = load(option(args, "--audit", str(REPO / "audit.toml")))
    for advisory in ids(audit):
        print("--ignore")
        print(advisory)
    return 0


def main(argv):
    commands = {"check": cmd_check, "args": cmd_args}
    if not argv or argv[0] not in commands:
        print(__doc__, file=sys.stderr)
        return 2
    return commands[argv[0]](argv[1:])


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
