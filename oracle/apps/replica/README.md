# Réplica de desenvolvimento (Nest 10.4 + TypeORM + pg)

App de teste visível aos agentes do Tyrus. As rotas E1 a E6 do Gate 0 são byte a byte as do `nest-app` do Gate 0
(`gate0-parity.sha256`); o resto exercita o que um backend Nest real usa: `ValidationPipe`, `ParseIntPipe`, guard,
interceptor, filtro de exceção, decorator de parâmetro, DTO aninhado, `@nestjs/throttler`.

| Rota | O que é |
|---|---|
| `GET /health`, `GET /products/:id`, `GET /products`, `POST /products`, `GET /reports/sales-by-category` | E1 a E6 do Gate 0 |
| `GET /throttled/ping` (5 por 60 s), `/throttled/strict` (2 por 60 s), `/throttled/skipped` | throttler; só estas rotas são limitadas |
| `GET /extras/whoami`, `/extras/items/:id`, `/extras/domain-error`, `POST /extras/validate` | guard (`x-api-key: replica-key`), interceptor, filtro, pipes |

## Comandos (Node 26.11.1 exato)

```sh
npm ci && npm run build            # dist/emit (tsc por arquivo) e dist/bundle (webpack, dependências externas)
db/up.sh                           # Postgres 17 por digest, porta 55493, tmpfs
python3 -I run_requests.py run --target "node dist/bundle/main.js" --out run.json --twice --golden golden/express-node-26.11.1.json
tools/verify_replica.sh            # tudo acima mais árvore de produção, Fastify e 1000 requisições a E2
```

Ambiente do alvo: `DATABASE_URL`, `PORT`, `LISTEN_HOST`, `ADAPTER=express|fastify`, `TYPEORM_LOGGING=1` (SQL no log).
`golden/express-node-26.11.1.json` é regressão da própria réplica (Node contra Node), não recibo de equivalência.
Fastify é baseline de desempenho: só se compara no nível `body`.
