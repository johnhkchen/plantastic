#!/bin/bash
# Apply Plantastic migrations on first Postgres container start.
# Mounted into /docker-entrypoint-initdb.d/ by docker-compose.yml.
set -e

for f in /migrations/*.sql; do
    case "$f" in *.down.sql) continue ;; esac
    echo "Applying: $f"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$f"
done

echo "All migrations applied."
