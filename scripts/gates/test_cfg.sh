#!/bin/sh
# Testes de gates.toml, cfg.py e das portas direction/unsafe/filesize/SKIPPED.
# Saída: 0 = todos passaram. Usa fixtures em diretório temporário; o repositório não é alterado.
set -u

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
toml="$repo/gates.toml"
gates="$repo/scripts/gates.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM
fails=0

cfg() { python3 -B -I "$here/cfg.py" "$@"; }

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

# O comando DEVE falhar e a saída deve conter a mensagem esperada.
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

gate_with() {
    # gate_with <root> <toml> <gate>
    TYRUS_GATES_ROOT=$1 TYRUS_GATES_TOML=$2 sh "$gates" "$3"
}

make_member() {
    # make_member <root> <crate> [dependency]
    dir="$1/crates/$2"
    mkdir -p "$dir/src"
    {
        printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n\n' "$2"
        printf '[dependencies]\n'
        [ -n "${3:-}" ] && printf '%s = { path = "../%s" }\n' "$3" "$3"
    } >"$dir/Cargo.toml"
    printf '#![forbid(unsafe_code)]\n' >"$dir/src/lib.rs"
}

with_workspace_lints() {
    printf '\n[lints]\nworkspace = true\n' >>"$1/crates/$2/Cargo.toml"
}

make_workspace() {
    mkdir -p "$1/crates"
    printf '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n' >"$1/Cargo.toml"
}

t_validate_real() { cfg validate "$toml"; }
t_parity_real() { cfg parity; }

t_threshold_drives_filesize() {
    fx="$tmp/fs"
    mkdir -p "$fx/crates/demo/src"
    seq 1 30 | sed 's/.*/\/\/ line/' >"$fx/crates/demo/src/lib.rs"
    gate_with "$fx" "$toml" filesize || return 1
    sed 's/^file_soft_cap = .*/file_soft_cap = 10/' "$toml" >"$tmp/low.toml"
    must_fail 'Rule 4 soft cap' gate_with "$fx" "$tmp/low.toml" filesize
}

t_skipped_never_pass() {
    gate_with "$repo" "$toml" claims >"$tmp/skip.out" 2>&1 || return 1
    grep -q 'SKIPPED(docs-claims-migration)' "$tmp/skip.out" || return 1
    ! grep -q 'PASS' "$tmp/skip.out"
}

t_skipped_overrides_failure() {
    fx="$tmp/sk"
    mkdir -p "$fx/crates/demo/src"
    seq 1 900 | sed 's/.*/\/\/ line/' >"$fx/crates/demo/src/lib.rs"
    must_fail 'hard cap' gate_with "$fx" "$toml" filesize || return 1
    awk '/^\[gates.filesize\]/ {print; print "state = \"SKIPPED\""; print "activates_in = \"m1\""; skip=1; next}
         /^\[/ {skip=0} skip && /^(state|activates_in)/ {next} {print}' "$toml" >"$tmp/skipped.toml"
    gate_with "$fx" "$tmp/skipped.toml" filesize >"$tmp/skip2.out" 2>&1 || return 1
    grep -q 'SKIPPED(m1)' "$tmp/skip2.out" && ! grep -q 'PASS' "$tmp/skip2.out"
}

t_direction() {
    ok="$tmp/dir_ok"
    make_workspace "$ok"
    make_member "$ok" tyrus_place
    make_member "$ok" tyrus_assemble tyrus_place
    gate_with "$ok" "$toml" direction || return 1
    bad="$tmp/dir_bad"
    make_workspace "$bad"
    make_member "$bad" tyrus_place
    make_member "$bad" tyrus_host tyrus_place
    must_fail 'tyrus_place' gate_with "$bad" "$toml" direction || return 1
    leg="$tmp/dir_legacy"
    make_workspace "$leg"
    make_member "$leg" tyrus_parser
    make_member "$leg" tyrus_newcrate tyrus_parser
    must_fail 'tyrus_parser' gate_with "$leg" "$toml" direction
}

t_unsafe_roots() {
    ok="$tmp/un_ok"
    make_workspace "$ok"
    make_member "$ok" tyrus_demo
    printf '#![deny(unsafe_code)]\n' >"$ok/crates/tyrus_demo/src/lib.rs"
    must_fail 'tyrus_demo' gate_with "$ok" "$toml" unsafe || return 1
    printf '#![forbid(unsafe_code)]\n' >"$ok/crates/tyrus_demo/src/lib.rs"
    gate_with "$ok" "$toml" unsafe || return 1
    make_member "$ok" tyrus_host
    with_workspace_lints "$ok" tyrus_host
    must_fail 'tyrus_host' gate_with "$ok" "$toml" unsafe || return 1
    printf '#![deny(unsafe_code)]\n' >"$ok/crates/tyrus_host/src/lib.rs"
    gate_with "$ok" "$toml" unsafe || return 1
    printf '#![deny(unsafe_code)]\n#[allow(unsafe_code)]\nmod ffi {}\n' >"$ok/crates/tyrus_host/src/lib.rs"
    must_fail 'unsafe_code liberado' gate_with "$ok" "$toml" unsafe
}

# Formas equivalentes de abrir unsafe_code fora do módulo nomeado devem reprovar.
t_unsafe_allow_variants() {
    fx="$tmp/un_var"
    make_workspace "$fx"
    make_member "$fx" tyrus_host
    with_workspace_lints "$fx" tyrus_host
    for body in \
        '#[allow(dead_code, unsafe_code)]' \
        '#[expect(unsafe_code)]' \
        '#![allow(unsafe_code)]' \
        '#[cfg_attr(not(test), allow(
    unsafe_code,
))]'; do
        printf '#![deny(unsafe_code)]\n%s\nmod ffi {}\n' "$body" >"$fx/crates/tyrus_host/src/lib.rs"
        must_fail 'unsafe_code liberado' gate_with "$fx" "$toml" unsafe || return 1
    done
    printf '#![deny(unsafe_code)]\n#[cfg_attr(test, deny(unsafe_code))]\n// allow(unsafe_code) em comentário\nmod ffi {}\n' \
        >"$fx/crates/tyrus_host/src/lib.rs"
    gate_with "$fx" "$toml" unsafe
}

t_unsafe_needs_workspace_lints() {
    fx="$tmp/un_lints"
    make_workspace "$fx"
    make_member "$fx" tyrus_host
    printf '#![deny(unsafe_code)]\n' >"$fx/crates/tyrus_host/src/lib.rs"
    must_fail 'workspace = true' gate_with "$fx" "$toml" unsafe || return 1
    with_workspace_lints "$fx" tyrus_host
    gate_with "$fx" "$toml" unsafe
}

lint_workspace() {
    # Cópia fiel das tabelas de lint do workspace real: o teste valida a configuração versionada.
    fx=$1
    mkdir -p "$fx/crates/tyrus_host/src"
    {
        printf '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n\n'
        sed -n '/^\[workspace.lints/,$p' "$repo/Cargo.toml"
    } >"$fx/Cargo.toml"
    printf '[package]\nname = "tyrus_host"\nversion = "0.1.0"\nedition = "2021"\n\n[lints]\nworkspace = true\n' \
        >"$fx/crates/tyrus_host/Cargo.toml"
}

host_source() {
    printf '#![deny(unsafe_code)]\n\n#[allow(unsafe_code)]\npub fn zero() -> u8 {\n%s    unsafe { core::mem::zeroed() }\n}\n' "$1"
}

t_unsafe_needs_safety_comment() {
    fx="$tmp/lint"
    lint_workspace "$fx"
    host_source '    // SAFETY: u8 aceita o padrão de bits zero.
' >"$fx/crates/tyrus_host/src/lib.rs"
    (cd "$fx" && cargo clippy --offline -q -- -D warnings) || return 1
    host_source '' >"$fx/crates/tyrus_host/src/lib.rs"
    must_fail 'undocumented_unsafe_blocks' sh -c "cd '$fx' && cargo clippy --offline -q -- -D warnings"
}

t_validate_rejects() {
    for key in g1_margin m2_rss_ratio_max m2_startup_ratio_max; do
        grep -v "^$key = " "$toml" >"$tmp/miss.toml"
        must_fail "$key" cfg validate "$tmp/miss.toml" || return 1
    done
    sed 's/^g1_margin = .*/g1_margin = 0.99/' "$toml" >"$tmp/low_margin.toml"
    must_fail 'g1_margin' cfg validate "$tmp/low_margin.toml" || return 1
    sed 's/^m2_rss_ratio_max = .*/m2_rss_ratio_max = "1.0"/' "$toml" >"$tmp/str.toml"
    must_fail 'm2_rss_ratio_max' cfg validate "$tmp/str.toml"
}

t_parity_detects_drift() {
    mkdir -p "$tmp/par"
    cp "$gates" "$tmp/par/gates.sh"
    printf 'gate_phantom() {\n    :\n}\n' >>"$tmp/par/gates.sh"
    must_fail 'phantom' cfg parity --gates-sh "$tmp/par/gates.sh" || return 1
    awk '/^\[gates.audit\]/ {skip=1; next} /^\[/ {skip=0} !skip {print}' "$toml" >"$tmp/par.toml"
    must_fail 'audit' cfg parity "$tmp/par.toml" || return 1
    ci="$repo/.github/workflows/ci.yml"
    grep -v 'gates.sh direction' "$ci" >"$tmp/par/ci_missing.yml"
    must_fail "gate 'direction' sem passo" cfg parity --ci "$tmp/par/ci_missing.yml" || return 1
    { cat "$ci"; printf '      - run: ./scripts/gates.sh phantom\n'; } >"$tmp/par/ci_phantom.yml"
    must_fail "ci.yml chama gate inexistente 'phantom'" cfg parity --ci "$tmp/par/ci_phantom.yml"
}

t_get_values() {
    [ "$(cfg get thresholds.g1_margin)" = "1.05" ] || return 1
    [ "$(cfg get gates.claims.state)" = "SKIPPED" ] || return 1
    cfg get legacy.crates | grep -qx tyrus_parser
}

check "gates.toml valida" t_validate_real
check "paridade gates.sh x gates.toml x ci.yml" t_parity_real
check "limiar do gates.toml muda o resultado do filesize" t_threshold_drives_filesize
check "gate SKIPPED imprime SKIPPED( e nunca PASS" t_skipped_never_pass
check "SKIPPED vence a falha do gate" t_skipped_overrides_failure
check "direction reprova dependência invertida e legado" t_direction
check "unsafe exige forbid/deny e módulos nomeados" t_unsafe_roots
check "unsafe reprova allow/expect/cfg_attr equivalentes" t_unsafe_allow_variants
check "unsafe exige lints do workspace na allowlist" t_unsafe_needs_workspace_lints
check "clippy exige SAFETY em unsafe" t_unsafe_needs_safety_comment
check "validate reprova limiares ausentes ou inválidos" t_validate_rejects
check "parity detecta gate fantasma e gate sem entrada" t_parity_detects_drift
check "get lê escalares e listas" t_get_values

if [ "$fails" -ne 0 ]; then
    echo "test_cfg: $fails falha(s)"
    exit 1
fi
echo "test_cfg: tudo ok"
