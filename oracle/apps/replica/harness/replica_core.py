"""Lógica pura da execução de requisições da réplica: ETag, normalização e comparação."""
from __future__ import annotations

import base64
import hashlib
import re
from typing import Any

IGNORED_HEADERS = frozenset({"date", "connection", "keep-alive"})
TIMESTAMP_FIELD = re.compile(
    rb'"(createdAt|updatedAt)":"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z"'
)
VOLATILE_MARK = "<volatile>"
LEVELS = ("strict", "body")


def weak_etag(body: bytes) -> str:
    """ETag fraco do Express: comprimento em hexadecimal e 27 caracteres do SHA-1 em base64."""
    digest = base64.b64encode(hashlib.sha1(body).digest()).decode("ascii")[:27]
    return f'W/"{len(body):x}-{digest}"'


def mask_timestamps(body: bytes) -> bytes:
    return TIMESTAMP_FIELD.sub(rb'"\1":"<TS>"', body)


def _in_range(value: str, bounds: list[int]) -> bool:
    return value.isdigit() and bounds[0] <= int(value) <= bounds[1]


def build_record(request: dict[str, Any], status: int, headers: list[tuple[str, str]], body: bytes) -> dict[str, Any]:
    """Resposta observada, já marcada com as verificações que só valem com o corpo bruto."""
    visible = [(name.lower(), value) for name, value in headers if name.lower() not in IGNORED_HEADERS]
    etag = next((value for name, value in visible if name == "etag"), None)
    masked = bool(request.get("mask_timestamps")) and 200 <= status < 300
    problems: list[str] = []
    if status != request["expect_status"]:
        problems.append(f"status {status} diferente do esperado {request['expect_status']}")
    if etag is not None and etag != weak_etag(body):
        problems.append("etag não bate com o corpo")
    for name, bounds in request.get("volatile_headers", {}).items():
        value = next((v for n, v in visible if n == name), None)
        if value is None or not _in_range(value, bounds):
            problems.append(f"cabeçalho volátil {name} ausente ou fora de {bounds}")
    stable_body = mask_timestamps(body) if masked else body
    stable_headers = [
        [name, VOLATILE_MARK if name in request.get("volatile_headers", {}) else value]
        for name, value in visible
        if not (masked and name == "etag")
    ]
    return {
        "id": request["id"],
        "status": status,
        "headers": stable_headers,
        "body": stable_body.decode("utf-8", errors="backslashreplace"),
        "body_sha256": hashlib.sha256(stable_body).hexdigest(),
        "problems": problems,
    }


def diff_runs(left: list[dict[str, Any]], right: list[dict[str, Any]], level: str) -> list[str]:
    """Lista de diferenças entre duas execuções; vazia significa idênticas no nível pedido."""
    if level not in LEVELS:
        raise ValueError(f"nível desconhecido: {level}")
    if [r["id"] for r in left] != [r["id"] for r in right]:
        return ["conjuntos de requisições diferentes"]
    diffs: list[str] = []
    for a, b in zip(left, right):
        if a["status"] != b["status"]:
            diffs.append(f"{a['id']}: status {a['status']} contra {b['status']}")
        if a["body_sha256"] != b["body_sha256"]:
            diffs.append(f"{a['id']}: corpo diferente")
        if level == "strict" and a["headers"] != b["headers"]:
            diffs.append(f"{a['id']}: cabeçalhos diferentes")
    return diffs
