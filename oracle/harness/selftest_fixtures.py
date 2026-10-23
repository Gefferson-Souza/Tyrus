"""Traço sintético mínimo (sem Docker) usado pelo autoteste e pelos testes unitários do comparador."""
from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "apps" / "replica" / "harness"))
import replica_core  # noqa: E402

LONE_SURROGATE_ID = "52-post-valid-lone-surrogate"
JSON_HEADER = ("content-type", "application/json; charset=utf-8")


def trip(protocol: str, sql: str, params: tuple[str, ...] = (), errors: tuple[str, ...] = ()) -> dict[str, Any]:
    simple = protocol == "simple"
    digest = hashlib.sha256(json.dumps([sql, params]).encode()).hexdigest()
    return {"protocol": protocol, "sql": sql, "params": list(params), "kinds": "Q" if simple else "PBDES",
            "param_formats": [] if simple else [0] * len(params), "result_formats": [] if simple else [0],
            "describe": None if simple else "portal", "parse_oids": [], "client_sha256": [digest],
            "errors": list(errors), "connection": 1}


def http(rid: str, status: int, body: bytes, expect: int | None = None) -> dict[str, Any]:
    request = {"id": rid, "expect_status": status if expect is None else expect}
    headers = [JSON_HEADER, ("content-length", str(len(body))), ("etag", replica_core.weak_etag(body)),
               ("date", "Fri, 23 Oct 2026 10:00:00 GMT")]
    return replica_core.build_record(request, status, headers, body)


def entry(rid: str, status: int, body: bytes, trips: list[dict[str, Any]]) -> dict[str, Any]:
    return {"http": http(rid, status, body), "trips": trips}


def created_body(row_id: int, sku: str, category_id: int) -> bytes:
    return json.dumps({"sku": sku, "price": 12.5, "stock": 1, "categoryId": category_id, "id": row_id,
                       "active": True}, separators=(",", ":")).encode()


def base_state() -> dict[str, Any]:
    new_row = {"id": 10001, "sku": "TST-1", "name": "n", "description": None, "price": "12.50", "stock": 1,
               "active": True, "category_id": 2, "timestamps_coherent": True}
    second_row = dict(new_row, id=10002, sku="TST-0009", category_id=4)
    return {"dump_sha256": "a" * 64, "dump_bytes": 1000, "seed_products_md5": "b" * 32,
            "new_products": [new_row, second_row], "row_counts": {"product": 10001, "order_item": 50000}}


def base_trace(engine: str = "node-26.11.1") -> dict[str, Any]:
    select = 'SELECT "p"."id" FROM "product" "p" WHERE "p"."id" = $1'
    insert = 'INSERT INTO "product"("sku") VALUES ($1) RETURNING "id"'
    requests = {
        "00-startup": {"http": None, "trips": [trip("simple", "SELECT version()")]},
        "01-health": entry("01-health", 200, b'{"status":"ok"}', []),
        "02-product-1": entry("02-product-1", 200, b'{"id":1}',
                              [trip("extended", select, ("1",)), trip("extended", select, ("1",))]),
        "03-list-defaults": entry("03-list-defaults", 200, b'{"items":[]}',
                                  [trip("simple", "SELECT 1"), trip("simple", "SELECT 2")]),
        "04-post": entry("04-post", 201, created_body(10001, "TST-1", 2),
                         [trip("simple", "START TRANSACTION"), trip("extended", insert, ("TST-1",)),
                          trip("simple", "COMMIT")]),
        "05-post-fk": entry("05-post-fk", 500, b'{"statusCode":500}',
                            [trip("simple", "START TRANSACTION"),
                             trip("extended", insert, ("TST-2",), errors=("23503",)), trip("simple", "ROLLBACK")]),
        LONE_SURROGATE_ID: entry(LONE_SURROGATE_ID, 201, created_body(10002, "TST-0009", 4),
                                 [trip("simple", "START TRANSACTION"),
                                  trip("extended", insert, ("TST-0009", "a�b", "3", "1", "4")),
                                  trip("simple", "COMMIT")]),
    }
    return {"meta": {"engine": engine, "target": "synthetic", "adapter": "express", "requests": len(requests) - 1},
            "requests": requests, "state": base_state()}


def mutated(trace: dict[str, Any], mutate: Any) -> dict[str, Any]:
    clone = copy.deepcopy(trace)
    mutate(clone)
    return clone
