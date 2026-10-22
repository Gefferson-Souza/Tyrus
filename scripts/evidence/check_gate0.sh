#!/bin/sh
# Testes de aceite do manifesto do Gate 0 (program-rfc.md, m0-gate0-evidence-manifest).
set -eu
ROOT=$(git rev-parse --show-toplevel)
SRC="${GATE0_SRC:-$HOME/Desktop/personal-projects/TypeRust/.claude/plan/oracle-seed/ceiling/results-2026-10-20}"
DIR="$ROOT/docs/evidence/gate0"
cd "$ROOT"

(cd "$DIR" && sha256sum -c MANIFEST.sha256 >/dev/null) || { echo "FAIL 1: sha256sum -c"; exit 1; }

if [ -d "$SRC" ]; then
  for f in "$DIR"/*.md "$DIR"/raw.jsonl; do
    b=$(basename "$f")
    [ "$b" = README.md ] && continue
    cmp -s "$f" "$SRC/$b" || { echo "FAIL 2: cmp $b"; exit 1; }
  done
else
  echo "AVISO 2: origem ausente, cmp pulado ($SRC)"
fi

python3 -I -c "
import json, os, re
d = 'docs/evidence/gate0'
m = json.load(open(d + '/MANIFEST.json'))
files = m['files']
assert len(files) == 12, 'esperadas 12 entradas'
assert all({'file', 'sha256', 'bytes'} <= set(e) for e in files), 'chaves ausentes'
lines = [l.split('  ', 1) for l in open(d + '/MANIFEST.sha256').read().splitlines() if l]
assert {(f, h) for h, f in lines} == {(e['file'], e['sha256']) for e in files}, 'MANIFEST.json diverge de MANIFEST.sha256'
for e in files:
    assert os.path.isfile(d + '/' + e['file']), 'arquivo listado ausente: ' + e['file']
    assert e['bytes'] == os.stat(d + '/' + e['file']).st_size, 'bytes diverge: ' + e['file']
extra = set(os.listdir(d)) - {e['file'] for e in files} - {'README.md', 'MANIFEST.json', 'MANIFEST.sha256'}
assert not extra, 'arquivos extras: ' + str(extra)
bins = m['rust_ceiling_binaries']
assert len(bins) == 2 and all(re.fullmatch('[0-9a-f]{64}', b['sha256_declared']) for b in bins), 'rust_ceiling_binaries invalido'
" || { echo "FAIL 3: estrutura do MANIFEST.json"; exit 1; }

grep -q 'v26.11.1' "$DIR/MANIFEST.json" && grep -q '14.6.202.34-node.37' "$DIR/MANIFEST.json" \
  && grep -q '78.3' "$DIR/MANIFEST.json" || { echo "FAIL 4: tupla de runtimes"; exit 1; }

if git check-ignore -q "$DIR/summary.md"; then echo "FAIL 5: summary.md ignorado"; exit 1; fi
echo "OK: 5 testes de aceite"
