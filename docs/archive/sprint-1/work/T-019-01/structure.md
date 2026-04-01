# T-019-01 Structure: Docker Compose + Migrations

## Files Created

### `docker-compose.yml` (project root)

Top-level Docker Compose file. Two services, one named volume.

```yaml
services:
  db:
    image: postgis/postgis:18-3.6
    ports: ["5432:5432"]
    environment:
      POSTGRES_DB: plantastic
      POSTGRES_USER: plantastic
      POSTGRES_PASSWORD: plantastic
    volumes:
      - plantastic-db-data:/var/lib/postgresql/data
      - ./migrations:/migrations:ro
      - ./scripts/docker-init-db.sh:/docker-entrypoint-initdb.d/01-migrations.sh:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U plantastic -d plantastic"]
      interval: 5s
      timeout: 5s
      retries: 5

  valkey:
    image: valkey/valkey:8
    ports: ["6379:6379"]
    healthcheck:
      test: ["CMD", "valkey-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  plantastic-db-data:
```

### `scripts/docker-init-db.sh` (new)

Shell script mounted into Postgres container's initdb.d directory.

```bash
#!/bin/bash
set -e
for f in /migrations/*.sql; do
    case "$f" in *.down.sql) continue ;; esac
    echo "Applying: $f"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$f"
done
echo "All migrations applied."
```

Key details:
- Skips `.down.sql` files
- Uses `ON_ERROR_STOP=1` so a bad migration fails loudly
- Uses the container's built-in `POSTGRES_USER` and `POSTGRES_DB` env vars
- Glob sorts alphabetically, matching the NNN- prefix ordering

## Files Modified

### `.env.example`

Update DATABASE_URL default to match Docker Compose credentials:
```
DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic
```

### `justfile`

Add three recipes under a new `# ─── Docker ──────` section:
- `up` — `docker compose up -d`
- `down` — `docker compose down`
- `down-clean` — `docker compose down -v`

### `.gitignore`

Verify Docker-related entries exist (`.env` should already be ignored).
No changes expected.

## Files NOT Created

- No `Dockerfile` — we use stock images only
- No custom Valkey config — defaults are fine for dev
- No `.dockerignore` — we're not building images
- No `docker-compose.override.yml` — keep it simple

## Module Boundaries

This ticket creates infrastructure only. No Rust code changes. No crate
modifications. The init script is a thin shell wrapper around the existing
migration SQL files.

## Dependency Ordering

1. `scripts/docker-init-db.sh` must exist before `docker compose up` works
2. `docker-compose.yml` references `./migrations` and `./scripts/docker-init-db.sh`
3. `.env.example` and `justfile` changes are independent
