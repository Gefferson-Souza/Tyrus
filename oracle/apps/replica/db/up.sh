#!/bin/sh
# Sobe o Postgres 17 da réplica (imagem fixada por digest, dados em tmpfs) e monta o template replica_seed.
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=replica.env
. "$HERE/replica.env"

free_kb=$(df -Pk "$HERE" | awk 'NR==2 {print $4}')
if [ "$free_kb" -lt 3145728 ]; then
    echo "up.sh: menos de 3 GB livres em disco, abortando" >&2
    exit 1
fi

psql_in() { docker exec -i "$REPLICA_PG_CONTAINER" psql -U postgres -v ON_ERROR_STOP=1 -q -X "$@"; }

if ! docker inspect "$REPLICA_PG_CONTAINER" >/dev/null 2>&1; then
    docker run -d --name "$REPLICA_PG_CONTAINER" \
        --tmpfs /var/lib/postgresql/data:rw,size=1024m \
        -e POSTGRES_PASSWORD="$REPLICA_PG_PASSWORD" -e POSTGRES_DB=postgres \
        -p "127.0.0.1:${REPLICA_PG_PORT}:5432" \
        "$REPLICA_PG_IMAGE" \
        -c fsync=off -c synchronous_commit=off -c full_page_writes=off \
        -c max_wal_size=256MB -c shared_buffers=128MB -c max_connections=100 -c timezone=UTC >/dev/null
fi

ready=0
for _ in $(seq 1 120); do
    if docker exec "$REPLICA_PG_CONTAINER" pg_isready -h 127.0.0.1 -U postgres >/dev/null 2>&1; then
        ready=1
        break
    fi
    sleep 0.5
done
[ "$ready" = "1" ] || { echo "up.sh: Postgres não ficou pronto" >&2; exit 1; }

have_seed=$(psql_in -At -c "SELECT count(*) FROM pg_database WHERE datname='replica_seed' AND datistemplate")
if [ "$have_seed" != "1" ] || [ "${FORCE_RESEED:-0}" = "1" ]; then
    psql_in -c "ALTER DATABASE replica_seed IS_TEMPLATE false" 2>/dev/null || true
    psql_in -c "DROP DATABASE IF EXISTS replica_seed WITH (FORCE)"
    psql_in -c "CREATE DATABASE replica_seed"
    psql_in -d replica_seed < "$HERE/schema.sql"
    psql_in -d replica_seed < "$HERE/seed.sql" >/dev/null
    psql_in -c "ALTER DATABASE replica_seed IS_TEMPLATE true"
fi
"$HERE/reset.sh"
echo "up.sh: pronto em 127.0.0.1:${REPLICA_PG_PORT} (banco replica)"
