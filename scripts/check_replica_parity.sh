#!/bin/sh
# Confere que as rotas E1 a E6, as entidades e o seed da réplica são byte a byte os do Gate 0.
# 1) Sempre: os arquivos da réplica batem com gate0-parity.sha256 (versionado).
# 2) Se TYRUS_GATE0_DIR existir (ex.: ~/.local/share/tyrus-ceiling): cada arquivo é comparado com a origem (cmp).
set -eu
REPLICA_DIR="${REPLICA_DIR:-$(cd "$(dirname "$0")/.." && pwd)/oracle/apps/replica}"
MANIFEST="$REPLICA_DIR/gate0-parity.sha256"

[ -f "$MANIFEST" ] || { echo "check_replica_parity: manifesto ausente: $MANIFEST" >&2; exit 1; }
(cd "$REPLICA_DIR" && sha256sum --quiet -c gate0-parity.sha256) \
    || { echo "check_replica_parity: arquivo da réplica diverge do manifesto do Gate 0" >&2; exit 1; }

if [ -n "${TYRUS_GATE0_DIR:-}" ]; then
    while read -r _ path; do
        case "$path" in
            src/*) origin="$TYRUS_GATE0_DIR/nest-app/$path" ;;
            db/*) origin="$TYRUS_GATE0_DIR/$path" ;;
            *) echo "check_replica_parity: caminho fora do mapa: $path" >&2; exit 1 ;;
        esac
        cmp -s "$REPLICA_DIR/$path" "$origin" \
            || { echo "check_replica_parity: $path difere da origem $origin" >&2; exit 1; }
    done < "$MANIFEST"
fi
echo "check_replica_parity: ok ($(wc -l < "$MANIFEST") arquivos)"
