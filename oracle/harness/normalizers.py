"""Registro fechado de normalizadores do harness de fio e carregador das divergências conhecidas.

Uma divergência só é perdoada por id deste registro e por texto exato de falha; id fora dele aborta o harness.
Ampliar o registro é um evento com gate (D14), não uma edição de teste.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

REGISTRY: frozenset[str] = frozenset({
    "W-BUN-LONE-SURROGATE-LEN",
})
FAILURE_LAYERS = ("http", "wire", "state")


class HarnessError(Exception):
    """Configuração inválida do harness (nunca um veredito de equivalência)."""


def validate_known(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    for entry in entries:
        normalizer = entry.get("normalizer")
        if normalizer not in REGISTRY:
            raise HarnessError(f"normalizador `{normalizer}` fora do registro fechado")
        if not entry.get("engine"):
            raise HarnessError(f"divergência `{normalizer}` sem `engine`")
        unknown = set(entry.get("failures", {})) - set(FAILURE_LAYERS)
        if unknown:
            raise HarnessError(f"camadas desconhecidas em `{normalizer}`: {sorted(unknown)}")
    return entries


def load_known(path: Path) -> list[dict[str, Any]]:
    return validate_known(json.loads(path.read_text(encoding="utf-8")))


def excused(known: list[dict[str, Any]], engine: str, layer: str) -> dict[str, str]:
    """Texto exato de falha -> id do normalizador que o perdoa, só para o motor e a camada pedidos."""
    return {text: entry["normalizer"] for entry in known if entry["engine"] == engine
            for text in entry.get("failures", {}).get(layer, [])}
