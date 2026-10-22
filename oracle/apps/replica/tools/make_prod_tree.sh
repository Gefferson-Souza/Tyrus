#!/bin/sh
# Monta a árvore de produção (o que a imagem do dono executa): bundle + `npm ci --omit=dev`.
# Uso: make_prod_tree.sh <diretório de saída vazio ou inexistente>
set -eu
HERE="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:?uso: make_prod_tree.sh <saída>}"
[ ! -e "$OUT" ] || [ -z "$(ls -A "$OUT")" ] || { echo "make_prod_tree: $OUT não está vazio" >&2; exit 1; }
[ -f "$HERE/dist/bundle/main.js" ] || { echo "make_prod_tree: rode npm run build antes" >&2; exit 1; }
mkdir -p "$OUT"
cp "$HERE/package.json" "$HERE/package-lock.json" "$HERE/.npmrc" "$OUT/"
cp -r "$HERE/dist/bundle" "$OUT/dist-bundle"
(cd "$OUT" && npm ci --omit=dev)
