#!/bin/sh
# Duas compilações de cada artefato (emit por arquivo e bundle) saem idênticas byte a byte, iguais a dist/,
# e nenhum caminho absoluto da máquina é embutido no bundle.
set -eu
HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"
[ -f package.json ] || { echo "check_build_determinism: package.json ausente" >&2; exit 1; }
OUT="${REPLICA_OUT:-$HOME/.local/share/tyrus-replica/runs}"
mkdir -p "$OUT"
work="$(mktemp -d -p "$OUT" build.XXXXXX)"

npm ci
rm -rf dist
npm run build
for i in 1 2; do
    npx tsc -p tsconfig.json --outDir "$work/emit$i"
    npx webpack --config webpack.config.js --output-path "$work/bundle$i" >/dev/null
done
diff -r "$work/emit1" "$work/emit2"
diff -r "$work/bundle1" "$work/bundle2"
diff -r "$work/emit1" dist/emit
diff -r "$work/bundle1" dist/bundle
if grep -q "$HERE" "$work/bundle1/main.js"; then
    echo "check_build_determinism: o bundle embute o caminho $HERE" >&2
    exit 1
fi
echo "check_build_determinism: ok (emit: $(find "$work/emit1" -type f | wc -l) arquivos)"
