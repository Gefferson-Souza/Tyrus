#!/bin/sh
# Hash do conteúdo semeado (as três tabelas, ordenadas por id); duas montagens do seed devem coincidir.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=replica.env
. "$HERE/replica.env"
DB="${1:-replica_seed}"
for table in category product order_item; do
    docker exec -i "$REPLICA_PG_CONTAINER" psql -U postgres -d "$DB" -At -X \
        -c "COPY (SELECT * FROM ${table} ORDER BY id) TO STDOUT"
done | sha256sum | cut -d' ' -f1
