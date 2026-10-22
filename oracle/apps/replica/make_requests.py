"""Gera requests.json: o conjunto canônico de requisições da réplica.

As requisições 01 a 54 vêm do Gate 0 sem alteração de ordem, rota ou corpo. As seguintes (55 em diante)
cobrem 404, JSON malformado, o módulo extras (guard, interceptor, filtro, decorator de parâmetro, pipes) e o throttler.
A ordem importa: toda leitura que depende do seed roda antes do primeiro POST válido.
"""
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional

BASE = {"name": "Teste Produto Alfa", "sku": "TST-0001", "price": 12.5, "stock": 3, "categoryId": 2}
LOADTEST_E4 = {"name": "Banana Prata 1kg", "sku": "BANANA-1KG", "description": "Penca com 6 unidades",
               "price": 7.9, "stock": 120, "categoryId": 2}
LOADTEST_E5 = {"name": "ab", "sku": "bad sku", "description": "x", "price": 10.505, "stock": 5, "categoryId": 1}
API_KEY = {"x-api-key": "replica-key"}
RETRY_WINDOW = {"retry-after": [1, 60]}

reqs: list[dict[str, Any]] = []


@dataclass(frozen=True)
class RequestSpec:
    """Uma requisição do conjunto canônico; os campos opcionais seguem a ordem do Gate 0."""

    name: str
    endpoint: str
    method: str
    path: str
    status: int
    body: Any = None
    body_raw: Optional[str] = None
    note: Optional[str] = None
    headers: Optional[dict[str, str]] = None
    volatile: Optional[dict[str, list[int]]] = None


def to_request(spec: RequestSpec, number: int) -> dict[str, Any]:
    r: dict[str, Any] = {"id": f"{number:02d}-{spec.name}", "endpoint": spec.endpoint, "method": spec.method,
                         "path": spec.path, "expect_status": spec.status}
    if spec.body is not None:
        r["body"] = spec.body
    if spec.body_raw is not None:
        r["body_raw"] = spec.body_raw
    if spec.headers:
        r["headers"] = spec.headers
    if spec.volatile:
        r["volatile_headers"] = spec.volatile
    if spec.method == "POST" and spec.status == 201:
        r["mask_timestamps"] = True
    if spec.note:
        r["note"] = spec.note
    return r


def add(*args: Any, **kwargs: Any) -> None:
    """Acrescenta uma requisição; os argumentos são os campos de RequestSpec (limite de 5 parâmetros do R4)."""
    reqs.append(to_request(RequestSpec(*args, **kwargs), len(reqs) + 1))


# E1
add("health", "E1", "GET", "/health", 200)
# E2: found
for pid, why in [(1, "null description"), (2, "null description"), (17, "plain"), (9999, "plain"), (10000, "last seeded id"),
                 (3, "description with backslash"), (4, "inactive product"), (6, "description with tab"),
                 (7, "description with emoji"), (10, "description with newline"), (16, "description with quotes and em dash"),
                 (35, "description with < and &"), (165, "emoji in the name")]:
    add(f"product-{pid}", "E2", "GET", f"/products/{pid}", 200, note=why)
add("product-007", "E2", "GET", "/products/007", 200, note="ParseIntPipe accepts leading zeros: parsed as 7")
add("product-1.5", "E2", "GET", "/products/1.5", 400, note="ParseIntPipe rejects decimals")
add("product-10001", "E2", "GET", "/products/10001", 404, note="first id past the seed")
add("product-minus5", "E2", "GET", "/products/-5", 404)
add("product-abc", "E2", "GET", "/products/abc", 400)
# E3
add("list-defaults", "E3", "GET", "/products", 200)
add("list-5-37", "E3", "GET", "/products?limit=5&offset=37", 200)
add("list-100-9950", "E3", "GET", "/products?limit=100&offset=9950", 200)
add("list-20-20000", "E3", "GET", "/products?limit=20&offset=20000", 200, note="offset past the end: empty items")
add("list-offset-only", "E3", "GET", "/products?offset=100", 200)
add("list-limit-0", "E3", "GET", "/products?limit=0", 400)
add("list-limit-101", "E3", "GET", "/products?limit=101", 400)
add("list-limit-x", "E3", "GET", "/products?limit=x", 400)
add("list-offset-neg", "E3", "GET", "/products?limit=5&offset=-1", 400)
# E6
add("report-7d", "E6", "GET", "/reports/sales-by-category?from=2026-03-01T00:00:00.000Z&to=2026-03-08T00:00:00.000Z", 200)
add("report-empty", "E6", "GET", "/reports/sales-by-category?from=2030-01-01T00:00:00.000Z&to=2030-01-02T00:00:00.000Z", 200)
add("report-1d", "E6", "GET", "/reports/sales-by-category?from=2026-03-01T00:00:00Z&to=2026-03-02T00:00:00Z", 200,
    note="ISO 8601 without milliseconds; the echo keeps the raw strings")
add("report-date-only", "E6", "GET", "/reports/sales-by-category?from=2026-06-10&to=2026-06-11", 200, note="date-only ISO 8601")
add("report-invalid-from", "E6", "GET", "/reports/sales-by-category?from=nope&to=2026-03-02T00:00:00.000Z", 400)
add("report-both-invalid", "E6", "GET", "/reports/sales-by-category?from=nope&to=also-nope", 400)
add("report-no-params", "E6", "GET", "/reports/sales-by-category", 400)
# E5 and the other invalid POSTs (no DB access)
add("post-invalid-loadtest", "E5", "POST", "/products", 400, body=LOADTEST_E5, note="the fixed 3-error body used by the load test")
add("post-invalid-missing-fields", "E5", "POST", "/products", 400, body={})
add("post-invalid-wrong-types", "E5", "POST", "/products", 400,
    body={"name": 123, "sku": 456, "description": 42, "price": "12.5", "stock": "3", "categoryId": "2", "active": "yes"})
add("post-invalid-too-short", "E5", "POST", "/products", 400, body={"name": "ab", "sku": "tst", "price": 1, "stock": 1, "categoryId": 1})
add("post-invalid-negatives", "E5", "POST", "/products", 400, body={"name": "Teste", "sku": "TST-0002", "price": -1, "stock": -1, "categoryId": 0})
add("post-invalid-extra-field", "E5", "POST", "/products", 400,
    body={"name": "ab", "sku": "TST-0001", "price": 1, "stock": 1, "categoryId": 1, "isAdmin": True})
add("post-invalid-emoji-length", "E5", "POST", "/products", 400,
    body={"name": "\U0001F600\U0001F600", "sku": "TST-0003", "price": 1, "stock": 1, "categoryId": 1},
    note="2 code points (4 UTF-16 units): Length rejects by code points but picks the message by UTF-16 length")
add("post-invalid-name-121", "E5", "POST", "/products", 400, body={**BASE, "name": "n" * 121})
add("post-invalid-description-2001", "E5", "POST", "/products", 400, body={**BASE, "description": "d" * 2001})
# E4: valid POSTs (they change the DB, so they come after every read of seed data)
add("post-valid-loadtest", "E4", "POST", "/products", 201, body=LOADTEST_E4, note="the fixed valid body used by the load test")
add("post-valid-minimal", "E4", "POST", "/products", 201, body=BASE, note="no description, no active")
add("post-valid-active-false", "E4", "POST", "/products", 201, body={**BASE, "sku": "TST-0004", "active": False})
add("post-valid-extra-field", "E4", "POST", "/products", 201,
    body={**BASE, "sku": "TST-0005", "isAdmin": True}, note="non-whitelisted field is stripped")
add("post-valid-description-null", "E4", "POST", "/products", 201,
    body={**BASE, "sku": "TST-0006", "description": None}, note="null counts as provided: INSERT binds NULL")
add("post-valid-reversed-keys", "E4", "POST", "/products", 201,
    body={"active": True, "description": "Chaves em ordem inversa", "categoryId": 3, "stock": 9, "price": 0.1, "sku": "TST-0007",
          "name": "Teste Ordem das Chaves"}, note="response key order does not follow the request key order")
add("post-valid-emoji-name", "E4", "POST", "/products", 201,
    body={**BASE, "sku": "TST-0008", "name": "\U0001F600\U0001F600\U0001F600", "description": "Pão com ç, ã, é e 🍞"},
    note="3 code points: valid, stored and echoed as raw UTF-8")
add("post-valid-lone-surrogate", "E4", "POST", "/products", 201,
    body_raw='{"name":"a\\ud800b","sku":"TST-0009","price":3,"stock":1,"categoryId":4}',
    note="JS accepts the lone surrogate and echoes it as \\ud800; the DB stores U+FFFD")
add("post-valid-price-100.0", "E4", "POST", "/products", 201,
    body_raw='{"name":"Preco com ponto zero","sku":"TST-0010","price":100.0,"stock":1,"categoryId":5}',
    note="JS number formatting: 100.0 echoes as 100")
add("post-fk-500", "E4", "POST", "/products", 500, body={**BASE, "sku": "TST-0011", "categoryId": 99999},
    note="FK violation: ROLLBACK, a sequence value is consumed")

# 404 do roteador
add("not-found-get", "NF", "GET", "/nope", 404)
add("not-found-post", "NF", "POST", "/nope", 404, body={})
# JSON malformado (a mensagem vem do V8 e faz parte da impressão digital do motor)
add("malformed-json-eof", "MJ", "POST", "/products", 400, body_raw='{"name":')
add("malformed-json-key", "MJ", "POST", "/products", 400, body_raw="{bad}")
add("malformed-json-trailing-comma", "MJ", "POST", "/products", 400, body_raw='{"a":1,}')
# módulo extras: guard, decorator de parâmetro, interceptor, filtro, ParseIntPipe, ValidationPipe aninhado
EXTRAS_VALID = {"label": "  rotulo  ", "priority": "high", "contact": "a@b.co", "tags": ["x", "y"],
                "address": {"city": "Rio", "country": "BR"}}
add("extras-no-key", "EX", "GET", "/extras/whoami", 403)
add("extras-wrong-key", "EX", "GET", "/extras/whoami", 403, headers={"x-api-key": "nope"})
add("extras-whoami-anonymous", "EX", "GET", "/extras/whoami", 200, headers=API_KEY)
add("extras-whoami-client", "EX", "GET", "/extras/whoami", 200, headers={**API_KEY, "x-client-id": "client-42"})
add("extras-item-21", "EX", "GET", "/extras/items/21", 200, headers=API_KEY)
add("extras-item-abc", "EX", "GET", "/extras/items/abc", 400, headers=API_KEY)
add("extras-domain-error", "EX", "GET", "/extras/domain-error", 422, headers=API_KEY)
add("extras-validate-valid", "EX", "POST", "/extras/validate", 201, body=EXTRAS_VALID, headers=API_KEY,
    note="Transform apara o rótulo")
add("extras-validate-invalid", "EX", "POST", "/extras/validate", 400, headers=API_KEY,
    body={"label": "", "priority": "mid", "contact": "x", "tags": [1, 2, 3, 4], "address": {"city": 1}})
add("extras-validate-forbidden-first", "EX", "POST", "/extras/validate", 403, body={})
# throttler: limite global 5 por 60 s só em /throttled/ping; /throttled/strict limita 2; /throttled/skipped não limita
for n in range(1, 6):
    add(f"throttled-ping-{n}", "TH", "GET", "/throttled/ping", 200)
add("throttled-ping-6", "TH", "GET", "/throttled/ping", 429, volatile=RETRY_WINDOW)
for n in range(1, 3):
    add(f"throttled-strict-{n}", "TH", "GET", "/throttled/strict", 200)
add("throttled-strict-3", "TH", "GET", "/throttled/strict", 429, volatile=RETRY_WINDOW)
for n in range(1, 9):
    add(f"throttled-skipped-{n}", "TH", "GET", "/throttled/skipped", 200)

out = {"loadtest": {"E4": LOADTEST_E4, "E5": LOADTEST_E5,
                    "E6_path": "/reports/sales-by-category?from=2026-03-01T00:00:00.000Z&to=2026-03-08T00:00:00.000Z"},
       "requests": reqs}
TARGET = Path(__file__).with_name("requests.json")
rendered = json.dumps(out, indent=1, ensure_ascii=False) + "\n"
if __name__ == "__main__":
    import sys
    if "--check" in sys.argv:
        sys.exit(0 if TARGET.read_text(encoding="utf-8") == rendered else 1)
    TARGET.write_text(rendered, encoding="utf-8")
    print(len(reqs), "requests written")
