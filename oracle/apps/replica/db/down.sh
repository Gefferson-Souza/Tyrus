#!/bin/sh
# Remove o contêiner da réplica (os dados vivem em tmpfs e somem junto).
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=replica.env
. "$HERE/replica.env"
docker rm -f "$REPLICA_PG_CONTAINER" >/dev/null 2>&1 || true
