#!/bin/sh
# Verifica o ADR 0019 (ferramental do oráculo): seções, ferramentas com versão, índice e links.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ADR="$ROOT/docs/architecture/decisions/0019-oracle-and-harness-tooling.md"
INDEX="$ROOT/docs/ARCHITECTURE.md"
fail=0
err() { echo "check_adr_0019: $*" >&2; fail=1; }

[ -f "$ADR" ] || { echo "check_adr_0019: ADR ausente: $ADR" >&2; exit 1; }

grep -q '^Status: Proposto' "$ADR" || err "status deve ser Proposto"
for section in '## Alternativas rejeitadas' '## Gatilhos de revisão'; do
    grep -q "^$section\$" "$ADR" || err "seção ausente: $section"
done

# Cada ferramenta precisa aparecer com a versão exata ao lado.
for pair in 'cargo-mutants:27\.1\.0' 'jsonschema:0\.59\.0' 'python3 -I:3\.12' \
    'Node:26\.11\.1' 'Node:22\.21\.1' 'cargo-nextest:0\.9\.143' 'setup-node:v7'; do
    name="${pair%%:*}"
    version="${pair#*:}"
    grep -E "$name" "$ADR" | grep -Eq "$version" || err "$name sem a versão $version"
done
for bump in zmij regex-automata num-bigint; do
    grep -q "$bump" "$ADR" || err "bump de Cargo.lock sem decisão: $bump"
done

grep -q '0019-oracle-and-harness-tooling.md' "$INDEX" || err "docs/ARCHITECTURE.md não indexa o 0019"

dir="$(dirname "$ADR")"
for link in $(grep -oE '\]\(([^)#]+)(#[^)]*)?\)' "$ADR" | sed -E 's/^\]\(//;s/\)$//;s/#.*//' | grep -vE '^(https?:|$)'); do
    [ -e "$dir/$link" ] || err "link quebrado: $link"
done

[ "$fail" -eq 0 ] && echo "check_adr_0019: ok"
exit "$fail"
