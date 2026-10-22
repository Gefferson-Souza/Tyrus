#!/bin/sh
# Testes de disk_guard.sh: simulam df e /proc/meminfo por arquivos.
set -u

# Hermético: o ambiente do chamador (ex.: DISK_GUARD_MIN_GB=15 do fluxo de gates) não pode mudar o resultado.
unset DISK_GUARD_MIN_GB DISK_GUARD_MIN_MEM_MB DISK_GUARD_DF_FILE DISK_GUARD_MEMINFO_FILE CARGO_TARGET_DIR

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

# df falso no PATH: livre depende do caminho recebido e falha se ele não existir (como o df real).
mkdir -p "$tmp/bin" "$tmp/lowvol" "$tmp/home"
cat >"$tmp/bin/df" <<'FAKE'
#!/bin/sh
for last; do :; done
echo "$last" >>"$DF_LOG"
[ -e "$last" ] || exit 1
case $last in "$LOW_ROOT"*) free=2097152 ;; *) free=104857600 ;; esac
printf 'Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sim 1 1 %s 1%% /\n' "$free"
FAKE
chmod +x "$tmp/bin/df"
write_meminfo 4000000

run_fake_df() {
    title=$1; expected=$2; shift 2
    : >"$tmp/df.log"
    out=$(env -u DISK_GUARD_DF_FILE PATH="$tmp/bin:$PATH" DF_LOG="$tmp/df.log" LOW_ROOT="$tmp/lowvol" \
        HOME="$tmp/home" DISK_GUARD_MEMINFO_FILE="$tmp/meminfo.txt" "$@" sh "$guard" 2>&1)
    code=$?
    if [ "$code" -ne "$expected" ]; then
        echo "FAIL: $title (esperado $expected, obtido $code): $out"; failures=$((failures + 1))
    else
        echo "ok: $title"
    fi
    last_out=$out
}

run_fake_df "CARGO_TARGET_DIR inexistente em volume cheio bloqueia" 3 CARGO_TARGET_DIR="$tmp/lowvol/a/b/c"
case $last_out in *"$tmp/lowvol/a/b/c"*) echo "ok: mensagem cita o CARGO_TARGET_DIR";; *) echo "FAIL: mensagem sem o alvo: $last_out"; failures=$((failures + 1));; esac
grep -qx "$tmp/lowvol" "$tmp/df.log" && echo "ok: df recebeu o ancestral existente" || { echo "FAIL: df nao recebeu o ancestral: $(cat "$tmp/df.log")"; failures=$((failures + 1)); }
run_fake_df "CARGO_TARGET_DIR em volume com espaco passa" 0 CARGO_TARGET_DIR="$tmp/home/novo/alvo"
run_fake_df "sem CARGO_TARGET_DIR verifica so o HOME" 0


out=$(DISK_GUARD_MIN_GB=0 DISK_GUARD_MIN_MEM_MB=0 sh "$guard" 2>&1); code=$?
[ "$code" -eq 0 ] && echo "ok: df e meminfo reais com limiar 0" || { echo "FAIL: real limiar 0 ($code): $out"; failures=$((failures + 1)); }
out=$(DISK_GUARD_MIN_GB=1000000 sh "$guard" 2>&1); code=$?
[ "$code" -eq 3 ] && echo "ok: df real com limiar absurdo bloqueia" || { echo "FAIL: real limiar absurdo ($code): $out"; failures=$((failures + 1)); }

if [ -z "${DISK_GUARD_TEST_NESTED:-}" ]; then
    out=$(DISK_GUARD_TEST_NESTED=1 DISK_GUARD_MIN_GB=15 DISK_GUARD_MIN_MEM_MB=1000 DISK_GUARD_DF_FILE=/nonexistent sh "$0" 2>&1); code=$?
    [ "$code" -eq 0 ] && echo "ok: hermetico com env de gates exportado" || { echo "FAIL: nao hermetico ($code)"; failures=$((failures + 1)); }
fi

if [ "$failures" -ne 0 ]; then
    echo "$failures caso(s) falharam"
    exit 1
fi
echo "todos os casos passaram"
