# Corpus do oráculo

`seed/` guarda o corpus semente importado byte a byte de `.claude/plan/oracle-seed/`
(`probes/`, `calib/`, `perf-ts/`, `http-crud/`). É imutável: `seed/corpus.lock` lista o sha256 de
cada arquivo no formato do `sha256sum -c`, e o gate `scripts/gates.sh corpus-immutable` reprova
qualquer alteração, inclusão ou remoção. Só o dono altera o corpus (regerar o lock com
`python3 -I scripts/gates/corpus_check.py --write` é decisão dele).
