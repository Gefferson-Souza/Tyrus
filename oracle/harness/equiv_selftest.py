#!/usr/bin/env python3
"""Autoteste do comparador: controles positivos devem passar e negativos devem reprovar (sem Docker).

Um comparador que não consegue reprovar não prova nada. Cada regra de `wire_compare.py` é atacada em traços
sintéticos e, com `--trace`, em todas as requisições de um traço real gravado por `equiv_wire.py record`.
"""
from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any, Callable

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import normalizers  # noqa: E402
import selftest_fixtures as fx  # noqa: E402
import wire_compare  # noqa: E402

Control = tuple[str, bool, str]
GREEN = (wire_compare.IDENTICAL, wire_compare.NORMALIZED)
KNOWN = normalizers.load_known(HERE / "known_divergences.json")


def verdict(golden: dict, actual: dict, engine: str = "node-26.11.1", level: str = "bytes") -> dict[str, Any]:
    return wire_compare.compare_traces(golden, actual, KNOWN, {"engine": engine, "wire_level": level})


def attempt(name: str, mutate: Callable[[dict], None], expect_green: bool, **options: str) -> Control:
    base = fx.base_trace()
    report = verdict(base, fx.mutated(base, mutate), **options)
    first = next((f for layer in report["layers"].values() for f in layer["fails"]), "")
    return name, report["green"] == expect_green, first[:90]


def req(trace: dict, rid: str) -> dict:
    return trace["requests"][rid]


def set_body(trace: dict, rid: str, body: bytes) -> None:
    trace["requests"][rid]["http"] = fx.http(rid, req(trace, rid)["http"]["status"], body)


def drop_trip(trace: dict, rid: str) -> None:
    req(trace, rid)["trips"].pop()


def extra_trip(trace: dict, rid: str) -> None:
    req(trace, rid)["trips"].append(copy.deepcopy(req(trace, rid)["trips"][0]))


def patch_trip(rid: str, index: int, **fields: Any) -> Callable[[dict], None]:
    return lambda trace: req(trace, rid)["trips"][index].update(fields)


def patch_state(**fields: Any) -> Callable[[dict], None]:
    return lambda trace: trace["state"].update(fields)


def patch_new_row(**fields: Any) -> Callable[[dict], None]:
    return lambda trace: trace["state"]["new_products"][0].update(fields)


def health_with_db_trip(trace: dict) -> None:
    req(trace, "01-health")["trips"].append(copy.deepcopy(req(trace, "03-list-defaults")["trips"][0]))


def etag_problem(trace: dict) -> None:
    req(trace, "01-health")["http"]["problems"] = ["etag não bate com o corpo"]


def header_change(trace: dict) -> None:
    http = req(trace, "02-product-1")["http"]
    http["headers"] = [[n, "text/html" if n == "content-type" else v] for n, v in http["headers"]]


def drop_request(trace: dict) -> None:
    trace["requests"].pop("03-list-defaults")


def http_controls() -> list[Control]:
    return [
        attempt("P1 identical traces are green", lambda t: None, True),
        attempt("N1 status differs", lambda t: t["requests"]["01-health"]["http"].update(status=503), False),
        attempt("N2 body differs by one byte", lambda t: set_body(t, "02-product-1", b'{"id":2}'), False),
        attempt("N3 a header value differs", header_change, False),
        attempt("N4 request missing from the target", drop_request, False),
        attempt("N24 a recorded response problem (ETag not matching the body) fails", etag_problem, False),
        attempt("N5 status of a POST differs", lambda t: t["requests"]["04-post"]["http"].update(status=200), False),
    ]


def wire_controls() -> list[Control]:
    select = req(fx.base_trace(), "02-product-1")["trips"][0]["sql"]
    return [
        attempt("P2 binary result format and Describe(statement) are accepted at sql level",
                patch_trip("02-product-1", 0, result_formats=[1], describe="statement", parse_oids=[705]), True,
                level="sql"),
        attempt("N6 an extra round trip", lambda t: extra_trip(t, "04-post"), False),
        attempt("N7 a missing round trip", lambda t: drop_trip(t, "03-list-defaults"), False),
        attempt("N8 simple query replaced by extended", patch_trip("03-list-defaults", 0, protocol="extended"), False),
        attempt("N9 statement text differs by one character", patch_trip("02-product-1", 0, sql=select + " "), False),
        attempt("N10 a text parameter differs", patch_trip("02-product-1", 0, params=["x"]), False),
        attempt("N11 COMMIT missing at the end of a POST", lambda t: drop_trip(t, "04-post"), False),
        attempt("N12 a DB round trip where none is expected", health_with_db_trip, False),
        attempt("N13 same SQL and parameters but different client bytes", patch_trip("02-product-1", 0, client_sha256=["0" * 64]), False),
        attempt("P3 different client bytes are ignored at sql level", patch_trip("02-product-1", 0, client_sha256=["0" * 64]), True,
                level="sql"),
        attempt("N14 backend error code differs", patch_trip("05-post-fk", 1, errors=["23505"]), False),
        attempt("N15 startup round trip changes", patch_trip("00-startup", 0, sql="SELECT 2"), False),
    ]


def state_controls() -> list[Control]:
    return [
        attempt("N16 dump hash differs", patch_state(dump_sha256="c" * 64), False),
        attempt("N17 seeded rows changed", patch_state(seed_products_md5="d" * 32), False),
        attempt("N18 an inserted row is missing", patch_state(new_products=[]), False),
        attempt("N19 an extra inserted row", lambda t: t["state"]["new_products"].append(dict(t["state"]["new_products"][0], id=10009)), False),
        attempt("N20 inserted row price differs", patch_new_row(price="12.51"), False),
        attempt("N21 inserted row name differs (U+FFFD vs ?)", patch_new_row(name="?"), False),
        attempt("N22 timestamps of the inserted row are incoherent", patch_new_row(timestamps_coherent=False), False),
        attempt("N23 row counts differ", lambda t: t["state"]["row_counts"].update(order_item=49999), False),
    ]


def consistency_controls() -> list[Control]:
    """A resposta 201 precisa casar com a linha gravada: o equivalente de CP1 e CN1 a CN3 do Gate 0."""
    def fails(mutate: Callable[[dict], None]) -> list[str]:
        return wire_compare.response_state_consistency(fx.mutated(fx.base_trace(), mutate))

    def body_of(trace: dict, **fields: Any) -> None:
        current = json.loads(req(trace, "04-post")["http"]["body"])
        set_body(trace, "04-post", json.dumps(dict(current, **fields), separators=(",", ":")).encode())

    inconsistent = fx.mutated(fx.base_trace(), lambda t: body_of(t, stock=7))
    return [
        ("CP1 response and stored row agree", fails(lambda t: None) == [], ""),
        ("CN1 the response id has no stored row", bool(fails(lambda t: body_of(t, id=99999))), ""),
        ("CN2 the response stock differs from the stored stock", bool(fails(lambda t: body_of(t, stock=7))), ""),
        ("CN3 the response price differs from the stored price", bool(fails(lambda t: body_of(t, price=12.51))), ""),
        ("CN4 the response category differs from the stored one", bool(fails(lambda t: body_of(t, categoryId=3))), ""),
        ("CN5 two equal traces that are both inconsistent still fail the state layer",
         not verdict(inconsistent, inconsistent)["green"], ""),
    ]


def lone_surrogate_pair() -> tuple[dict, dict]:
    base = fx.base_trace()
    actual = fx.mutated(base, lambda t: req(t, fx.LONE_SURROGATE_ID)["trips"][1].update(
        params=["TST-0009", "a�", "3", "1", "4"], client_sha256=["1" * 64]))
    return base, actual


def divergence_controls() -> list[Control]:
    base, actual = lone_surrogate_pair()
    bun = verdict(base, actual, engine="bun-1.2.22")
    node = verdict(base, actual, engine="node-26.11.1")
    altered = copy.deepcopy(actual)
    req(altered, fx.LONE_SURROGATE_ID)["trips"][1]["params"][1] = "a�c"
    other = verdict(base, altered, engine="bun-1.2.22")
    bad_id = [dict(KNOWN[0], normalizer="W-NOT-IN-REGISTRY")]
    try:
        normalizers.validate_known(bad_id)
        refused = False
    except normalizers.HarnessError:
        refused = True
    return [
        ("KP1 the documented wire divergence is NORMALIZED for bun", bun["green"] and
         bun["layers"]["wire"]["normalizers"] == ["W-BUN-LONE-SURROGATE-LEN"], ""),
        ("KN1 the same failure is not excused for another engine", not node["green"], ""),
        ("KN2 a slightly different failure is not excused", not other["green"], ""),
        ("KN3 an id outside the closed registry aborts the harness", refused, ""),
        ("KN4 the registry holds only ids the known file uses or documents", normalizers.REGISTRY >= {KNOWN[0]["normalizer"]}, ""),
    ]


def sweep_controls(trace: dict[str, Any]) -> list[Control]:
    """Cada requisição com idas e voltas do traço real, atacada uma a uma: nenhuma mutação pode passar."""
    ids = [rid for rid, e in trace["requests"].items() if e["trips"]]
    attacks = {
        "S1 drop the last trip": lambda t, rid: t["requests"][rid]["trips"].pop(),
        "S2 flip one client byte hash": lambda t, rid: t["requests"][rid]["trips"][0]["client_sha256"].__setitem__(0, "f" * 64),
        "S3 append a space to the first statement": lambda t, rid: t["requests"][rid]["trips"][0].update(
            sql=t["requests"][rid]["trips"][0]["sql"] + " "),
    }
    controls = []
    for name, attack in attacks.items():
        survivors = [rid for rid in ids if verdict(trace, fx.mutated(trace, lambda t, r=rid: attack(t, r)))["green"]]
        controls.append((f"{name} over {len(ids)} requests", not survivors, ",".join(survivors[:3])))
    http_ids = [rid for rid, e in trace["requests"].items() if e["http"]]
    flips = [rid for rid in http_ids if verdict(trace, fx.mutated(
        trace, lambda t, r=rid: t["requests"][r]["http"].update(status=599)))["green"]]
    controls.append((f"S4 change the status over {len(http_ids)} requests", not flips, ",".join(flips[:3])))
    return controls


def run_all(trace_path: str | None) -> list[Control]:
    controls = http_controls() + wire_controls() + state_controls() + consistency_controls() + divergence_controls()
    if trace_path:
        controls += sweep_controls(json.loads(Path(trace_path).read_text(encoding="utf-8")))
    return controls


def main() -> int:
    trace_path = sys.argv[sys.argv.index("--trace") + 1] if "--trace" in sys.argv else None
    controls = run_all(trace_path)
    bad = [c for c in controls if not c[1]]
    for name, ok, detail in controls:
        print(("PASS " if ok else "FAIL ") + name + (f"  [{detail}]" if detail and not ok else ""))
    print(f"{len(controls) - len(bad)}/{len(controls)} controls behaved as expected")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
