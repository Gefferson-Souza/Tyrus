# Equivalence gate results

Generated 2026-10-20T13:10:11+00:00 by `harness/equiv.py report`.

- Canonical request set: `harness/requests.json`, 54 requests, sha1 `14937a0a3abfa4c7239e7ef61ebcac305abd99e3`.
- Golden: `golden/golden.json`, recorded from node-express on 2026-10-14T13:11:33.382211+00:00 (node v22.21.1, nest-app lock sha1 `34bf8a3fc9ed39b6c1da8114690c868c33ff5566`).
- Comparison: `strict` = status + body bytes + `Content-Type`, `Content-Length`, `ETag`, `X-Powered-By`; `body` (the three fastify contestants) = status + body bytes only. `Date`, `Connection`, `Keep-Alive` are ignored.
- Masking, and only this: `createdAt`/`updatedAt` in successful POST responses are compared by format `^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$`; their `ETag` and `Content-Length` are checked for format and for consistency with the actual body (Express weak-ETag formula, byte length).
- DB-state check after the requests: row counts, sequence `last_value`/`is_called`, checksum of every seeded row (also against the template database), the inserted rows field by field (timestamps by format, `created_at == updated_at`), and each 2xx POST response against the DB row it created.

## Verdicts

node-express ran twice in the latest `equiv.py all` (labels `node-express` and `node-express-rerun`); both are compared strictly with the golden, so the two runs are identical to each other up to the masked POST timestamps.

| Contestant (label) | Mode | Requests | DB state | Startup (s) | Verdict |
|---|---|---|---|---|---|
| node-express (node-express) | strict | 54/54 | OK | 0.46 | **PASS** |
| node-express (node-express-rerun) | strict | 54/54 | OK | 0.407 | **PASS** |
| node-fastify (node-fastify) | body | 54/54 | OK | 0.508 | **PASS** |
| node24-express (node24-express) | strict | 54/54 | OK | 0.458 | **PASS** |
| node24-fastify (node24-fastify) | body | 54/54 | OK | 0.457 | **PASS** |
| node26-express (node26-express) | strict | 54/54 | OK | 0.459 | **PASS** |
| node26-fastify (node26-fastify) | body | 54/54 | OK | 0.558 | **PASS** |
| bun-express (bun-express) | strict | 54/54 | OK (documented divergence) | 0.558 | **PASS*** |
| node-cluster (node-cluster) | strict | 54/54 | OK | 1.066 | **PASS** |
| rust-ceiling (rust-ceiling) | strict | 54/54 | OK | 0.051 | **PASS** |
| rust-ceiling-full (rust-ceiling-full) | strict | 54/54 | OK | 0.051 | **PASS** |

## Runtimes and artifacts of the gated runs (what `bench.py` binds to)

| Contestant (label) | Runtime (process.version / v8 / icu / tz) | Artifact | sha256 | Live process audit |
|---|---|---|---|---|
| node-express (node-express) | v22.21.1 / 12.4.254.21-node.33 / 77.1 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| node-express (node-express-rerun) | v22.21.1 / 12.4.254.21-node.33 / 77.1 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| node-fastify (node-fastify) | v22.21.1 / 12.4.254.21-node.33 / 77.1 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| node24-express (node24-express) | v24.21.0 / 13.6.233.17-node.53 / 78.3 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| node24-fastify (node24-fastify) | v24.21.0 / 13.6.233.17-node.53 / 78.3 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| node26-express (node26-express) | v26.11.1 / 14.6.202.34-node.37 / 78.3 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| node26-fastify (node26-fastify) | v26.11.1 / 14.6.202.34-node.37 / 78.3 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| bun-express (bun-express) | v24.3.0 / 13.6.233.10-node.18 / 75.1 / UTC (bun 1.2.22) | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| node-cluster (node-cluster) | v22.21.1 / 12.4.254.21-node.33 / 77.1 / UTC | nest-dist | `e8acfad60d829b80` | clean env, runtime ok |
| rust-ceiling (rust-ceiling) | - | rust-binary | `cf6ed9d6b7570170` | clean env, runtime ok |
| rust-ceiling-full (rust-ceiling-full) | - | rust-binary | `1ba0809abfb1f4ce` | clean env, runtime ok |

## Gate self-test

96/96 controls behaved as expected (79 negative-style, 17 positive-style). Details: `results/selftest.md` (includes mutants of the gate itself, each must be killed).

## SQL on the wire (`harness/sql_equiv.py`)

- bun-express: Verdict: **PASS*** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- node-express: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- node-fastify: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- node24-express: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- node24-fastify: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- node26-express: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- node26-fastify: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- rust-ceiling-full: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).
- rust-ceiling: Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).

## Contestant x request

| # | Request | Expect | node-express | node-express-rerun | node-fastify | node24-express | node24-fastify | node26-express | node26-fastify | bun-express | node-cluster | rust-ceiling | rust-ceiling-full |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `GET /health` (health) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 2 | `GET /products/1` (product-1) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 3 | `GET /products/2` (product-2) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 4 | `GET /products/17` (product-17) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 5 | `GET /products/9999` (product-9999) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 6 | `GET /products/10000` (product-10000) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 7 | `GET /products/3` (product-3) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 8 | `GET /products/4` (product-4) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 9 | `GET /products/6` (product-6) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 10 | `GET /products/7` (product-7) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 11 | `GET /products/10` (product-10) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 12 | `GET /products/16` (product-16) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 13 | `GET /products/35` (product-35) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 14 | `GET /products/165` (product-165) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 15 | `GET /products/007` (product-007) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 16 | `GET /products/1.5` (product-1.5) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 17 | `GET /products/10001` (product-10001) | 404 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 18 | `GET /products/-5` (product-minus5) | 404 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 19 | `GET /products/abc` (product-abc) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 20 | `GET /products` (list-defaults) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 21 | `GET /products?limit=5&offset=37` (list-5-37) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 22 | `GET /products?limit=100&offset=9950` (list-100-9950) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 23 | `GET /products?limit=20&offset=20000` (list-20-20000) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 24 | `GET /products?offset=100` (list-offset-only) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 25 | `GET /products?limit=0` (list-limit-0) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 26 | `GET /products?limit=101` (list-limit-101) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 27 | `GET /products?limit=x` (list-limit-x) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 28 | `GET /products?limit=5&offset=-1` (list-offset-neg) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 29 | `GET /reports/sales-by-category?from=2026-03-01T00:00:00.000Z&to=2026-03-08` (report-7d) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 30 | `GET /reports/sales-by-category?from=2030-01-01T00:00:00.000Z&to=2030-01-02` (report-empty) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 31 | `GET /reports/sales-by-category?from=2026-03-01T00:00:00Z&to=2026-03-02T00:` (report-1d) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 32 | `GET /reports/sales-by-category?from=2026-06-10&to=2026-06-11` (report-date-only) | 200 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 33 | `GET /reports/sales-by-category?from=nope&to=2026-03-02T00:00:00.000Z` (report-invalid-from) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 34 | `GET /reports/sales-by-category?from=nope&to=also-nope` (report-both-invalid) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 35 | `GET /reports/sales-by-category` (report-no-params) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 36 | `POST /products` (post-invalid-loadtest) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 37 | `POST /products` (post-invalid-missing-fields) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 38 | `POST /products` (post-invalid-wrong-types) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 39 | `POST /products` (post-invalid-too-short) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 40 | `POST /products` (post-invalid-negatives) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 41 | `POST /products` (post-invalid-extra-field) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 42 | `POST /products` (post-invalid-emoji-length) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 43 | `POST /products` (post-invalid-name-121) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 44 | `POST /products` (post-invalid-description-2001) | 400 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 45 | `POST /products` (post-valid-loadtest) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 46 | `POST /products` (post-valid-minimal) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 47 | `POST /products` (post-valid-active-false) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 48 | `POST /products` (post-valid-extra-field) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 49 | `POST /products` (post-valid-description-null) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 50 | `POST /products` (post-valid-reversed-keys) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 51 | `POST /products` (post-valid-emoji-name) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 52 | `POST /products` (post-valid-lone-surrogate) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 53 | `POST /products` (post-valid-price-100.0) | 201 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |
| 54 | `POST /products` (post-fk-500) | 500 | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok | ok |

`PASS*` = every check passed except exact, documented divergences listed below; `FAIL` = anything else.

## Documented divergences

- **bun-express** (2 excused failures):
  - `inserted product 10008 field name: golden 'a�b' vs actual 'a�'`
  - `52-post-valid-lone-surrogate: response name='a�b' but the DB row has 'a�'`
  - reason: Bun 1.2.22 reports Buffer.byteLength('a\ud800b') = 4 although the UTF-8 encoding (the lone surrogate becomes U+FFFD) takes 5 bytes. node-pg uses that length to declare the Bind parameter, so Bun sends 61 ef bf bd (4 bytes) and Postgres stores 'a\ufffd': the trailing 'b' is lost. Node sends 61 ef bf bd 62. The HTTP response of bun-express is still byte-identical (save() echoes the in-memory entity, only id/active/created_at/updated_at come back from RETURNING).
  - evidence: wire bytes captured with tools/pgproxy.py (Node: declared length 5, 61 ef bf bd 62; Bun: declared length 4, 61 ef bf bd); reproduction: PROBE_URL=postgres://postgres:ceiling@127.0.0.1:<proxy>/postgres bun tools/probe-lone-surrogate.js (and the same with node) prints Buffer.byteLength and the stored bytes

## Failures

None.

## Sanity of the request set against the golden

Every golden status equals the `expect_status` declared in requests.json.

## Golden DB state

- counts {'category': 20, 'product': 10009, 'order_item': 50000}; sequences {'product_id_seq': {'last_value': 10010, 'is_called': True}, 'order_item_id_seq': {'last_value': 50000, 'is_called': True}, 'category_id_seq': {'last_value': 20, 'is_called': True}}
- inserted rows: [10001, 10002, 10003, 10004, 10005, 10006, 10007, 10008, 10009]
