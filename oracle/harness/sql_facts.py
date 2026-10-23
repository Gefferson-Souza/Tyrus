"""Fatos sobre o SQL que Nest + TypeORM + node-pg realmente enviam, verificados por asserção sobre um traço.

Porte do `capture_sql.py` do Gate 0 para os ids de `oracle/apps/replica/requests.json`: cada fato é uma
asserção, não prosa; um fato que deixa de valer reprova.
"""
from __future__ import annotations

import json
import re
from collections import OrderedDict
from typing import Any, Callable

Fact = tuple[str, bool]
ENTITY_COLS = ("sku", "name", "description", "price", "stock", "active", "categoryId")
COLUMN_OF = {"sku": "sku", "name": "name", "description": "description", "price": "price", "stock": "stock",
             "active": "active", "categoryId": "category_id"}
INSERT_COLS = ("sku", "name", "description", "price", "stock", "active", "category_id", "created_at", "updated_at")
RETURNING = '"id", "active", "created_at", "updated_at"'


def normalize_sql(sql: str) -> tuple[str, dict[str, Any]]:
    """Modelo do SQL mais os literais que o TypeORM embute (LIMIT/OFFSET e listas IN)."""
    literals: dict[str, Any] = {}
    normalized = sql
    if " OFFSET " in sql:
        normalized = re.sub(r"LIMIT (\d+)", lambda m: literals.update(limit=int(m.group(1))) or "LIMIT <limit>", normalized)
        normalized = re.sub(r"OFFSET (\d+)", lambda m: literals.update(offset=int(m.group(1))) or "OFFSET <offset>", normalized)
    normalized = re.sub(r"IN \((\d+(?:, \d+)*)\)", lambda m: literals.update(ids=m.group(1)) or "IN (<ids>)", normalized)
    return normalized, literals


def predict_key_order(body: dict[str, Any]) -> list[str]:
    keys = [k for k in ENTITY_COLS if k in body]
    if "description" not in body:
        keys.append("description")
    keys.append("id")
    if "active" not in body:
        keys.append("active")
    return keys + ["createdAt", "updatedAt"]


def _text_param(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, float) and value == int(value):
        return str(int(value))
    return value if isinstance(value, str) else repr(value)


def predict_insert(body: dict[str, Any]) -> tuple[str, list[str | None]]:
    by_column = {column: key for key, column in COLUMN_OF.items()}
    values, params = [], []
    for column in INSERT_COLS:
        if column in by_column and by_column[column] in body:
            params.append(_text_param(body[by_column[column]]))
            values.append(f"${len(params)}")
        else:
            values.append("DEFAULT")
    columns = ", ".join(f'"{c}"' for c in INSERT_COLS)
    return f'INSERT INTO "product"({columns}) VALUES ({", ".join(values)}) RETURNING {RETURNING}', params


def _protocols(entry: dict[str, Any]) -> list[str]:
    return [t["protocol"] for t in entry["trips"]]


def _shape_facts(requests: dict[str, Any]) -> list[Fact]:
    def protocols(rid: str) -> list[str]:
        return _protocols(requests[rid])

    return [
        ("E1: 0 round trips", protocols("01-health") == []),
        ("E5: 0 round trips", protocols("36-post-invalid-loadtest") == []),
        ("E6 invalid from: 0 round trips", protocols("33-report-invalid-from") == []),
        ("E2 found: 2 extended", protocols("04-product-17") == ["extended", "extended"]),
        ("E2 not found: 1 extended", protocols("17-product-10001") == ["extended"]),
        ("E3: 2 simple, no parameters",
         all(protocols(r) == ["simple", "simple"] and not any(t["params"] for t in requests[r]["trips"])
             for r in ("20-list-defaults", "21-list-5-37", "22-list-100-9950"))),
        ("E3 offset past the end: only the ids query runs", protocols("23-list-20-20000") == ["simple"]),
        ("E6: 1 extended, 2 parameters",
         protocols("29-report-7d") == ["extended"] and len(requests["29-report-7d"]["trips"][0]["params"]) == 2),
    ]


def _transaction_facts(requests: dict[str, Any]) -> list[Fact]:
    ok = requests["45-post-valid-loadtest"]["trips"]
    fk = requests["54-post-fk-500"]["trips"]
    return [
        ("E4: START TRANSACTION (simple), INSERT (extended), COMMIT (simple)",
         len(ok) == 3 and [(t["protocol"], t["sql"]) for t in (ok[0], ok[2])]
         == [("simple", "START TRANSACTION"), ("simple", "COMMIT")]
         and ok[1]["protocol"] == "extended" and ok[1]["sql"].startswith("INSERT INTO")),
        ("E4 FK violation: ends with ROLLBACK, SQLSTATE 23503",
         len(fk) == 3 and fk[2]["sql"] == "ROLLBACK" and fk[1]["errors"] == ["23503"]),
    ]


def _protocol_facts(trips: list[dict[str, Any]]) -> list[Fact]:
    extended = [t for t in trips if t["protocol"] == "extended"]
    return [
        ("extended trips are exactly Parse, Bind, Describe, Execute, Sync", all(t["kinds"] == "PBDES" for t in extended)),
        ("Describe targets the portal", all(t["describe"] == "portal" for t in extended)),
        ("Parse carries no parameter OIDs", all(t["parse_oids"] == [] for t in extended)),
        ("parameters and results are all text format",
         all(set(t["param_formats"]) <= {0} and t["result_formats"] in ([0], []) for t in extended)),
        ("every simple trip is a single Q message", all(t["kinds"] == "Q" for t in trips if t["protocol"] == "simple")),
        ("all requests reuse one pooled connection", len({t["connection"] for t in trips}) == 1),
    ]


def _post_rule_facts(requests: dict[str, Any], definitions: list[dict[str, Any]]) -> list[Fact]:
    bodies = [d for d in definitions if d["method"] == "POST" and "body" in d and d["expect_status"] == 201
              and d["path"] == "/products"]
    keys_ok = insert_ok = 0
    for definition in bodies:
        entry = requests[definition["id"]]
        keys = list(json.loads(entry["http"]["body"], object_pairs_hook=OrderedDict))
        keys_ok += keys == predict_key_order(definition["body"])
        sql, params = predict_insert(definition["body"])
        insert = next((t for t in entry["trips"] if t["sql"].startswith("INSERT")), None)
        insert_ok += bool(insert) and insert["sql"] == sql and insert["params"] == params
    count = len(bodies)
    return [(f"POST key-order rule predicts the response key order ({count} bodies)", count > 0 and keys_ok == count),
            (f"INSERT shape rule predicts SQL text and parameters ({count} bodies)", count > 0 and insert_ok == count)]


def check_facts(trace: dict[str, Any], definitions: list[dict[str, Any]]) -> list[Fact]:
    requests = trace["requests"]
    trips = [t for entry in requests.values() for t in entry["trips"]]
    groups: list[Callable[[], list[Fact]]] = [
        lambda: _shape_facts(requests), lambda: _transaction_facts(requests),
        lambda: _protocol_facts(trips), lambda: _post_rule_facts(requests, definitions)]
    facts: list[Fact] = []
    for group in groups:
        for name, ok in group():
            facts.append((name, bool(ok)))
    return facts
