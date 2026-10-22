#!/bin/sh
# Testes de disk_guard.sh: simulam df e /proc/meminfo por arquivos.
set -u

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
guard="$here/disk_guard.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failures=0

write_df() {
    # $1 = KiB livres; formato de `df -Pk`
    printf 'Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sim 100000000 1 %s 50%% /\n' "$1" >"$tmp/df.txt"
}

write_meminfo() {
    # $1 = kB de MemAvailable
    printf 'MemTotal:       8000000 kB\nMemAvailable:   %s kB\n' "$1" >"$tmp/meminfo.txt"
}

# run_case <titulo> <codigo-esperado> [VAR=valor ...]
run_case() {
    title=$1
    expected=$2
    shift 2
    out=$(env DISK_GUARD_DF_FILE="$tmp/df.txt" DISK_GUARD_MEMINFO_FILE="$tmp/meminfo.txt" "$@" sh "$guard" 2>&1)
    code=$?
    if [ "$code" -ne "$expected" ]; then
        echo "FAIL: $title (esperado $expected, obtido $code): $out"
        failures=$((failures + 1))
    else
        echo "ok: $title"
    fi
    last_out=$out
}

gb() { echo $(($1 * 1024 * 1024)); }

write_df "$(gb 2)"; write_meminfo 4000000
run_case "2 GB livres bloqueia" 3
case $last_out in *"BLOCKED(disk)"*) echo "ok: mensagem BLOCKED(disk)";; *) echo "FAIL: sem BLOCKED(disk): $last_out"; failures=$((failures + 1));; esac

write_df "$(gb 4)"; write_meminfo 716800
run_case "4 GB com 700 MB de memoria bloqueia" 3
write_meminfo 921600
run_case "4 GB com 900 MB de memoria passa" 0

write_df "$(gb 3)"; write_meminfo 921600
run_case "exatamente 3 GB passa (limiar estrito)" 0
write_df "$(gb 4)"; write_meminfo 819200
run_case "exatamente 800 MB passa (limiar estrito)" 0

write_df "$(gb 10)"; write_meminfo 921600
run_case "override DISK_GUARD_MIN_GB=15 bloqueia 10 GB" 3 DISK_GUARD_MIN_GB=15
run_case "override DISK_GUARD_MIN_MEM_MB=1000 bloqueia 900 MB" 3 DISK_GUARD_MIN_MEM_MB=1000
run_case "limiar nao numerico e erro de uso" 2 DISK_GUARD_MIN_GB=abc

printf 'MemTotal: 8000000 kB\n' >"$tmp/meminfo.txt"
run_case "meminfo sem MemAvailable falha fechado" 2

out=$(DISK_GUARD_MIN_GB=0 DISK_GUARD_MIN_MEM_MB=0 sh "$guard" 2>&1); code=$?
[ "$code" -eq 0 ] && echo "ok: df e meminfo reais com limiar 0" || { echo "FAIL: real limiar 0 ($code): $out"; failures=$((failures + 1)); }
out=$(DISK_GUARD_MIN_GB=1000000 sh "$guard" 2>&1); code=$?
[ "$code" -eq 3 ] && echo "ok: df real com limiar absurdo bloqueia" || { echo "FAIL: real limiar absurdo ($code): $out"; failures=$((failures + 1)); }

if [ "$failures" -ne 0 ]; then
    echo "$failures caso(s) falharam"
    exit 1
fi
echo "todos os casos passaram"
