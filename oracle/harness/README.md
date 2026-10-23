# Harness de fio: HTTP, Postgres e estado final

Compara um alvo qualquer (`--target-cmd`: o Node de referência hoje, o binário do Tyrus depois) com outra execução em
três camadas, sobre as 86 requisições de `oracle/apps/replica/requests.json`:

| Camada | O que é comparado | De onde vem |
|---|---|---|
| `http` | status, cabeçalhos (menos `Date`, `Connection`, `Keep-Alive`) e corpo em bytes | `replica_core.diff_runs` da réplica |
| `wire` | por requisição, as idas e voltas do Postgres (protocolo, texto, parâmetros, erros) e o sha256 de cada mensagem do cliente | `pgproxy.py` decodifica o protocolo v3 entre o alvo e o banco |
| `state` | `pg_dump --data-only` com hash, sementes por hash, linhas inseridas por valor | `db_state.py` |

Veredito por camada: `IDENTICAL`, `NORMALIZED(id)` ou `DIFFERENT`. `NORMALIZED` só existe para um texto de falha exato
de uma divergência conhecida (`known_divergences.json`) cujo id está no registro fechado de `normalizers.py`; id fora do
registro aborta o harness com saída 2. Ampliar o registro é um evento com gate, não uma edição de teste.

## Comandos (Node 26.11.1 exato, sempre `python3 -I`)

```sh
WIRE_PG_CONTAINER=tyrus-wire-pg sh tools/pg.sh up      # Postgres 17 por digest (o da réplica), tmpfs, porta 55521
python3 -I equiv_wire.py twice  --target-cmd "node dist/bundle/main.js"          # duas execuções do mesmo alvo
python3 -I equiv_wire.py record --target-cmd "<cmd>" --out trace.json            # grava um traço
python3 -I equiv_wire.py check  --target-cmd "<cmd>" --golden trace.json         # compara com um traço
python3 -I equiv_wire.py compare a.json b.json                                   # sem Docker
python3 -I equiv_selftest.py [--trace trace.json]                                # controles do comparador
python3 -I capture_sql.py --target-cmd "<cmd>" --out-dir DIR                     # fatos de SQL
tools/verify_wire.sh                                                             # tudo, ponta a ponta
```

O `--target-cmd` roda no `--cwd` (padrão `oracle/apps/replica`). Saída 0: tudo verde; 1: alguma camada `DIFFERENT`;
2: configuração inválida. O nível `--wire-level sql` aceita as diferenças de drivers que não são o node-pg (formato
binário de resultado, `Describe` do statement); o padrão `bytes` não aceita nenhuma.

## Procedência

`PROVENANCE.json` guarda o sha256 de cada origem do Gate 0 (`~/.local/share/tyrus-ceiling`) e o do arquivo
importado; `wire.py` é cópia idêntica, os demais são adaptações descritas ali. `equiv.py`, `equiv_core.py` e
`make_requests.py` do Gate 0 não foram importados: executor, gerador e normalização são os da réplica.

## O que este harness não faz

Não grava golden de fio no repositório (o golden é o oráculo e a gravação é decisão do dono), não fala AMQP e não
executa o Tyrus. Contêineres devem começar com `tyrus-`; as portas 3000 a 3002 são recusadas. A CLI (`record`, `check`, `twice`) valida
`--container`, `--port`, `--proxy-port` e `--upstream-port` antes de qualquer coisa (saída 2), e o reset do banco usa o
mesmo contêiner e a mesma porta do proxy e do `pg_dump`.

## Autoteste e contagem de controles

`equiv_selftest.py` roda 38 controles sem traço real (mais os de varredura com `--trace`). O aceite do RFC pedia a
contagem do Gate 0 (96), mas ela vem de `equiv_core.py`, que o RFC proíbe importar; a dispensa fica registrada aqui:
o que vale é que todos passem e o piso nunca desça (`test_selftest.py`). O equivalente de CP1/CN1 a CN3 do Gate 0 é
`wire_compare.response_state_consistency` (cada 201 precisa casar com a linha gravada), com os controles CP1, CN1 a CN5.
Não portados: formato de `createdAt` e `stock` pós-escrita no corpo, porque a réplica mascara os instantes.

## Registro de normalizadores (provisório)

`normalizers.py` é um registro Python provisório e paralelo ao `tyrus_oracle` (Rust: `registry.rs` e `verdict.rs`).
Aqui `NORMALIZED` é verde; no crate, divergência de motor ainda não tem veredito próprio. A unificação (fonte única
lida pelos dois lados) e a decisão de a divergência de motor ser verde ou premissa ficam para antes de
`orc-driver-triple` e `orc-ledger-ci`. A exceção é indexada por `--engine-id`, informado pelo chamador.
