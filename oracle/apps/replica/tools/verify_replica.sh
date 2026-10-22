#!/bin/sh
# Verificação ponta a ponta da réplica (exige Docker e o Node de referência). Sai != 0 na primeira falha.
set -eu
HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"
LOCK="$HERE/../../oracle.lock"
[ -f "$LOCK" ] || { echo "verify_replica: oracle.lock ausente ($LOCK)" >&2; exit 2; }
lock_value() { python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["reference"][sys.argv[2]])' "$LOCK" "$1"; }
EXPECT_NODE="${EXPECT_NODE:-v$(lock_value node)}"
EXPECT_TUPLE="$(lock_value v8) $(lock_value icu) $(lock_value tz) $(lock_value cldr)"
OUT="${REPLICA_OUT:-$HOME/.local/share/tyrus-replica/runs}"
GOLDEN="golden/express-node-26.11.1.json"
RUN() { python3 -I run_requests.py "$@"; }

actual="$(node --version)"
[ "$actual" = "$EXPECT_NODE" ] || { echo "verify_replica: Node $actual, esperado $EXPECT_NODE" >&2; exit 2; }
tuple="$(node -p 'process.versions.v8 + " " + process.versions.icu + " " + process.versions.tz + " " + process.versions.cldr')"
[ "$tuple" = "$EXPECT_TUPLE" ] || { echo "verify_replica: tupla $tuple, esperada $EXPECT_TUPLE" >&2; exit 2; }
mkdir -p "$OUT"
work="$(mktemp -d -p "$OUT" verify.XXXXXX)"

echo "== build determinístico =="
tools/check_build_determinism.sh

echo "== requisições contra o golden (Express, duas execuções cada) =="
db/check_seed.sh
RUN run --target "node dist/emit/main.js" --out "$work/emit.json" --twice --golden "$GOLDEN" --expect-node "$EXPECT_NODE"
RUN run --target "node dist/bundle/main.js" --out "$work/bundle.json" --twice --golden "$GOLDEN" --expect-node "$EXPECT_NODE"

echo "== árvore de produção =="
tools/make_prod_tree.sh "$work/prod"
RUN run --target "node main.js" --cwd "$work/prod/dist-bundle" --out "$work/prod.json" --twice \
    --golden "$GOLDEN" --expect-node "$EXPECT_NODE"
rm -rf "$work/prod"

echo "== Fastify (nível body) =="
RUN run --target "node dist/bundle/main.js" --adapter fastify --port 18102 --out "$work/fastify.json" \
    --twice --level body --golden "$GOLDEN" --expect-node "$EXPECT_NODE"

echo "== 1000 requisições a E2 sem 429 =="
RUN burst --target "node dist/bundle/main.js" --path-template "/products/{n}" --count 1000

if [ -n "${TYRUS_GATE0_DIR:-}" ]; then
    echo "== comparação com o golden do Gate 0 =="
    python3 -I tools/compare_gate0_golden.py "$work/bundle.json"
fi
echo "verify_replica: ok (artefatos em $work)"
