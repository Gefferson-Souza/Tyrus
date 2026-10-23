"""Estado final do banco da réplica: `pg_dump --data-only` com hash e as linhas que a execução inseriu.

As linhas de `product` não entram no dump porque `created_at` e `updated_at` das linhas novas vêm de `now()`:
as sementes (id <= SEED_MAX_ID) entram por hash e as novas por valor, com os instantes trocados por uma prova
(`created_at = updated_at` e recente).
"""
from __future__ import annotations

import hashlib
import json
import subprocess
from typing import Any

SEED_MAX_ID = 10000
DATABASE = "replica"
DOCKER_TIMEOUT_S = 120

SEED_SQL = f"SELECT md5(string_agg(p::text, E'\\n' ORDER BY id)) FROM product p WHERE id <= {SEED_MAX_ID}"
NEW_ROWS_SQL = f"""SELECT coalesce(json_agg(json_build_object(
  'id', id, 'sku', sku, 'name', name, 'description', description, 'price', price::text, 'stock', stock,
  'active', active, 'category_id', category_id,
  'timestamps_coherent', created_at = updated_at AND created_at > now() - interval '1 hour') ORDER BY id), '[]'::json)
  FROM product WHERE id > {SEED_MAX_ID}"""
COUNTS_SQL = """SELECT json_object_agg(table_name,
  (xpath('/row/c/text()', query_to_xml(format('select count(*) as c from %I', table_name), false, true, '')))[1]::text::int)
  FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"""


def _docker(container: str, command: list[str]) -> bytes:
    done = subprocess.run(["docker", "exec", container, *command], capture_output=True, check=False,
                          timeout=DOCKER_TIMEOUT_S)
    if done.returncode != 0:
        raise RuntimeError(f"docker exec {command[0]} falhou: {done.stderr.decode('utf-8', 'replace')[:300]}")
    return done.stdout


def _psql(container: str, sql: str) -> str:
    return _docker(container, ["psql", "-U", "postgres", "-d", DATABASE, "-At", "-X", "-c", sql]).decode().strip()


def execute(container: str, sql: str) -> None:
    """Escrita direta no banco (controle negativo do estado: uma mudança que nenhuma requisição causou)."""
    _psql(container, sql)


def stable_dump(dump: bytes) -> bytes:
    """O pg_dump 17.11 imprime `\\restrict <token aleatório>` e `\\unrestrict`; fora isso o dump é determinístico."""
    return b"\n".join(line for line in dump.split(b"\n") if not line.startswith((b"\\restrict ", b"\\unrestrict ")))


def collect_state(container: str) -> dict[str, Any]:
    dump = _docker(container, ["pg_dump", "-U", "postgres", "--data-only", "--column-inserts",
                               "--exclude-table-data=product", DATABASE])
    dump = stable_dump(dump)
    return {
        "dump_sha256": hashlib.sha256(dump).hexdigest(),
        "dump_bytes": len(dump),
        "seed_products_md5": _psql(container, SEED_SQL),
        "new_products": json.loads(_psql(container, NEW_ROWS_SQL)),
        "row_counts": json.loads(_psql(container, COUNTS_SQL)),
    }
