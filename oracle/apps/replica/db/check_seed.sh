#!/bin/sh
# O seed é determinístico: duas montagens forçadas do template replica_seed têm o mesmo hash, igual a db/seed.checksum.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
expected="$(cat "$HERE/seed.checksum")"

FORCE_RESEED=1 "$HERE/up.sh" >/dev/null
first="$("$HERE/checksum.sh" replica_seed)"
FORCE_RESEED=1 "$HERE/up.sh" >/dev/null
second="$("$HERE/checksum.sh" replica_seed)"
reset_copy="$("$HERE/checksum.sh" replica)"

for actual in "$first" "$second" "$reset_copy"; do
    if [ "$actual" != "$expected" ]; then
        echo "check_seed: hash $actual diferente do esperado $expected" >&2
        exit 1
    fi
done
echo "check_seed: ok ($expected)"
