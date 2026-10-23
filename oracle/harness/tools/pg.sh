#!/bin/sh
# Contêiner do Postgres do harness de fio: reaproveita db/up.sh, db/reset.sh e db/down.sh da réplica
# (imagem por digest, tmpfs, seed determinístico) com outro nome e outra porta.
# Uso: pg.sh up | reset | down. Variáveis: WIRE_PG_CONTAINER (prefixo tyrus-), WIRE_PG_PORT.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
REPLICA_DB="$HERE/../../apps/replica/db"
WIRE_PG_CONTAINER="${WIRE_PG_CONTAINER:-tyrus-wire-pg}"
WIRE_PG_PORT="${WIRE_PG_PORT:-55521}"
case "$WIRE_PG_CONTAINER" in
    tyrus-*) ;;
    *) echo "pg.sh: o contêiner deve começar com tyrus- (recebido $WIRE_PG_CONTAINER)" >&2; exit 2 ;;
esac
case "$WIRE_PG_PORT" in
    3000|3001|3002) echo "pg.sh: portas 3000 a 3002 são do dono" >&2; exit 2 ;;
esac
export REPLICA_PG_CONTAINER="$WIRE_PG_CONTAINER" REPLICA_PG_PORT="$WIRE_PG_PORT"
case "${1:-}" in
    up) exec "$REPLICA_DB/up.sh" ;;
    reset) exec "$REPLICA_DB/reset.sh" ;;
    down) exec "$REPLICA_DB/down.sh" ;;
    *) echo "uso: pg.sh up|reset|down" >&2; exit 2 ;;
esac
