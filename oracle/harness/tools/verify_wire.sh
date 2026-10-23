#!/bin/sh
# Verificação ponta a ponta do harness de fio (exige Docker e o Node de referência): Node contra Node em HTTP,
# fio e estado final, controle negativo no proxy, autoteste sobre o traço real e fatos de SQL.
set -eu
HARNESS="$(cd "$(dirname "$0")/.." && pwd)"
REPLICA="$HARNESS/../apps/replica"
LOCK="$HARNESS/../oracle.lock"
[ -f "$LOCK" ] || { echo "verify_wire: oracle.lock ausente ($LOCK)" >&2; exit 2; }
EXPECT_NODE="${EXPECT_NODE:-v$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["reference"]["node"])' "$LOCK")}"
actual="$(node --version)"
[ "$actual" = "$EXPECT_NODE" ] || { echo "verify_wire: Node $actual, esperado $EXPECT_NODE" >&2; exit 2; }
export WIRE_PG_CONTAINER="${WIRE_PG_CONTAINER:-tyrus-wire-pg}" WIRE_PG_PORT="${WIRE_PG_PORT:-55521}"
OUT="${WIRE_OUT:-$HOME/.local/share/tyrus-replica/wire-runs}"
mkdir -p "$OUT"
work="$(mktemp -d -p "$OUT" verify.XXXXXX)"
TARGET="node dist/bundle/main.js"
WIRE() { python3 -I -B "$HARNESS/equiv_wire.py" "$@" --target-cmd "$TARGET" --expect-node "$EXPECT_NODE"; }

[ -f "$REPLICA/dist/bundle/main.js" ] || (cd "$REPLICA" && npm ci && npm run build)
trap 'sh "$HARNESS/tools/pg.sh" down' EXIT
sh "$HARNESS/tools/pg.sh" up

echo "== autoteste do comparador (sem Docker) =="
python3 -I -B "$HARNESS/equiv_selftest.py"

echo "== Node contra Node: HTTP, fio e estado =="
WIRE twice --out "$work/twice.json"
python3 -I -c '
import json, sys
report = json.load(open(sys.argv[1]))
counts = report["client_messages"]
if counts["reference"] != counts["target"] or counts["reference"] == 0:
    sys.exit(f"verify_wire: contagem de mensagens do fio instável: {counts}")
' "$work/twice.json"

echo "== controle negativo: SELECT reescrito no proxy =="
WIRE record --out "$work/golden.json"
if WIRE check --golden "$work/golden.json" --inject-mutation lowercase-select --out "$work/mutated.json" >/dev/null; then
    echo "verify_wire: o SELECT reescrito passou como equivalente" >&2
    exit 1
fi
python3 -I -c '
import json, sys
layers = json.load(open(sys.argv[1]))["layers"]
wire, http = layers["wire"]["status"], layers["http"]["status"]
if wire != "DIFFERENT" or http != "IDENTICAL":
    sys.exit("verify_wire: camadas inesperadas no controle negativo: wire=" + wire + " http=" + http)
' "$work/mutated.json"

echo "== autoteste sobre o traço real e fatos de SQL =="
python3 -I -B "$HARNESS/equiv_selftest.py" --trace "$work/golden.json"
python3 -I -B "$HARNESS/capture_sql.py" --target-cmd "$TARGET" --expect-node "$EXPECT_NODE" --out-dir "$work/capture"
echo "verify_wire: ok (artefatos em $work)"
