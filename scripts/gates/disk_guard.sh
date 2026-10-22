#!/bin/sh
# Guarda de disco e memória para scripts longos.
# Saída: 0 = ok, 3 = BLOCKED(disk), 2 = erro de uso ou dado ilegível.
# Env: DISK_GUARD_MIN_GB (3), DISK_GUARD_MIN_MEM_MB (800), CARGO_TARGET_DIR, HOME;
#      DISK_GUARD_DF_FILE e DISK_GUARD_MEMINFO_FILE substituem df e /proc/meminfo (testes).
set -u

min_gb=${DISK_GUARD_MIN_GB:-3}
min_mem_mb=${DISK_GUARD_MIN_MEM_MB:-800}
meminfo=${DISK_GUARD_MEMINFO_FILE:-/proc/meminfo}

usage_error() {
    echo "disk_guard: $1" >&2
    exit 2
}

is_number() {
    case $1 in
        '' | *[!0-9]*) return 1 ;;
        *) return 0 ;;
    esac
}

is_number "$min_gb" || usage_error "DISK_GUARD_MIN_GB não numérico: '$min_gb'"
is_number "$min_mem_mb" || usage_error "DISK_GUARD_MIN_MEM_MB não numérico: '$min_mem_mb'"

# df falha em caminho inexistente; sobe até o ancestral que existe (alvo ainda não criado).
existing_ancestor() {
    path=$1
    while [ ! -e "$path" ] && [ "$path" != "/" ] && [ "$path" != "." ]; do
        path=$(dirname -- "$path")
    done
    echo "$path"
}

free_kib() {
    if [ -n "${DISK_GUARD_DF_FILE:-}" ]; then
        df_output=$(cat -- "$DISK_GUARD_DF_FILE" 2>/dev/null) || usage_error "DISK_GUARD_DF_FILE ilegível"
    else
        df_output=$(df -Pk -- "$(existing_ancestor "$1")" 2>/dev/null) || usage_error "df falhou para '$1'"
    fi
    kib=$(echo "$df_output" | awk 'NR == 2 { print $4 }')
    is_number "$kib" || usage_error "saída do df não reconhecida para '$1'"
    echo "$kib"
}

block() {
    echo "BLOCKED(disk): $1" >&2
    exit 3
}

min_kib=$((min_gb * 1024 * 1024))
volumes="${HOME:-/}"
[ -n "${CARGO_TARGET_DIR:-}" ] && volumes="$CARGO_TARGET_DIR $volumes"

for volume in $volumes; do
    available=$(free_kib "$volume") || exit $?
    if [ "$available" -lt "$min_kib" ]; then
        block "$((available / 1024 / 1024)) GB livres em '$volume', mínimo ${min_gb} GB"
    fi
done

mem_kib=$(awk '/^MemAvailable:/ { print $2 }' "$meminfo" 2>/dev/null)
is_number "$mem_kib" || usage_error "MemAvailable ilegível em '$meminfo'"
if [ "$mem_kib" -lt $((min_mem_mb * 1024)) ]; then
    block "MemAvailable $((mem_kib / 1024)) MB, mínimo ${min_mem_mb} MB"
fi
