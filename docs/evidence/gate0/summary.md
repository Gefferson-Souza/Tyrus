# Ceiling test — resultados

Início: 2026-10-20T13:49:26.019Z. Duração: 4220 s. Contestantes: node-express, node-fastify, node24-express, node24-fastify, node26-express, node26-fastify, bun-express, rust-ceiling, rust-ceiling-full. Endpoints: E1, E2, E3, E4, E5, E6. Rodadas: 3. Cold starts por contestante: 10.

Ordem dos contestantes (rotativa): rodada 1: node-express → node-fastify → node24-express → node24-fastify → node26-express → node26-fastify → bun-express → rust-ceiling → rust-ceiling-full; rodada 2: node-fastify → node24-express → node24-fastify → node26-express → node26-fastify → bun-express → rust-ceiling → rust-ceiling-full → node-express; rodada 3: node24-express → node24-fastify → node26-express → node26-fastify → bun-express → rust-ceiling → rust-ceiling-full → node-express → node-fastify.

> Equivalência: `results/equivalence.md` (2026-10-20 13:10 UTC): node-express PASS/PASS, node-fastify PASS, node24-express PASS, node24-fastify PASS, node26-express PASS, node26-fastify PASS, bun-express PASS*, rust-ceiling PASS, rust-ceiling-full PASS. (`PASS*` = divergência documentada; o harness não interpreta além da tabela de veredictos.) Ligação gate→medição: 9 contestante(s) com o sha256 do artefato (binário Rust ou dist do Nest) e o runtime iguais aos do run de equivalência que os aprovou.

## Como ler

- **Métrica primária: CPU do servidor por requisição (µs/req).** É o delta de utime+stime em `/proc/<pid>/stat` (todas as threads do processo, em ticks de 10 ms) dividido pelas requisições concluídas; é a menos sensível à máquina ocupada. A vazão (req/s) na CPU 2 é secundária.
- Servidor fixado na CPU 2 (`taskset -c 2`), Postgres nas CPUs 4-7, oha nas CPUs 8-11. "req/s por core" = req/s dividido pelo número de CPUs do servidor (1 nos runs fixados).
- Cada célula é `mediana [mín–máx]` sobre as rodadas (3 previstas, ordem rotativa). Medições que falharam ficam fora e aparecem em Falhas.
- **Razões:** acima de 1× o contestante é melhor que a referência. CPU, latência, RSS e startup: referência ÷ contestante. req/s: contestante ÷ referência. Calculadas sobre as medianas. NE = node-express, NF = node-fastify, BE = bun-express (Node 22 e Bun nas tabelas de razão por endpoint; a regra de decisão usa o melhor baseline barato, seção própria).
- Capacidade: `oha -c 32 -z 10.0s`. Latência: taxa fixa de 50% da capacidade do node-express na rodada 1, `--latency-correction`, mesma concorrência (o SPEC não fixa a concorrência do run de latência).
- CPU do Postgres: delta de `usage_usec` do cgroup do contêiner; inclui trabalho de fundo e a diferença binário × texto do tokio-postgres (limite aceito do contestante Rust).
- Erros `aborted due to deadline` são as requisições em voo no fim do `-z`: aparecem à parte e não são falha. Qualquer status inesperado (E4 espera 201, E5 espera 400, os demais 200) ou outro erro do oha FALHA a medição.
- **Ordem dos endpoints: E1, E2, E3, E5, E6 e por último E4.** O E4 insere linhas; aquecimento e medição dele vêm depois de todos os outros, então os demais endpoints sempre veem o banco do seed, qualquer que seja a velocidade do contestante. O banco é recriado entre contestantes (`reset.sh`). O RSS ocioso é lido depois do aquecimento dos endpoints de leitura; o `VmHWM` final cobre o E4 também.
- E2 sorteia o id uniforme entre 1000 e 9999 (`[1-9][0-9]{3}`). E3 sorteia `offset` uniforme entre os múltiplos de 10 de 10 a 9990. O offset 0 nunca é sorteado (0,1% de um sorteio uniforme): o rand-regex não emite um `0` sem padding dentro de um ramo uniforme.

## Regra de decisão (CPU do servidor por requisição: teto Rust vs MELHOR baseline barato)

O baseline de cada endpoint é o contestante de **menor CPU mediana por requisição** entre Node 22, 24 e 26 × Express e Fastify (`node-express`, `node-fastify`, `node24-express`, `node24-fastify`, `node26-express`, `node26-fastify`); a tabela abaixo diz quem foi em cada endpoint. Bun é informativo: aparece à parte e nunca é o baseline. A regra é avaliada nos DOIS tetos: `rust-ceiling-full` é o teto conservador (hidrata todas as colunas que o TypeORM hidrata).

| Razão baseline ÷ teto | Leitura |
|---|---|
| ≥ 3× menos CPU | go |
| 1,5× a 3× | só vale pela memória e pelo startup |
| < 1,5× | não vale pela velocidade |

### Melhor baseline barato por endpoint (CPU do servidor, µs/req, mediana)

| Endpoint | Melhor baseline barato | CPU dele (µs/req) | node-express | node-fastify | node24-express | node24-fastify | node26-express | node26-fastify | bun-express (informativo) |
|---|---|---|---|---|---|---|---|---|---|
| E1 | **node26-fastify** | 11,3 [10,7–13,0] | 56,6 | 13,3 | 24,6 | 14,8 | 22,1 | 11,3 | 18,0 |
| E2 | **node26-fastify** | 198,0 [180,3–221,4] | 332,4 | 209,1 | 278,0 | 283,7 | 297,2 | 198,0 | 428,7 |
| E3 | **node26-fastify** | 556,2 [497,3–575,5] | 661,9 | 697,3 | 669,9 | 652,2 | 680,7 | 556,2 | 841,7 |
| E4 | **node26-fastify** | 145,8 [144,4–341,6] | 306,7 | 198,1 | 253,0 | 208,0 | 206,6 | 145,8 | 327,6 |
| E5 | **node26-fastify** | 42,0 [36,3–45,7] | 127,6 | 50,7 | 84,0 | 57,1 | 66,3 | 42,0 | 85,4 |
| E6 | **node-fastify** | 11626,0 [11538,5–12699,6] | 12312,4 | 11626,0 | 13132,4 | 12048,2 | 12481,3 | 11733,0 | 10000,0 |

### rust-ceiling — teto (no E6 decodifica só as 6 colunas que a agregação lê)

| Endpoint | Tipo | Baseline (o melhor) | CPU baseline (µs/req) | CPU teto (µs/req) | Razão baseline÷teto (medianas) | Faixa pior–melhor (min/max das rodadas) | Leitura | Faixa estável? | Bun ÷ teto (informativo) | CPU total (servidor+PG) baseline÷teto |
|---|---|---|---|---|---|---|---|---|---|---|
| E1 | overhead de framework (informativo) | node26-fastify | 11,3 [10,7–13,0] | 7,1 [5,8–8,1] | 1,59× | 1,32× a 2,24× | só vale pela memória e pelo startup (1,5× a 3× menos CPU) | NÃO: a faixa cruza um limiar | 2,52× | 1,59× |
| E2 | decisão (banco) | node26-fastify | 198,0 [180,3–221,4] | 47,0 [44,4–65,3] | 4,21× | 2,76× a 4,99× | GO (≥ 3× menos CPU) | NÃO: a faixa cruza um limiar | 9,12× | 1,28× |
| E3 | decisão (banco) | node26-fastify | 556,2 [497,3–575,5] | 113,7 [113,5–158,1] | 4,89× | 3,15× a 5,07× | GO (≥ 3× menos CPU) | sim | 7,40× | 1,16× |
| E4 | decisão (banco) | node26-fastify | 145,8 [144,4–341,6] | 37,6 [35,7–57,5] | 3,87× | 2,51× a 9,56× | GO (≥ 3× menos CPU) | NÃO: a faixa cruza um limiar | 8,70× | 1,50× |
| E5 | overhead de framework (informativo) | node26-fastify | 42,0 [36,3–45,7] | 7,6 [7,2–11,4] | 5,54× | 3,18× a 6,34× | GO (≥ 3× menos CPU) | sim | 11,27× | 5,55× |
| E6 | decisão (banco) | node-fastify | 11626,0 [11538,5–12699,6] | 670,2 [582,4–689,7] | 17,35× | 16,73× a 21,80× | GO (≥ 3× menos CPU) | sim | 14,92× | 2,42× |

### rust-ceiling-full — teto CONSERVADOR (hidrata todas as colunas que o TypeORM hidrata: no E6, as 19 em vez de 6)

| Endpoint | Tipo | Baseline (o melhor) | CPU baseline (µs/req) | CPU teto (µs/req) | Razão baseline÷teto (medianas) | Faixa pior–melhor (min/max das rodadas) | Leitura | Faixa estável? | Bun ÷ teto (informativo) | CPU total (servidor+PG) baseline÷teto |
|---|---|---|---|---|---|---|---|---|---|---|
| E1 | overhead de framework (informativo) | node26-fastify | 11,3 [10,7–13,0] | 7,0 [5,6–7,9] | 1,62× | 1,35× a 2,33× | só vale pela memória e pelo startup (1,5× a 3× menos CPU) | NÃO: a faixa cruza um limiar | 2,57× | 1,62× |
| E2 | decisão (banco) | node26-fastify | 198,0 [180,3–221,4] | 51,1 [37,2–57,5] | 3,87× | 3,14× a 5,95× | GO (≥ 3× menos CPU) | sim | 8,39× | 1,28× |
| E3 | decisão (banco) | node26-fastify | 556,2 [497,3–575,5] | 128,7 [95,8–139,5] | 4,32× | 3,57× a 6,01× | GO (≥ 3× menos CPU) | sim | 6,54× | 1,13× |
| E4 | decisão (banco) | node26-fastify | 145,8 [144,4–341,6] | 42,7 [37,3–52,4] | 3,42× | 2,75× a 9,17× | GO (≥ 3× menos CPU) | NÃO: a faixa cruza um limiar | 7,68× | 1,38× |
| E5 | overhead de framework (informativo) | node26-fastify | 42,0 [36,3–45,7] | 7,2 [7,2–11,9] | 5,80× | 3,05× a 6,36× | GO (≥ 3× menos CPU) | sim | 11,81× | 5,82× |
| E6 | decisão (banco) | node-fastify | 11626,0 [11538,5–12699,6] | 1342,1 [1317,5–1785,4] | 8,66× | 6,46× a 9,64× | GO (≥ 3× menos CPU) | sim | 7,45× | 2,23× |

**Leitura:** **rust-ceiling:** por endpoint de banco: E2 = GO, E3 = GO, E4 = GO, E6 = GO. Os 4 endpoints avaliados caem na mesma faixa. Faixa instável (cruza um limiar entre as rodadas) em: E2, E4. **rust-ceiling-full:** por endpoint de banco: E2 = GO, E3 = GO, E4 = GO, E6 = GO. Os 4 endpoints avaliados caem na mesma faixa. Faixa instável (cruza um limiar entre as rodadas) em: E4. (teto conservador) Os dois tetos dão a mesma faixa em todos os endpoints de banco. A decisão é do dono; o harness só aplica a tabela.

**Req/s do Rust nos endpoints de banco é limitado pelo Postgres, não pelo servidor.** O Postgres roda em `--cpuset-cpus=4-7`, que são 2 núcleos físicos (pares SMT 4-5 e 6-7) para a vazão de 32 conexões; um servidor Rust bem mais barato que o Node espera o banco, então o req/s dele nesses endpoints mede o Postgres. Por isso a decisão usa a CPU do servidor por requisição (µs/req), que não depende de quem é o gargalo. Utilização mediana do core 2 nos endpoints de banco: rust-ceiling: E2 0,34, E3 0,24, E4 0,75, E6 0,44; rust-ceiling-full: E2 0,36, E3 0,26, E4 0,74, E6 0,85.

oha não saturou em E1, E5: no máximo 2,93 de 4 núcleos (limiar 3,60).

## Resultados por endpoint

### E1 — GET /health

| Contestante | rodadas ok | CPU servidor (µs/req) | CPU Postgres (µs/req) | CPU total (µs/req) | req/s | util. da CPU 2 | p99 cap. (ms) | p99 taxa fixa (ms) | RSS ocioso (MB) | startup frio (ms) [mín–máx] | erros deadline |
|---|---|---|---|---|---|---|---|---|---|---|---|
| node-express | 3 | 56,6 [53,8–61,5] | 0,1 [0,1–0,1] | 56,8 [53,9–61,7] | 17644 [16252–18596] | 0,99 [0,99–0,99] | 5,12 [4,52–5,55] | 14,07 [14,06–22,94] | 235,9 [224,5–238,0] | 399 [378–442] | 96 |
| node-fastify | 3 | 13,3 [12,2–13,7] | 0,0 [0,0–0,0] | 13,4 [12,2–13,8] | 74831 [72783–82142] | 0,97 [0,97–0,97] | 0,94 [0,78–0,95] | 1,43 [1,02–1,67] | 239,6 [237,1–243,8] | 438 [410–468] | 89 |
| node24-express | 3 | 24,6 [22,9–27,2] | 0,1 [0,1–0,1] | 24,7 [22,9–27,3] | 40308 [36626–43627] | 0,98 [0,98–0,98] | 1,98 [1,56–2,20] | 2,51 [1,34–3,65] | 354,8 [345,9–355,4] | 402 [381–467] | 94 |
| node24-fastify | 3 | 14,8 [14,4–18,9] | 0,0 [0,0–0,0] | 14,8 [14,4–18,9] | 67611 [52183–69313] | 0,97 [0,96–0,97] | 1,09 [1,03–1,85] | 2,32 [2,22–3,32] | 382,2 [349,4–389,6] | 431 [398–488] | 93 |
| node26-express | 3 | 22,1 [20,3–23,9] | 0,1 [0,0–0,1] | 22,2 [20,4–23,9] | 45003 [41699–49040] | 0,98 [0,98–0,98] | 1,84 [1,48–2,02] | 2,21 [0,78–3,46] | 268,5 [267,9–308,3] | 375 [350–399] | 95 |
| node26-fastify | 3 | 11,3 [10,7–13,0] | 0,0 [0,0–0,0] | 11,4 [10,7–13,0] | 88170 [76055–93648] | 0,96 [0,96–0,97] | 0,70 [0,68–1,07] | 2,21 [0,79–2,85] | 275,2 [273,8–303,0] | 378 [372–448] | 92 |
| bun-express | 3 | 18,0 [17,6–19,8] | 0,0 [0,0–0,0] | 18,1 [17,7–19,8] | 55526 [50507–56609] | 0,98 [0,98–0,98] | 2,54 [2,52–2,84] | 3,23 [2,11–4,14] | 176,6 [172,1–193,4] | 447 [429–679] | 95 |
| rust-ceiling | 3 | 7,1 [5,8–8,1] | 0,0 [0,0–0,0] | 7,1 [5,8–8,1] | 128356 [107594–171683] | 0,86 [0,83–0,94] | 1,84 [0,32–2,99] | 3,01 [0,60–5,87] | 9,2 [8,6–9,5] | 17 [17–17] | 88 |
| rust-ceiling-full | 3 | 7,0 [5,6–7,9] | 0,0 [0,0–0,0] | 7,0 [5,6–7,9] | 140829 [124059–178301] | 0,94 [0,94–0,94] | 0,50 [0,32–0,89] | 2,49 [1,23–3,61] | 8,6 [8,4–9,0] | 17 [17–18] | 85 |

Latência a taxa fixa:

| Contestante | rodadas ok | taxa (req/s) | p50 (ms) | p99 (ms) | p99.9 (ms) | taxa sustentada |
|---|---|---|---|---|---|---|
| node-express | 3 | 9297 | 0,64 [0,64–0,91] | 14,07 [14,06–22,94] | 19,22 [19,09–42,46] | sim |
| node-fastify | 3 | 9297 | 0,26 [0,25–0,27] | 1,43 [1,02–1,67] | 3,39 [2,96–3,42] | sim |
| node24-express | 3 | 9297 | 0,38 [0,36–0,43] | 2,51 [1,34–3,65] | 4,28 [3,38–5,64] | sim |
| node24-fastify | 3 | 9297 | 0,28 [0,27–0,35] | 2,32 [2,22–3,32] | 3,92 [3,73–4,97] | sim |
| node26-express | 3 | 9297 | 0,34 [0,31–0,37] | 2,21 [0,78–3,46] | 3,90 [2,35–10,16] | sim |
| node26-fastify | 3 | 9297 | 0,26 [0,25–0,27] | 2,21 [0,79–2,85] | 3,82 [2,71–4,64] | sim |
| bun-express | 3 | 9297 | 0,32 [0,30–0,33] | 3,23 [2,11–4,14] | 7,76 [4,81–9,14] | sim |
| rust-ceiling | 3 | 9297 | 0,19 [0,18–0,40] | 3,01 [0,60–5,87] | 4,85 [2,47–12,25] | sim |
| rust-ceiling-full | 3 | 9297 | 0,17 [0,16–0,20] | 2,49 [1,23–3,61] | 4,26 [3,15–6,23] | sim |

Razões contra as três referências (acima de 1× = contestante melhor):

| Contestante | CPU/req vs NE | CPU/req vs NF | CPU/req vs BE | req/s core vs NE | req/s core vs NF | req/s core vs BE | p99 fixo vs NE | p99 fixo vs NF | p99 fixo vs BE |
|---|---|---|---|---|---|---|---|---|---|
| node-express | — | 0,24× | 0,32× | — | 0,24× | 0,32× | — | 0,10× | 0,23× |
| node-fastify | 4,25× | — | 1,35× | 4,24× | — | 1,35× | 9,85× | — | 2,26× |
| node24-express | 2,30× | 0,54× | 0,73× | 2,28× | 0,54× | 0,73× | 5,61× | 0,57× | 1,29× |
| node24-fastify | 3,84× | 0,90× | 1,22× | 3,83× | 0,90× | 1,22× | 6,06× | 0,62× | 1,39× |
| node26-express | 2,56× | 0,60× | 0,81× | 2,55× | 0,60× | 0,81× | 6,37× | 0,65× | 1,46× |
| node26-fastify | 4,99× | 1,18× | 1,59× | 5,00× | 1,18× | 1,59× | 6,38× | 0,65× | 1,46× |
| bun-express | 3,14× | 0,74× | — | 3,15× | 0,74× | — | 4,35× | 0,44× | — |
| rust-ceiling | 7,93× | 1,87× | 2,52× | 7,27× | 1,72× | 2,31× | 4,67× | 0,47× | 1,07× |
| rust-ceiling-full | 8,07× | 1,90× | 2,57× | 7,98× | 1,88× | 2,54× | 5,65× | 0,57× | 1,30× |

### E2 — GET /products/:id

| Contestante | rodadas ok | CPU servidor (µs/req) | CPU Postgres (µs/req) | CPU total (µs/req) | req/s | util. da CPU 2 | p99 cap. (ms) | p99 taxa fixa (ms) | RSS ocioso (MB) | startup frio (ms) [mín–máx] | erros deadline |
|---|---|---|---|---|---|---|---|---|---|---|---|
| node-express | 3 | 332,4 [303,6–333,2] | 469,2 [442,5–474,2] | 801,6 [746,1–807,4] | 3005 [3004–3287] | 1,00 [0,99–1,00] | 24,95 [24,33–25,52] | 17,36 [12,94–27,35] | 235,9 [224,5–238,0] | 399 [378–442] | 96 |
| node-fastify | 3 | 209,1 [199,7–213,3] | 467,4 [457,3–482,4] | 676,6 [657,0–695,7] | 4765 [4671–4990] | 0,99 [0,99–0,99] | 16,74 [14,79–16,87] | 7,35 [5,06–58,76] | 239,6 [237,1–243,8] | 438 [410–468] | 95 |
| node24-express | 3 | 278,0 [246,0–323,1] | 509,2 [473,2–581,2] | 787,1 [719,2–904,3] | 3529 [3072–4048] | 0,99 [0,98–0,99] | 22,49 [19,26–25,02] | 21,84 [19,87–23,94] | 354,8 [345,9–355,4] | 402 [381–467] | 96 |
| node24-fastify | 3 | 283,7 [235,4–293,9] | 581,2 [507,8–601,6] | 865,0 [743,2–895,4] | 3477 [3361–4174] | 0,98 [0,98–0,98] | 20,96 [20,19–22,94] | 14,30 [6,53–14,91] | 382,2 [349,4–389,6] | 431 [398–488] | 96 |
| node26-express | 3 | 297,2 [200,9–331,2] | 571,7 [441,5–722,2] | 869,0 [642,4–1053,5] | 3317 [2744–4970] | 0,98 [0,91–1,00] | 21,87 [13,02–48,90] | 17,08 [3,75–62,86] | 268,5 [267,9–308,3] | 375 [350–399] | 96 |
| node26-fastify | 3 | 198,0 [180,3–221,4] | 484,6 [449,4–525,7] | 682,7 [629,7–747,1] | 5042 [4478–5531] | 0,99 [0,99–1,00] | 13,60 [13,57–15,27] | 5,72 [4,42–6,18] | 275,2 [273,8–303,0] | 378 [372–448] | 96 |
| bun-express | 3 | 428,7 [409,2–436,7] | 500,6 [489,0–506,7] | 929,3 [898,2–943,4] | 2322 [2284–2432] | 0,99 [0,99–0,99] | 44,49 [38,50–45,98] | 16,60 [11,90–18,69] | 176,6 [172,1–193,4] | 447 [429–679] | 96 |
| rust-ceiling | 3 | 47,0 [44,4–65,3] | 486,7 [461,5–585,6] | 533,7 [505,8–650,8] | 7361 [4491–8276] | 0,34 [0,29–0,37] | 8,97 [6,27–15,93] | 3,68 [3,40–7,40] | 9,2 [8,6–9,5] | 17 [17–17] | 96 |
| rust-ceiling-full | 3 | 51,1 [37,2–57,5] | 481,0 [446,6–523,0] | 532,1 [483,8–580,5] | 7612 [6368–8547] | 0,36 [0,32–0,39] | 8,31 [6,51–10,50] | 4,10 [3,29–6,10] | 8,6 [8,4–9,0] | 17 [17–18] | 95 |

Latência a taxa fixa:

| Contestante | rodadas ok | taxa (req/s) | p50 (ms) | p99 (ms) | p99.9 (ms) | taxa sustentada |
|---|---|---|---|---|---|---|
| node-express | 3 | 1643 | 1,09 [1,02–2,12] | 17,36 [12,94–27,35] | 24,41 [23,83–38,77] | sim |
| node-fastify | 3 | 1643 | 0,98 [0,83–1,48] | 7,35 [5,06–58,76] | 24,56 [14,90–65,01] | sim |
| node24-express | 3 | 1643 | 1,14 [1,03–2,49] | 21,84 [19,87–23,94] | 29,86 [28,70–29,98] | sim |
| node24-fastify | 3 | 1643 | 1,14 [1,05–1,82] | 14,30 [6,53–14,91] | 29,17 [10,82–32,67] | sim |
| node26-express | 3 | 1643 | 1,48 [0,80–1,86] | 17,08 [3,75–62,86] | 21,02 [6,01–70,82] | sim |
| node26-fastify | 3 | 1643 | 0,86 [0,75–0,91] | 5,72 [4,42–6,18] | 23,77 [9,20–24,83] | sim |
| bun-express | 3 | 1643 | 1,26 [1,20–1,34] | 16,60 [11,90–18,69] | 25,48 [23,13–29,10] | sim |
| rust-ceiling | 3 | 1643 | 0,64 [0,60–1,34] | 3,68 [3,40–7,40] | 6,19 [5,49–11,29] | sim |
| rust-ceiling-full | 3 | 1643 | 0,84 [0,55–1,17] | 4,10 [3,29–6,10] | 6,28 [5,72–9,08] | sim |

Razões contra as três referências (acima de 1× = contestante melhor):

| Contestante | CPU/req vs NE | CPU/req vs NF | CPU/req vs BE | req/s core vs NE | req/s core vs NF | req/s core vs BE | p99 fixo vs NE | p99 fixo vs NF | p99 fixo vs BE |
|---|---|---|---|---|---|---|---|---|---|
| node-express | — | 0,63× | 1,29× | — | 0,63× | 1,29× | — | 0,42× | 0,96× |
| node-fastify | 1,59× | — | 2,05× | 1,59× | — | 2,05× | 2,36× | — | 2,26× |
| node24-express | 1,20× | 0,75× | 1,54× | 1,17× | 0,74× | 1,52× | 0,80× | 0,34× | 0,76× |
| node24-fastify | 1,17× | 0,74× | 1,51× | 1,16× | 0,73× | 1,50× | 1,21× | 0,51× | 1,16× |
| node26-express | 1,12× | 0,70× | 1,44× | 1,10× | 0,70× | 1,43× | 1,02× | 0,43× | 0,97× |
| node26-fastify | 1,68× | 1,06× | 2,16× | 1,68× | 1,06× | 2,17× | 3,04× | 1,29× | 2,90× |
| bun-express | 0,78× | 0,49× | — | 0,77× | 0,49× | — | 1,05× | 0,44× | — |
| rust-ceiling | 7,07× | 4,45× | 9,12× | 2,45× | 1,54× | 3,17× | 4,72× | 2,00× | 4,51× |
| rust-ceiling-full | 6,50× | 4,09× | 8,39× | 2,53× | 1,60× | 3,28× | 4,24× | 1,79× | 4,05× |

### E3 — GET /products?limit=20&offset=N

| Contestante | rodadas ok | CPU servidor (µs/req) | CPU Postgres (µs/req) | CPU total (µs/req) | req/s | util. da CPU 2 | p99 cap. (ms) | p99 taxa fixa (ms) | RSS ocioso (MB) | startup frio (ms) [mín–máx] | erros deadline |
|---|---|---|---|---|---|---|---|---|---|---|---|
| node-express | 3 | 661,9 [656,5–870,3] | 1458,6 [1399,7–1712,2] | 2120,5 [2056,1–2582,5] | 1512 [1146–1526] | 1,00 [0,99–1,00] | 39,99 [38,47–50,01] | 14,03 [8,11–52,94] | 235,9 [224,5–238,0] | 399 [378–442] | 96 |
| node-fastify | 3 | 697,3 [514,4–703,9] | 1787,0 [1589,1–1824,0] | 2490,9 [2103,5–2521,3] | 1421 [1380–1920] | 0,98 [0,97–0,99] | 42,25 [32,25–46,46] | 16,31 [7,89–19,72] | 239,6 [237,1–243,8] | 438 [410–468] | 96 |
| node24-express | 3 | 669,9 [587,0–803,1] | 1644,5 [1611,8–1798,5] | 2314,4 [2198,8–2601,6] | 1476 [1239–1705] | 0,99 [0,98–1,00] | 45,26 [35,58–47,92] | 32,60 [10,10–102,06] | 354,8 [345,9–355,4] | 402 [381–467] | 96 |
| node24-fastify | 3 | 652,2 [589,3–710,3] | 1765,6 [1721,3–1821,8] | 2417,8 [2310,6–2532,2] | 1516 [1395–1681] | 0,99 [0,99–0,99] | 41,89 [37,29–44,83] | 23,59 [5,22–34,22] | 382,2 [349,4–389,6] | 431 [398–488] | 96 |
| node26-express | 3 | 680,7 [563,3–690,9] | 1723,7 [1613,3–1756,4] | 2404,4 [2176,6–2447,4] | 1460 [1415–1770] | 0,99 [0,97–0,99] | 41,61 [33,37–52,83] | 16,10 [11,13–19,34] | 268,5 [267,9–308,3] | 375 [350–399] | 96 |
| node26-fastify | 3 | 556,2 [497,3–575,5] | 1745,1 [1689,3–1792,9] | 2320,6 [2186,6–2349,1] | 1761 [1713–1947] | 0,98 [0,97–0,98] | 30,74 [29,12–34,08] | 12,45 [6,42–22,95] | 275,2 [273,8–303,0] | 378 [372–448] | 96 |
| bun-express | 3 | 841,7 [716,7–1336,6] | 1650,3 [1538,8–1952,6] | 2492,0 [2255,5–3289,2] | 1189 [672–1398] | 1,00 [0,89–1,00] | 67,43 [42,94–108,62] | 42,02 [12,42–1379,50] | 176,6 [172,1–193,4] | 447 [429–679] | 96 |
| rust-ceiling | 3 | 113,7 [113,5–158,1] | 1888,5 [1847,2–2122,6] | 2002,2 [1960,7–2280,7] | 2034 [1597–2082] | 0,24 [0,23–0,25] | 25,64 [22,83–41,06] | 8,63 [4,49–11,01] | 9,2 [8,6–9,5] | 17 [17–17] | 95 |
| rust-ceiling-full | 3 | 128,7 [95,8–139,5] | 1920,6 [1782,7–1983,1] | 2049,3 [1878,5–2122,5] | 2015 [1888–2175] | 0,26 [0,21–0,26] | 23,54 [22,18–26,00] | 5,07 [3,61–10,35] | 8,6 [8,4–9,0] | 17 [17–18] | 95 |

Latência a taxa fixa:

| Contestante | rodadas ok | taxa (req/s) | p50 (ms) | p99 (ms) | p99.9 (ms) | taxa sustentada |
|---|---|---|---|---|---|---|
| node-express | 3 | 763 | 2,24 [1,95–4,94] | 14,03 [8,11–52,94] | 23,41 [21,86–59,82] | sim |
| node-fastify | 3 | 763 | 3,15 [2,05–3,21] | 16,31 [7,89–19,72] | 27,84 [21,64–33,89] | sim |
| node24-express | 3 | 763 | 3,70 [2,90–5,96] | 32,60 [10,10–102,06] | 47,47 [29,59–110,62] | sim |
| node24-fastify | 3 | 763 | 4,39 [1,75–5,51] | 23,59 [5,22–34,22] | 40,47 [23,35–49,35] | sim |
| node26-express | 3 | 763 | 2,39 [2,07–3,61] | 16,10 [11,13–19,34] | 27,12 [25,84–32,88] | sim |
| node26-fastify | 3 | 763 | 2,61 [1,72–3,00] | 12,45 [6,42–22,95] | 30,81 [25,18–35,35] | sim |
| bun-express | 3 | 763 | 3,97 [2,21–1161,66] | 42,02 [12,42–1379,50] | 47,49 [18,49–1386,57] | NÃO |
| rust-ceiling | 3 | 763 | 1,59 [1,41–2,94] | 8,63 [4,49–11,01] | 13,26 [6,79–17,28] | sim |
| rust-ceiling-full | 3 | 763 | 1,44 [1,30–2,77] | 5,07 [3,61–10,35] | 8,46 [5,84–14,44] | sim |

Razões contra as três referências (acima de 1× = contestante melhor):

| Contestante | CPU/req vs NE | CPU/req vs NF | CPU/req vs BE | req/s core vs NE | req/s core vs NF | req/s core vs BE | p99 fixo vs NE | p99 fixo vs NF | p99 fixo vs BE |
|---|---|---|---|---|---|---|---|---|---|
| node-express | — | 1,05× | 1,27× | — | 1,06× | 1,27× | — | 1,16× | 2,99× |
| node-fastify | 0,95× | — | 1,21× | 0,94× | — | 1,20× | 0,86× | — | 2,58× |
| node24-express | 0,99× | 1,04× | 1,26× | 0,98× | 1,04× | 1,24× | 0,43× | 0,50× | 1,29× |
| node24-fastify | 1,01× | 1,07× | 1,29× | 1,00× | 1,07× | 1,28× | 0,59× | 0,69× | 1,78× |
| node26-express | 0,97× | 1,02× | 1,24× | 0,97× | 1,03× | 1,23× | 0,87× | 1,01× | 2,61× |
| node26-fastify | 1,19× | 1,25× | 1,51× | 1,16× | 1,24× | 1,48× | 1,13× | 1,31× | 3,37× |
| bun-express | 0,79× | 0,83× | — | 0,79× | 0,84× | — | 0,33× | 0,39× | — |
| rust-ceiling | 5,82× | 6,13× | 7,40× | 1,35× | 1,43× | 1,71× | 1,63× | 1,89× | 4,87× |
| rust-ceiling-full | 5,14× | 5,42× | 6,54× | 1,33× | 1,42× | 1,69× | 2,77× | 3,22× | 8,29× |

### E4 — POST /products (valid)

| Contestante | rodadas ok | CPU servidor (µs/req) | CPU Postgres (µs/req) | CPU total (µs/req) | req/s | util. da CPU 2 | p99 cap. (ms) | p99 taxa fixa (ms) | RSS ocioso (MB) | startup frio (ms) [mín–máx] | erros deadline |
|---|---|---|---|---|---|---|---|---|---|---|---|
| node-express | 3 | 306,7 [299,6–363,5] | 172,6 [171,3–221,2] | 478,0 [472,2–584,6] | 3260 [2751–3334] | 1,00 [1,00–1,00] | 18,61 [14,98–20,94] | 9,39 [7,01–17,02] | 235,9 [224,5–238,0] | 399 [378–442] | 96 |
| node-fastify | 3 | 198,1 [167,9–231,7] | 185,5 [151,2–217,9] | 383,5 [319,1–449,7] | 5040 [4227–5930] | 0,99 [0,98–1,00] | 12,26 [11,65–19,36] | 8,62 [4,07–29,92] | 239,6 [237,1–243,8] | 438 [410–468] | 96 |
| node24-express | 3 | 253,0 [252,8–331,0] | 218,8 [203,9–282,3] | 471,8 [456,6–613,4] | 3931 [2903–3939] | 0,99 [0,96–0,99] | 18,25 [15,59–27,88] | 12,09 [7,96–17,16] | 354,8 [345,9–355,4] | 402 [381–467] | 96 |
| node24-fastify | 3 | 208,0 [156,1–208,6] | 198,6 [137,6–201,6] | 406,7 [293,7–410,1] | 4771 [4692–6408] | 0,99 [0,98–1,00] | 15,55 [8,23–18,61] | 5,97 [2,01–14,91] | 382,2 [349,4–389,6] | 431 [398–488] | 96 |
| node26-express | 3 | 206,6 [189,2–255,6] | 184,7 [160,8–231,1] | 391,3 [350,0–486,8] | 4823 [3891–5262] | 0,99 [0,99–0,99] | 13,66 [13,16–15,82] | 7,39 [6,23–7,85] | 268,5 [267,9–308,3] | 375 [350–399] | 96 |
| node26-fastify | 3 | 145,8 [144,4–341,6] | 138,1 [137,1–323,6] | 283,9 [281,6–665,2] | 6848 [2520–6925] | 0,99 [0,86–1,00] | 8,92 [8,41–30,75] | 4,37 [3,24–220,48] | 275,2 [273,8–303,0] | 378 [372–448] | 96 |
| bun-express | 3 | 327,6 [327,0–401,3] | 195,6 [191,4–246,9] | 523,2 [518,5–648,2] | 3052 [2487–3052] | 1,00 [0,99–1,00] | 27,56 [27,08–29,72] | 9,65 [7,46–173,70] | 176,6 [172,1–193,4] | 447 [429–679] | 96 |
| rust-ceiling | 3 | 37,6 [35,7–57,5] | 152,2 [147,9–222,1] | 189,8 [183,6–279,6] | 20124 [7256–21798] | 0,75 [0,41–0,77] | 3,95 [2,76–12,28] | 3,11 [2,32–12,14] | 9,2 [8,6–9,5] | 17 [17–17] | 92 |
| rust-ceiling-full | 3 | 42,7 [37,3–52,4] | 162,5 [148,3–198,0] | 205,1 [185,6–250,4] | 17431 [10583–20044] | 0,74 [0,55–0,74] | 4,76 [3,73–8,64] | 3,11 [1,51–4,16] | 8,6 [8,4–9,0] | 17 [17–18] | 95 |

Latência a taxa fixa:

| Contestante | rodadas ok | taxa (req/s) | p50 (ms) | p99 (ms) | p99.9 (ms) | taxa sustentada |
|---|---|---|---|---|---|---|
| node-express | 3 | 1375 | 0,75 [0,70–1,24] | 9,39 [7,01–17,02] | 25,27 [23,64–27,66] | sim |
| node-fastify | 3 | 1375 | 1,01 [0,50–2,23] | 8,62 [4,07–29,92] | 33,27 [24,50–42,10] | sim |
| node24-express | 3 | 1375 | 1,32 [0,75–1,61] | 12,09 [7,96–17,16] | 29,70 [23,30–44,98] | sim |
| node24-fastify | 3 | 1375 | 0,86 [0,51–0,93] | 5,97 [2,01–14,91] | 19,37 [17,92–20,34] | sim |
| node26-express | 3 | 1375 | 0,73 [0,60–0,77] | 7,39 [6,23–7,85] | 23,94 [20,10–24,50] | sim |
| node26-fastify | 3 | 1375 | 0,51 [0,48–0,75] | 4,37 [3,24–220,48] | 18,28 [17,60–231,26] | sim |
| bun-express | 3 | 1375 | 0,84 [0,72–1,20] | 9,65 [7,46–173,70] | 18,72 [13,03–182,13] | sim |
| rust-ceiling | 3 | 1375 | 0,43 [0,36–1,35] | 3,11 [2,32–12,14] | 4,73 [4,13–16,83] | sim |
| rust-ceiling-full | 3 | 1375 | 0,41 [0,37–0,61] | 3,11 [1,51–4,16] | 5,88 [4,53–6,77] | sim |

Razões contra as três referências (acima de 1× = contestante melhor):

| Contestante | CPU/req vs NE | CPU/req vs NF | CPU/req vs BE | req/s core vs NE | req/s core vs NF | req/s core vs BE | p99 fixo vs NE | p99 fixo vs NF | p99 fixo vs BE |
|---|---|---|---|---|---|---|---|---|---|
| node-express | — | 0,65× | 1,07× | — | 0,65× | 1,07× | — | 0,92× | 1,03× |
| node-fastify | 1,55× | — | 1,65× | 1,55× | — | 1,65× | 1,09× | — | 1,12× |
| node24-express | 1,21× | 0,78× | 1,29× | 1,21× | 0,78× | 1,29× | 0,78× | 0,71× | 0,80× |
| node24-fastify | 1,47× | 0,95× | 1,58× | 1,46× | 0,95× | 1,56× | 1,57× | 1,45× | 1,62× |
| node26-express | 1,48× | 0,96× | 1,59× | 1,48× | 0,96× | 1,58× | 1,27× | 1,17× | 1,31× |
| node26-fastify | 2,10× | 1,36× | 2,25× | 2,10× | 1,36× | 2,24× | 2,15× | 1,97× | 2,21× |
| bun-express | 0,94× | 0,60× | — | 0,94× | 0,61× | — | 0,97× | 0,89× | — |
| rust-ceiling | 8,15× | 5,26× | 8,70× | 6,17× | 3,99× | 6,59× | 3,02× | 2,77× | 3,10× |
| rust-ceiling-full | 7,19× | 4,64× | 7,68× | 5,35× | 3,46× | 5,71× | 3,02× | 2,77× | 3,10× |

### E5 — POST /products (invalid, 3 errors)

| Contestante | rodadas ok | CPU servidor (µs/req) | CPU Postgres (µs/req) | CPU total (µs/req) | req/s | util. da CPU 2 | p99 cap. (ms) | p99 taxa fixa (ms) | RSS ocioso (MB) | startup frio (ms) [mín–máx] | erros deadline |
|---|---|---|---|---|---|---|---|---|---|---|---|
| node-express | 3 | 127,6 [122,0–187,2] | 0,3 [0,3–0,5] | 127,9 [122,3–187,8] | 7842 [5332–8196] | 1,00 [0,99–1,00] | 10,28 [8,92–16,13] | 16,88 [15,83–39,91] | 235,9 [224,5–238,0] | 399 [378–442] | 96 |
| node-fastify | 3 | 50,7 [46,1–51,4] | 0,1 [0,1–0,2] | 50,8 [46,2–51,5] | 19682 [19451–21627] | 0,99 [0,99–0,99] | 3,64 [3,19–3,66] | 2,75 [1,83–3,28] | 239,6 [237,1–243,8] | 438 [410–468] | 96 |
| node24-express | 3 | 84,0 [80,5–105,4] | 0,2 [0,2–0,3] | 84,2 [80,7–105,7] | 11898 [9376–12406] | 0,99 [0,98–0,99] | 5,95 [5,65–9,95] | 4,21 [2,87–80,31] | 354,8 [345,9–355,4] | 402 [381–467] | 96 |
| node24-fastify | 3 | 57,1 [44,3–58,4] | 0,2 [0,1–0,2] | 57,3 [44,4–58,6] | 17352 [17079–22528] | 0,99 [0,98–0,99] | 4,46 [2,75–4,90] | 3,13 [0,64–3,55] | 382,2 [349,4–389,6] | 431 [398–488] | 96 |
| node26-express | 3 | 66,3 [63,7–81,0] | 0,2 [0,2–0,2] | 66,4 [63,9–81,2] | 15045 [12294–15662] | 0,99 [0,99–0,99] | 5,22 [4,94–6,54] | 3,21 [2,83–3,22] | 268,5 [267,9–308,3] | 375 [350–399] | 95 |
| node26-fastify | 3 | 42,0 [36,3–45,7] | 0,1 [0,1–0,1] | 42,1 [36,4–45,8] | 23748 [21788–27508] | 0,99 [0,99–0,99] | 2,73 [2,21–3,64] | 2,19 [0,70–2,98] | 275,2 [273,8–303,0] | 378 [372–448] | 95 |
| bun-express | 3 | 85,4 [82,2–162,2] | 0,2 [0,2–0,6] | 85,7 [82,5–162,8] | 11695 [6042–12186] | 0,99 [0,98–1,00] | 9,64 [8,73–15,23] | 4,81 [4,75–252,18] | 176,6 [172,1–193,4] | 447 [429–679] | 96 |
| rust-ceiling | 3 | 7,6 [7,2–11,4] | 0,0 [0,0–0,0] | 7,6 [7,2–11,4] | 131505 [73488–138301] | 0,94 [0,81–0,95] | 0,41 [0,39–3,66] | 2,04 [0,84–3,65] | 9,2 [8,6–9,5] | 17 [17–17] | 93 |
| rust-ceiling-full | 3 | 7,2 [7,2–11,9] | 0,0 [0,0–0,0] | 7,2 [7,2–11,9] | 137931 [73693–138448] | 0,94 [0,85–0,95] | 0,39 [0,39–3,56] | 2,16 [1,42–3,58] | 8,6 [8,4–9,0] | 17 [17–18] | 93 |

Latência a taxa fixa:

| Contestante | rodadas ok | taxa (req/s) | p50 (ms) | p99 (ms) | p99.9 (ms) | taxa sustentada |
|---|---|---|---|---|---|---|
| node-express | 3 | 3920 | 0,75 [0,72–1,42] | 16,88 [15,83–39,91] | 24,88 [22,94–47,92] | sim |
| node-fastify | 3 | 3920 | 0,40 [0,35–0,48] | 2,75 [1,83–3,28] | 4,34 [3,59–4,43] | sim |
| node24-express | 3 | 3920 | 0,70 [0,54–1,06] | 4,21 [2,87–80,31] | 7,81 [4,12–85,34] | sim |
| node24-fastify | 3 | 3920 | 0,44 [0,36–0,53] | 3,13 [0,64–3,55] | 4,84 [1,46–5,51] | sim |
| node26-express | 3 | 3920 | 0,51 [0,48–0,52] | 3,21 [2,83–3,22] | 4,78 [4,15–6,15] | sim |
| node26-fastify | 3 | 3920 | 0,38 [0,35–0,40] | 2,19 [0,70–2,98] | 3,93 [2,12–4,40] | sim |
| bun-express | 3 | 3920 | 0,59 [0,57–12,86] | 4,81 [4,75–252,18] | 11,85 [9,00–260,16] | sim |
| rust-ceiling | 3 | 3920 | 0,16 [0,13–0,17] | 2,04 [0,84–3,65] | 3,26 [2,90–7,25] | sim |
| rust-ceiling-full | 3 | 3920 | 0,17 [0,16–0,20] | 2,16 [1,42–3,58] | 3,60 [3,42–5,30] | sim |

Razões contra as três referências (acima de 1× = contestante melhor):

| Contestante | CPU/req vs NE | CPU/req vs NF | CPU/req vs BE | req/s core vs NE | req/s core vs NF | req/s core vs BE | p99 fixo vs NE | p99 fixo vs NF | p99 fixo vs BE |
|---|---|---|---|---|---|---|---|---|---|
| node-express | — | 0,40× | 0,67× | — | 0,40× | 0,67× | — | 0,16× | 0,28× |
| node-fastify | 2,52× | — | 1,68× | 2,51× | — | 1,68× | 6,14× | — | 1,75× |
| node24-express | 1,52× | 0,60× | 1,02× | 1,52× | 0,60× | 1,02× | 4,01× | 0,65× | 1,14× |
| node24-fastify | 2,23× | 0,89× | 1,50× | 2,21× | 0,88× | 1,48× | 5,40× | 0,88× | 1,54× |
| node26-express | 1,92× | 0,77× | 1,29× | 1,92× | 0,76× | 1,29× | 5,26× | 0,86× | 1,50× |
| node26-fastify | 3,04× | 1,21× | 2,03× | 3,03× | 1,21× | 2,03× | 7,72× | 1,26× | 2,20× |
| bun-express | 1,49× | 0,59× | — | 1,49× | 0,59× | — | 3,51× | 0,57× | — |
| rust-ceiling | 16,83× | 6,69× | 11,27× | 16,77× | 6,68× | 11,24× | 8,28× | 1,35× | 2,36× |
| rust-ceiling-full | 17,63× | 7,01× | 11,81× | 17,59× | 7,01× | 11,79× | 7,81× | 1,27× | 2,22× |

### E6 — GET /reports/sales-by-category (7 days)

| Contestante | rodadas ok | CPU servidor (µs/req) | CPU Postgres (µs/req) | CPU total (µs/req) | req/s | util. da CPU 2 | p99 cap. (ms) | p99 taxa fixa (ms) | RSS ocioso (MB) | startup frio (ms) [mín–máx] | erros deadline |
|---|---|---|---|---|---|---|---|---|---|---|---|
| node-express | 3 | 12312,4 [12227,7–13180,0] | 4553,1 [4314,1–4918,1] | 16865,5 [16541,8–18098,2] | 84 [79–85] | 1,00 [1,00–1,00] | 543,19 [523,23–558,66] | 303,93 [241,38–370,84] | 235,9 [224,5–238,0] | 399 [378–442] | 96 |
| node-fastify | 3 | 11626,0 [11538,5–12699,6] | 4385,0 [4062,5–4727,8] | 16011,1 [15601,0–17427,4] | 89 [82–90] | 1,00 [1,00–1,00] | 489,24 [487,62–553,36] | 259,86 [255,09–330,99] | 239,6 [237,1–243,8] | 438 [410–468] | 96 |
| node24-express | 3 | 13132,4 [12267,2–13441,3] | 5070,8 [4781,5–5174,3] | 18306,7 [17048,7–18512,1] | 79 [77–85] | 1,00 [0,99–1,00] | 561,37 [509,93–650,46] | 267,74 [262,86–389,94] | 354,8 [345,9–355,4] | 402 [381–467] | 96 |
| node24-fastify | 3 | 12048,2 [10774,2–12877,4] | 4522,2 [3840,1–4863,0] | 16570,4 [14614,3–17740,4] | 86 [81–96] | 1,00 [1,00–1,00] | 516,51 [449,89–566,47] | 261,02 [222,25–275,63] | 382,2 [349,4–389,6] | 431 [398–488] | 96 |
| node26-express | 3 | 12481,3 [11933,2–13927,6] | 4439,2 [4169,4–5229,8] | 16920,5 [16102,6–19157,3] | 83 [75–87] | 1,00 [1,00–1,00] | 532,39 [527,42–656,03] | 249,74 [204,41–264,59] | 268,5 [267,9–308,3] | 375 [350–399] | 96 |
| node26-fastify | 3 | 11733,0 [11170,6–11846,2] | 4118,5 [3943,3–4145,2] | 15851,5 [15113,8–15991,4] | 89 [88–93] | 1,00 [1,00–1,00] | 500,55 [464,11–508,99] | 241,53 [207,67–249,61] | 275,2 [273,8–303,0] | 378 [372–448] | 96 |
| bun-express | 3 | 10000,0 [9625,7–13111,7] | 4143,8 [3896,4–6013,7] | 14143,8 [13522,1–19125,4] | 103 [78–107] | 1,00 [0,98–1,00] | 408,49 [392,74–582,09] | 152,21 [148,78–347,19] | 176,6 [172,1–193,4] | 447 [429–679] | 96 |
| rust-ceiling | 3 | 670,2 [582,4–689,7] | 5957,2 [5744,3–6089,9] | 6627,3 [6326,7–6779,6] | 669 [637–692] | 0,44 [0,40–0,45] | 58,64 [57,82–73,57] | 7,38 [6,45–15,45] | 9,2 [8,6–9,5] | 17 [17–17] | 96 |
| rust-ceiling-full | 3 | 1342,1 [1317,5–1785,4] | 5841,6 [5778,9–6430,8] | 7183,7 [7096,4–8216,2] | 636 [508–642] | 0,85 [0,84–0,90] | 74,16 [66,34–94,42] | 9,63 [7,80–12,55] | 8,6 [8,4–9,0] | 17 [17–18] | 96 |

Latência a taxa fixa:

| Contestante | rodadas ok | taxa (req/s) | p50 (ms) | p99 (ms) | p99.9 (ms) | taxa sustentada |
|---|---|---|---|---|---|---|
| node-express | 3 | 42 | 12,68 [12,58–13,82] | 303,93 [241,38–370,84] | 353,65 [290,66–411,49] | sim |
| node-fastify | 3 | 42 | 13,13 [12,50–14,00] | 259,86 [255,09–330,99] | 302,37 [291,64–341,71] | sim |
| node24-express | 3 | 42 | 14,98 [12,98–15,22] | 267,74 [262,86–389,94] | 314,16 [310,43–432,91] | sim |
| node24-fastify | 3 | 42 | 12,76 [12,11–13,70] | 261,02 [222,25–275,63] | 311,47 [250,40–394,28] | sim |
| node26-express | 3 | 42 | 16,77 [12,95–21,95] | 249,74 [204,41–264,59] | 312,31 [296,36–349,28] | sim |
| node26-fastify | 3 | 42 | 12,48 [12,47–12,58] | 241,53 [207,67–249,61] | 331,68 [261,09–391,14] | sim |
| bun-express | 3 | 42 | 12,41 [11,60–15,08] | 152,21 [148,78–347,19] | 266,85 [243,33–400,76] | sim |
| rust-ceiling | 3 | 42 | 4,12 [4,11–7,71] | 7,38 [6,45–15,45] | 8,50 [7,35–20,26] | sim |
| rust-ceiling-full | 3 | 42 | 6,04 [4,58–7,49] | 9,63 [7,80–12,55] | 12,23 [10,10–14,61] | sim |

Razões contra as três referências (acima de 1× = contestante melhor):

| Contestante | CPU/req vs NE | CPU/req vs NF | CPU/req vs BE | req/s core vs NE | req/s core vs NF | req/s core vs BE | p99 fixo vs NE | p99 fixo vs NF | p99 fixo vs BE |
|---|---|---|---|---|---|---|---|---|---|
| node-express | — | 0,94× | 0,81× | — | 0,95× | 0,82× | — | 0,86× | 0,50× |
| node-fastify | 1,06× | — | 0,86× | 1,06× | — | 0,86× | 1,17× | — | 0,59× |
| node24-express | 0,94× | 0,89× | 0,76× | 0,94× | 0,89× | 0,77× | 1,14× | 0,97× | 0,57× |
| node24-fastify | 1,02× | 0,96× | 0,83× | 1,02× | 0,97× | 0,83× | 1,16× | 1,00× | 0,58× |
| node26-express | 0,99× | 0,93× | 0,80× | 0,99× | 0,93× | 0,81× | 1,22× | 1,04× | 0,61× |
| node26-fastify | 1,05× | 0,99× | 0,85× | 1,05× | 0,99× | 0,86× | 1,26× | 1,08× | 0,63× |
| bun-express | 1,23× | 1,16× | — | 1,22× | 1,16× | — | 2,00× | 1,71× | — |
| rust-ceiling | 18,37× | 17,35× | 14,92× | 7,91× | 7,49× | 6,47× | 41,17× | 35,20× | 20,62× |
| rust-ceiling-full | 9,17× | 8,66× | 7,45× | 7,52× | 7,12× | 6,15× | 31,56× | 26,99× | 15,81× |

## Memória

RSS e VmHWM somados sobre os processos do servidor (sem o launcher). Mediana [mín–máx] entre as rodadas. RSS ocioso = depois do aquecimento dos endpoints de leitura; VmHWM = pico até o fim (inclui o E4).

| Contestante | RSS ocioso pós warm-up (MB) | VmHWM no fim (MB) | RSS vs NE | RSS vs NF | RSS vs BE | VmHWM vs NE | VmHWM vs NF | VmHWM vs BE |
|---|---|---|---|---|---|---|---|---|
| node-express | 235,9 [224,5–238,0] | 244,3 [233,7–244,9] | — | 1,02× | 0,75× | — | 1,00× | 1,09× |
| node-fastify | 239,6 [237,1–243,8] | 244,7 [243,8–252,9] | 0,98× | — | 0,74× | 1,00× | — | 1,08× |
| node24-express | 354,8 [345,9–355,4] | 369,8 [350,5–397,7] | 0,66× | 0,68× | 0,50× | 0,66× | 0,66× | 0,72× |
| node24-fastify | 382,2 [349,4–389,6] | 392,3 [390,4–395,9] | 0,62× | 0,63× | 0,46× | 0,62× | 0,62× | 0,68× |
| node26-express | 268,5 [267,9–308,3] | 318,3 [300,6–325,7] | 0,88× | 0,89× | 0,66× | 0,77× | 0,77× | 0,83× |
| node26-fastify | 275,2 [273,8–303,0] | 317,9 [306,4–329,6] | 0,86× | 0,87× | 0,64× | 0,77× | 0,77× | 0,83× |
| bun-express | 176,6 [172,1–193,4] | 265,1 [263,7–267,3] | 1,34× | 1,36× | — | 0,92× | 0,92× | — |
| rust-ceiling | 9,2 [8,6–9,5] | 10,7 [10,7–11,1] | 25,74× | 26,15× | 19,27× | 22,76× | 22,80× | 24,70× |
| rust-ceiling-full | 8,6 [8,4–9,0] | 10,2 [10,1–10,8] | 27,29× | 27,72× | 20,43× | 24,04× | 24,08× | 26,09× |

## Startup

Cold start = spawn até o primeiro 200 em `/health`, com os contestantes intercalados. O startup varia muito com a carga da máquina (o Nest já foi visto entre 1,1 s e 5,1 s): por isso aparecem a dispersão e o load average, e a razão só como faixa (menor e maior razão possíveis entre os mín e máx), nunca como número único.

| Contestante | cold start (ms) | n | load average 1min | startup nas rodadas (ms) | faixa vs NE | faixa vs NF | faixa vs BE |
|---|---|---|---|---|---|---|---|
| node-express | 399 [378–442] | 10 | 4,91 [máx 5,10] | 458 [433–882] | — | 0,93× a 1,24× | 0,97× a 1,80× |
| node-fastify | 438 [410–468] | 10 | 4,91 [máx 5,10] | 540 [476–827] | 0,81× a 1,08× | — | 0,92× a 1,66× |
| node24-express | 402 [381–467] | 10 | 4,92 [máx 5,10] | 675 [459–983] | 0,81× a 1,16× | 0,88× a 1,23× | 0,92× a 1,78× |
| node24-fastify | 431 [398–488] | 10 | 4,87 [máx 5,10] | 595 [460–628] | 0,77× a 1,11× | 0,84× a 1,18× | 0,88× a 1,70× |
| node26-express | 375 [350–399] | 10 | 4,87 [máx 5,10] | 442 [392–466] | 0,95× a 1,26× | 1,03× a 1,34× | 1,07× a 1,94× |
| node26-fastify | 378 [372–448] | 10 | 4,88 [máx 5,10] | 489 [391–536] | 0,84× a 1,19× | 0,91× a 1,26× | 0,96× a 1,83× |
| bun-express | 447 [429–679] | 10 | 4,92 [máx 5,10] | 712 [625–879] | 0,56× a 1,03× | 0,60× a 1,09× | — |
| rust-ceiling | 17 [17–17] | 10 | 4,87 [máx 5,10] | 17 [17–17] | 21,98× a 26,60× | 23,84× a 28,15× | 24,96× a 40,81× |
| rust-ceiling-full | 17 [17–18] | 10 | 4,87 [máx 5,10] | 22 [17–46] | 21,15× a 26,41× | 22,94× a 27,95× | 24,01× a 40,52× |

## Ambiente e qualidade da medição

Host: 12th Gen Intel(R) Core(TM) i5-12450H, kernel 6.14.0-28-generic, Python 3.15.0. Versões: {"oha": "oha 1.16.0", "node22": "v22.21.1", "node24": "v24.21.0", "node26": "v26.11.1", "bun": "1.2.22", "rust-ceiling": {"path": "/home/gefferson-souza/.local/share/tyrus-ceiling/rust-ceiling/target/release/ceiling", "bytes": 1820048, "sha256": "cf6ed9d6b7570170425f9a2da2147083019fa913984ec8def5675baf397c9985"}, "rust-ceiling-full": {"path": "/home/gefferson-souza/.local/share/tyrus-ceiling/rust-ceiling/target-fullhydration/release/ceiling", "bytes": 1822624, "sha256": "1ba0809abfb1f4ce9f1d8dad88933ba7bc7ab8f4a9f14e496fbc60f45c976094"}}.

### Runtimes (tupla exata, sondada com `TZ=UTC`)

| Runtime | Contestantes | process.version | motor (versions.v8) | versions.icu | TZ (Intl) | binário |
|---|---|---|---|---|---|---|
| bun | bun-express | bun 1.2.22 (process.version de compatibilidade: v24.3.0) | JavaScriptCore, webkit 0ddf6f47af0a | 75.1 | UTC | /home/gefferson-souza/.bun/bin/bun |
| node22 | node-express, node-fastify | v22.21.1 | 12.4.254.21-node.33 | 77.1 | UTC | /home/gefferson-souza/.local/share/fnm/node-versions/v22.21.1/installation/bin/node |
| node24 | node24-express, node24-fastify | v24.21.0 | 13.6.233.17-node.53 | 78.3 | UTC | /home/gefferson-souza/.local/share/fnm/node-versions/v24.21.0/installation/bin/node |
| node26 | node26-express, node26-fastify | v26.11.1 | 14.6.202.34-node.37 | 78.3 | UTC | /home/gefferson-souza/.local/share/fnm/node-versions/v26.11.1/installation/bin/node |

### Ambiente dos servidores

`tools/serve.sh` remove `TYPEORM_LOGGING`, `NODE_OPTIONS`, `DEBUG`, `NODE_DEBUG`, `UV_THREADPOOL_SIZE` do ambiente de todo contestante. Ambiente que o servidor recebe (execução a seco do `serve.sh`; — = não definida):

| Contestante | TZ | NODE_ENV | TYPEORM_LOGGING | NODE_OPTIONS | DEBUG | NODE_DEBUG | UV_THREADPOOL_SIZE |
|---|---|---|---|---|---|---|---|
| node-express | `UTC` | `production` | — | — | — | — | — |
| node-fastify | `UTC` | `production` | — | — | — | — | — |
| node24-express | `UTC` | `production` | — | — | — | — | — |
| node24-fastify | `UTC` | `production` | — | — | — | — | — |
| node26-express | `UTC` | `production` | — | — | — | — | — |
| node26-fastify | `UTC` | `production` | — | — | — | — | — |
| bun-express | `UTC` | `production` | — | — | — | — | — |
| rust-ceiling | `UTC` | — | — | — | — | — | — |
| rust-ceiling-full | `UTC` | — | — | — | — | — | — |

Verificado no processo vivo (`/proc/<pid>/environ`) em 27 inicialização(ões) de servidor: 0 com variável auditada definida.

### Carga da máquina e do gerador

Load average de 1 min e frequência da CPU 2 medidos junto de cada medição (mediana [mín–máx] sobre todas as medições do contestante). `util. do core` máx = CPU usada pelo servidor ÷ tempo de parede (perto de 1 = o core 2 limitou; bem abaixo = o gargalo foi outro, como Postgres ou oha). `oha máx` = núcleos ocupados pelo oha (de 4).

| Contestante | load average 1min | CPU 2 (MHz) | util. do core máx | oha máx (núcleos) |
|---|---|---|---|---|
| node-express | 4,66 [1,46–6,01] | 4088 [3691–4384] | 1,00 | 0,38 |
| node-fastify | 4,83 [2,64–8,84] | 3944 [3362–4369] | 1,00 | 1,33 |
| node24-express | 6,06 [4,74–8,03] | 3828 [3500–4100] | 1,00 | 0,85 |
| node24-fastify | 6,31 [3,07–7,54] | 3818 [3364–4225] | 1,00 | 1,23 |
| node26-express | 5,14 [2,51–6,40] | 3840 [3500–4198] | 1,00 | 0,89 |
| node26-fastify | 4,33 [2,18–6,12] | 3825 [3217–4400] | 1,00 | 1,48 |
| bun-express | 4,45 [2,92–8,44] | 4044 [3450–4362] | 1,00 | 0,95 |
| rust-ceiling | 8,94 [5,89–12,90] | 3604 [1177–4000] | 0,95 | 2,93 |
| rust-ceiling-full | 7,65 [4,95–12,61] | 3600 [691–4100] | 0,95 | 2,93 |

## Falhas

Nenhuma falha de medição registrada.

## Reprodução

`/home/gefferson-souza/.local/share/tyrus-ceiling/harness/bench.py --rounds 3 --contestants node-express,node-fastify,node24-express,node24-fastify,node26-express,node26-fastify,bun-express,rust-ceiling,rust-ceiling-full --rust-bin /home/gefferson-souza/.local/share/tyrus-ceiling/rust-ceiling/target/release/ceiling --rust-bin-full /home/gefferson-souza/.local/share/tyrus-ceiling/rust-ceiling/target-fullhydration/release/ceiling`
