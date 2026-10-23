#!/bin/sh
# Verificação ponta a ponta do fecho de produção da réplica (exige Docker, Node de referência e o binário `tyrus`).
# Sai != 0 na primeira falha. Variáveis: TYRUS_BIN, NPM_CACHE, CLOSURE_WORK, CLOSURE_DEV_TREE, CLOSURE_PG_PORT.
set -eu
HERE="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
cd "$HERE"
LOCK="$ROOT/oracle/oracle.lock"
EXPECT_NODE="v$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["reference"]["node"])' "$LOCK")"
[ "$(node --version)" = "$EXPECT_NODE" ] || { echo "verify_closure: Node $(node --version), esperado $EXPECT_NODE" >&2; exit 2; }
TYRUS="${TYRUS_BIN:-$ROOT/target/debug/tyrus}"
[ -x "$TYRUS" ] || { echo "verify_closure: binário tyrus ausente ($TYRUS); rode cargo build -p tyrus_cli" >&2; exit 2; }
NPM_CACHE="${NPM_CACHE:-$HOME/.cache/npm}"
WORK="${CLOSURE_WORK:-$HOME/.local/share/tyrus-closure}/run.$$"
free_kb=$(df -Pk "$HOME" | awk 'NR==2 {print $4}')
[ "$free_kb" -ge 2097152 ] || { echo "verify_closure: menos de 2 GB livres" >&2; exit 1; }
mkdir -p "$WORK"

export REPLICA_PG_CONTAINER="${REPLICA_PG_CONTAINER:-tyrus-wave3-closure-pg}"
export REPLICA_PG_PORT="${CLOSURE_PG_PORT:-55497}"
export REPLICA_DATABASE_URL="postgres://postgres:replica@127.0.0.1:${REPLICA_PG_PORT}/replica"
cleanup() { db/down.sh >/dev/null 2>&1 || true; rm -rf "$WORK"; }
trap cleanup EXIT

echo "== emit do ST0 sobre o projeto com dependências de desenvolvimento =="
dev="$WORK/dev"
mkdir -p "$dev"
cp package.json package-lock.json .npmrc tsconfig.json "$dev/"
cp -r src "$dev/src"
if [ -n "${CLOSURE_DEV_TREE:-}" ]; then
    cp -al "$CLOSURE_DEV_TREE/node_modules" "$dev/node_modules"
else
    (cd "$dev" && npm ci --ignore-scripts --no-audit --no-fund --cache "$NPM_CACHE")
fi
node "$ROOT/front/src/cli.ts" --project "$dev" --out "$WORK/st0"

echo "== fecho duas vezes: mesmo closure.lock =="
for run in a b; do
    "$TYRUS" --quiet closure --build --project "$HERE" --emit "$WORK/st0/emit" --out "$WORK/closure-$run" --npm-cache "$NPM_CACHE"
done
first="$(sha256sum "$WORK/closure-a/closure.lock" | cut -d' ' -f1)"
second="$(sha256sum "$WORK/closure-b/closure.lock" | cut -d' ' -f1)"
[ "$first" = "$second" ] || { echo "verify_closure: closure.lock difere entre execuções ($first $second)" >&2; exit 1; }
tree="$WORK/closure-a/tree"
lock="$WORK/closure-a/closure.lock"

echo "== o lock lista exatamente os arquivos regulares =="
entries="$(python3 -I -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["files"]))' "$lock")"
on_disk="$(find "$tree" -type f | wc -l)"
[ "$entries" = "$on_disk" ] || { echo "verify_closure: $entries entradas no lock, $on_disk arquivos regulares" >&2; exit 1; }
echo "$entries arquivos"

echo "== typescript e pacotes só de desenvolvimento fora do fecho =="
for blocked in typescript webpack ts-loader; do
    [ ! -e "$tree/node_modules/$blocked" ] || { echo "verify_closure: $blocked presente no fecho" >&2; exit 1; }
done
# @types só pode existir quando o lockfile o lista como produção (ex.: @types/validator do class-validator).
python3 -I - "$tree" "$HERE/package-lock.json" <<'PY'
import json, pathlib, sys
tree, packages = pathlib.Path(sys.argv[1]), json.load(open(sys.argv[2]))["packages"]
def production(name):
    entry = packages.get(f"node_modules/@types/{name}")
    return entry is not None and not entry.get("dev", False)
bad = [p.name for p in (tree / "node_modules" / "@types").glob("*") if not production(p.name)]
sys.exit(f"@types de desenvolvimento no fecho: {bad}" if bad else 0)
PY

echo "== réplica executando do fecho, com o gancho de rastro, contra o golden =="
trace="$WORK/trace.jsonl"
db/up.sh >/dev/null
python3 -I run_requests.py run --cwd "$tree" --engine-id "$EXPECT_NODE" --out "$WORK/run.json" \
    --golden golden/express-node-26.11.1.json \
    --target "env TYRUS_CLOSURE_ROOT=$tree TYRUS_CLOSURE_TRACE=$trace node --require $ROOT/oracle/harness/closure_trace_hook.cjs app/main.js"
requests="$(python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["meta"]["requests"])' "$WORK/run.json")"
[ "$requests" -ge 60 ] || { echo "verify_closure: só $requests requisições" >&2; exit 1; }

echo "== closure --validate com o rastro de $requests requisições =="
"$TYRUS" --quiet closure --validate --root "$tree" --lock "$lock" --trace "$trace"

echo "== cópia sem um arquivo carregado: validação recusa e lista =="
copy="$WORK/copy"
cp -al "$tree" "$copy"
victim="$(python3 -I -c 'import json,sys
for line in open(sys.argv[1]):
    entry = json.loads(line)
    if entry["kind"] == "load" and entry["path"].startswith("node_modules/@nestjs/core/"):
        print(entry["path"]); break' "$trace")"
[ -n "$victim" ] || { echo "verify_closure: nenhum módulo do @nestjs/core no rastro" >&2; exit 1; }
rm "$copy/$victim"
if "$TYRUS" --quiet closure --validate --root "$copy" --lock "$lock" --trace "$trace" >"$WORK/negative.out" 2>&1; then
    echo "verify_closure: validação aceitou o fecho sem $victim" >&2
    exit 1
fi
grep -q "$victim" "$WORK/negative.out" || { echo "verify_closure: $victim não listado" >&2; exit 1; }
echo "verify_closure: ok ($entries arquivos, $requests requisições, lock $first)"
