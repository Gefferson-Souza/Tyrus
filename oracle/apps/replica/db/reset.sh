#!/bin/sh
# Recria o banco replica a partir do template replica_seed (derruba conexões abertas).
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=replica.env
. "$HERE/replica.env"
docker exec -i "$REPLICA_PG_CONTAINER" psql -U postgres -v ON_ERROR_STOP=1 -q -X \
    -c "DROP DATABASE IF EXISTS replica WITH (FORCE)" \
    -c "CREATE DATABASE replica TEMPLATE replica_seed"
