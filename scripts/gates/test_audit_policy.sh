#!/bin/sh
# Testes da política de exceções do cargo audit (audit.toml + audit_policy.py + gates.sh audit).
# Saída: 0 = todos passaram. Fixtures em diretório temporário; o repositório não é alterado.
set -u

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
gates="$repo/scripts/gates.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM
fails=0
# Data fixa e exportada: nenhum caso depende do relógio real nem do TYRUS_AUDIT_TODAY do ambiente.
today=2026-10-23
export TYRUS_AUDIT_TODAY=$today

policy() { python3 -B -I "$here/audit_policy.py" "$@"; }

check() {
    name=$1
    shift
    if "$@" >"$tmp/out" 2>&1; then
        echo "ok   $name"
    else
        echo "FAIL $name"
        sed 's/^/     /' "$tmp/out"
        fails=$((fails + 1))
    fi
}

must_fail() {
    expected=$1
    shift
    if "$@" >"$tmp/neg" 2>&1; then
        echo "o comando deveria falhar: $*"
        return 1
    fi
    grep -q -- "$expected" "$tmp/neg" || {
        echo "falhou sem a mensagem esperada '$expected':"
        cat "$tmp/neg"
        return 1
    }
}

# entry <id> <reach> <expires>: bloco [[exception]] completo.
entry() {
    printf '[[exception]]\nid = "%s"\nreach = "%s"\nowner = "Gefferson-Souza"\nexpires = %s\njustification = "motivo de teste"\n\n' "$1" "$2" "$3"
}

deny_for() {
    printf '[advisories]\nversion = 2\nignore = [\n'
    for id in "$@"; do printf '    "%s",\n' "$id"; done
    printf ']\n'
}

t_real_policy_valid() { policy check --audit "$repo/audit.toml" --deny "$repo/deny.toml"; }

t_expired() {
    entry RUSTSEC-2025-0001 absent 2026-10-22 >"$tmp/a.toml"
    deny_for RUSTSEC-2025-0001 >"$tmp/d.toml"
    must_fail 'expirada' policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml" || return 1
    entry RUSTSEC-2025-0001 absent 2026-10-23 >"$tmp/a.toml"
    policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml"
}

t_missing_reach() {
    entry RUSTSEC-2025-0001 absent 2026-12-01 | grep -v '^reach' >"$tmp/a.toml"
    deny_for RUSTSEC-2025-0001 >"$tmp/d.toml"
    must_fail 'reach' policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml" || return 1
    entry RUSTSEC-2025-0001 maybe 2026-12-01 >"$tmp/a.toml"
    must_fail 'reach' policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml"
}

t_horizon() {
    deny_for RUSTSEC-2025-0001 >"$tmp/d.toml"
    entry RUSTSEC-2025-0001 absent 2027-01-22 >"$tmp/a.toml"
    must_fail '90 dias' policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml" || return 1
    entry RUSTSEC-2025-0001 absent 2027-01-21 >"$tmp/a.toml"
    policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml"
}

t_required_fields() {
    deny_for RUSTSEC-2025-0001 >"$tmp/d.toml"
    for field in owner justification; do
        entry RUSTSEC-2025-0001 absent 2026-12-01 | grep -v "^$field" >"$tmp/a.toml"
        must_fail "$field" policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml" || return 1
    done
    { entry RUSTSEC-2025-0001 absent 2026-12-01; entry RUSTSEC-2025-0001 absent 2026-12-01; } >"$tmp/a.toml"
    must_fail 'duplicad' policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml"
}

t_deny_sync() {
    entry RUSTSEC-2025-0001 absent 2026-12-01 >"$tmp/a.toml"
    deny_for RUSTSEC-2025-0001 RUSTSEC-2025-0002 >"$tmp/d.toml"
    must_fail 'RUSTSEC-2025-0002' policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml" || return 1
    deny_for >"$tmp/d.toml"
    must_fail 'RUSTSEC-2025-0001' policy check --audit "$tmp/a.toml" --deny "$tmp/d.toml"
}

t_args_single_source() {
    { entry RUSTSEC-2025-0001 absent 2026-12-01; entry RUSTSEC-2025-0002 build-only 2026-12-01; } >"$tmp/a.toml"
    policy args --audit "$tmp/a.toml" >"$tmp/args.out" || return 1
    [ "$(tr '\n' ' ' <"$tmp/args.out")" = "--ignore RUSTSEC-2025-0001 --ignore RUSTSEC-2025-0002 " ]
}

t_gate_has_no_inline_ignore() {
    ! grep -q -- '--ignore RUSTSEC' "$gates"
}

# Os casos a seguir passam por gates.sh audit (a ligação com audit_policy.py), não pelo script direto.
# TYRUS_SKIP_AUDIT=0 explícito: o escape do ambiente não pode desligar o gate sob teste.
t_gate_rejects_expired() {
    entry RUSTSEC-2025-0001 absent 2026-10-22 >"$tmp/a.toml"
    deny_for RUSTSEC-2025-0001 >"$tmp/d.toml"
    must_fail 'expirada' env TYRUS_SKIP_AUDIT=0 TYRUS_AUDIT_TOML="$tmp/a.toml" TYRUS_DENY_TOML="$tmp/d.toml" sh "$gates" audit
}

t_gate_rejects_missing_reach() {
    entry RUSTSEC-2025-0001 absent 2026-12-01 | grep -v '^reach' >"$tmp/a.toml"
    deny_for RUSTSEC-2025-0001 >"$tmp/d.toml"
    must_fail 'reach' env TYRUS_SKIP_AUDIT=0 TYRUS_AUDIT_TOML="$tmp/a.toml" TYRUS_DENY_TOML="$tmp/d.toml" sh "$gates" audit
}

# Remover a exceção de um advisory conhecido (audit.toml e deny.toml coerentes) faz o cargo audit reprovar.
# Sem cargo audit, sem advisory-db local ou sem o advisory no Cargo.lock vivo, o caso é pulado (nao vacuo: avisa).
t_removed_exception_fails_audit() {
    if ! (cd "$repo" && cargo audit --no-fetch --deny warnings) >"$tmp/live.out" 2>&1 \
        && ! grep -q 'RUSTSEC-2026-0097' "$tmp/live.out"; then
        echo "     pulado: cargo audit indisponivel ou sem advisory-db local"
        return 0
    fi
    grep -q 'RUSTSEC-2026-0097' "$tmp/live.out" || {
        echo "     pulado: RUSTSEC-2026-0097 nao aparece mais no Cargo.lock"
        return 0
    }
    entry RUSTSEC-2025-0119 absent 2026-12-01 >"$tmp/audit_missing.toml"
    deny_for RUSTSEC-2025-0119 >"$tmp/deny_missing.toml"
    must_fail 'RUSTSEC-2026-0097' env TYRUS_SKIP_AUDIT=0 TYRUS_AUDIT_TOML="$tmp/audit_missing.toml" \
        TYRUS_DENY_TOML="$tmp/deny_missing.toml" TYRUS_AUDIT_CARGO_ARGS=--no-fetch \
        sh "$gates" audit
}

check "audit.toml e deny.toml do repositório são válidos" t_real_policy_valid
check "exceção expirada reprova" t_expired
check "exceção sem reach (ou reach inválido) reprova" t_missing_reach
check "validade acima de 90 dias reprova" t_horizon
check "owner, justification e id único são obrigatórios" t_required_fields
check "deny.toml fora de sincronia com audit.toml reprova" t_deny_sync
check "args deriva os --ignore só do audit.toml" t_args_single_source
check "gates.sh não carrega --ignore inline" t_gate_has_no_inline_ignore
check "gates.sh audit reprova exceção expirada" t_gate_rejects_expired
check "gates.sh audit reprova exceção sem reach" t_gate_rejects_missing_reach
check "remover exceção de advisory conhecido reprova no cargo audit" t_removed_exception_fails_audit

if [ "$fails" -ne 0 ]; then
    echo "test_audit_policy: $fails falha(s)"
    exit 1
fi
echo "test_audit_policy: tudo ok"
