#!/bin/sh
# Gate unsafe (PoT R13 emendada pelo ADR 0016). Chamado por scripts/gates.sh na raiz da árvore.
#
# Crate fora da allowlist declara #![forbid(unsafe_code)]. Crate da allowlist declara
# #![deny(unsafe_code)], herda os lints do workspace (senão undocumented_unsafe_blocks some)
# e só abre allow/expect de unsafe_code em módulos nomeados em gates.toml. O lint do
# workspace é deny (forbid no workspace impediria o allow), então o forbid por crate é o
# que esta porta garante que nunca desaparece.
set -u

here=$(cd "$(dirname "$0")" && pwd)
cfg() { python3 -B -I "$here/cfg.py" "$@"; }
scan() { python3 -B -I "$here/unsafe_scan.py" "$@"; }

# "crate:raiz" para cada lib.rs/main.rs; o membro tests é legado.
crate_roots() {
    for root in crates/*/src/lib.rs crates/*/src/main.rs tests/src/lib.rs; do
        [ -f "$root" ] || continue
        case "$root" in
            crates/*) crate=${root#crates/}; crate=${crate%%/*} ;;
            *) crate=integration_tests ;;
        esac
        echo "$crate:$root"
    done
}

check_allow_modules() {
    crate=$1
    named=$(cfg get "unsafe_allowlist.modules.$crate" 2>/dev/null || true)
    fail=0
    for file in $(scan allows "crates/$crate/src"); do
        rel=${file#"crates/$crate/src/"}
        if ! echo "$named" | grep -qxF "$rel"; then
            echo "FAIL: unsafe_code liberado (allow/expect) em $file, módulo não nomeado em gates.toml [unsafe_allowlist.modules]"
            fail=1
        fi
    done
    return "$fail"
}

check_workspace_lints() {
    if ! scan lints "crates/$1/Cargo.toml"; then
        echo "FAIL: crates/$1/Cargo.toml sem [lints] workspace = true (undocumented_unsafe_blocks não se aplicaria)"
        return 1
    fi
}

allowlist=$(cfg get unsafe_allowlist.crates)
fail=0
for entry in $(crate_roots); do
    crate=${entry%%:*}
    root=${entry#*:}
    if echo "$allowlist" | grep -qx "$crate"; then
        wanted='deny'
        check_allow_modules "$crate" || fail=1
        check_workspace_lints "$crate" || fail=1
    else
        wanted='forbid'
    fi
    if ! grep -q "^#!\[$wanted(unsafe_code)\]" "$root"; then
        echo "FAIL: missing #![$wanted(unsafe_code)] in $root ($crate)"
        fail=1
    fi
done
if [ "$fail" -eq 0 ]; then
    echo "All crate roots declare their unsafe_code level."
fi
exit "$fail"
