# Corpus do oráculo

`seed/` guarda o corpus semente importado byte a byte de `.claude/plan/oracle-seed/`
(`probes/`, `calib/`, `perf-ts/`, `http-crud/`). É tratado como imutável: `seed/corpus.lock` lista o sha256 de
cada arquivo no formato do `sha256sum -c`, e o gate `scripts/gates.sh corpus-immutable` reprova
divergência entre `seed/` e o lock (alteração, inclusão ou remoção sem regerar o lock). Regerar o
lock com `python3 -I scripts/gates/corpus_check.py --write` é decisão do dono: o gate não detecta
um lock regerado na mesma PR, então a proteção contra isso é a revisão (CODEOWNERS do seed, unidade
`gov-ci-aggregator-codeowners`).

## live/

`live/` é o corpus regravado: cópia byte a byte de `seed/probes` (145) e `seed/calib` (101), com `package.json`
e `package-lock.json` próprios (`typescript` 5.1.6, a faixa da réplica) e `tsconfig.json` (CommonJS, ES2022,
`strict`, `moduleDetection: force`). Os goldens vêm do emit por arquivo do `tsc` fixado
(sidecar `front/src/cli.ts`), executado no Node de referência de `oracle.lock`; nunca de
`--experimental-strip-types`, que diverge em `C12_04` e em campos de classe.

```sh
(cd oracle/corpus/live && npm ci --ignore-scripts)          # typescript fixado pelo lockfile
export TYRUS_NODE_REFERENCE=<node 26.11.1> TYRUS_NODE_SEED=<node 22.21.1>
tyrus-oracle record --corpus live [--seed-results <probe-results>]   # regrava goldens e diff
tyrus-oracle record --corpus live --check                            # reexecuta 2x e confere o gravado
```

| Arquivo | Conteúdo |
|---|---|
| `live/golden/<conjunto>/<id>.json` | stdout, stderr (caminho do emit mascarado) e código de saída 0 da referência |
| `live/golden/meta.json` | tupla de `oracle.lock`, versão do `typescript`, perfil de comparação `exact` |
| `live/rerecord-diff.json` | `SAME`, `CHANGED` ou `BAD_PROBE(motivo)` por sonda contra a semente (esquema em `schemas/rerecord-diff.schema.json`) |

`SAME` e `CHANGED` comparam o golden novo com a execução da semente (Node 22.21.1 com remoção de tipos,
`C13_*` com `--experimental-transform-types`). `BAD_PROBE` (referência com saída diferente de 0, estouro de
prazo, stdout que não é UTF-8, duas execuções divergentes = `NONDETERMINISTIC_REFERENCE`) não tem golden e fica
fora do verde e das contagens de equivalência. O comando recusa com saída 2 (`TYRUS-E5001`) qualquer Node
cuja tupla difira de `oracle.lock`. `live/` entra no gate `corpus-immutable` só quando o dono decidir (hoje
protege apenas `seed/`).
