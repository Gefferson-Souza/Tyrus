# Corpus do oráculo

`seed/` guarda o corpus semente importado byte a byte de `.claude/plan/oracle-seed/`
(`probes/`, `calib/`, `perf-ts/`, `http-crud/`). É tratado como imutável: `seed/corpus.lock` lista o sha256 de
cada arquivo no formato do `sha256sum -c`, e o gate `scripts/gates.sh corpus-immutable` reprova
divergência entre `seed/` e o lock (alteração, inclusão ou remoção sem regerar o lock). Regerar o
lock com `python3 -I scripts/gates/corpus_check.py --write` é decisão do dono: o gate não detecta
um lock regerado na mesma PR, então a proteção contra isso é a revisão (CODEOWNERS do seed, unidade
`gov-ci-aggregator-codeowners`).
