# T-019-02 Research — Dev Stack Recipes

## What Exists

### Docker Compose (`docker-compose.yml`)
T-019-01 delivered a working Compose file with two services:
- **db**: `postgis/postgis:18-3.6`, port 5432, health check via `pg_isready`, volume `plantastic-db-data`, migrations auto-applied via `scripts/docker-init-db.sh` mounted into `/docker-entrypoint-initdb.d/`.
- **valkey**: `valkey/valkey:8`, port 6379, health check via `valkey-cli ping`.

No comment block at top of docker-compose.yml — ticket requires one.

### Justfile (project root)
Already has Docker-adjacent recipes:
- `up` — `docker compose up -d`
- `down` — `docker compose down`
- `down-clean` — `docker compose down -v`

Also has development recipes:
- `dev-api` — `cargo run -p plantastic-api`
- `dev-web` — `cd web && npm run dev`
- `dev` — prints reminder to run both in separate terminals

Missing from ticket requirements:
- `just dev-db` — start only DB, wait for health, print connection string
- `just dev-stack` — full local stack (compose + API + worker + frontend)
- `just dev-down` — stop compose services (overlap with existing `down`)
- `just dev-reset` — `docker compose down -v && docker compose up -d`

### .env.example (project root)
Exists with:
- `DATABASE_URL` (local Postgres connection string) ✓
- `PORT=3000`
- `S3_BUCKET`, `AWS_REGION`, placeholder AWS creds
- `VITE_MOCK_API`, `API_URL` for frontend

Missing from ticket requirements:
- `TEST_DATABASE_URL` — not present
- `VALKEY_URL` — not present
- `RUST_LOG` — not present
- `S3_ENDPOINT` — not present (has `S3_BUCKET` but not endpoint)

### .gitignore
Already ignores `.env` and `.env.*` while preserving `.env.example`. ✓

### .doppler.yaml
Configured with project `plantastic`, config `dev`. The `migrate` recipe uses `doppler run --` to inject `DATABASE_URL`. This is an alternative to `.env` — both paths should work.

### Docker Compose env integration
Currently, Compose hardcodes `POSTGRES_DB=plantastic`, `POSTGRES_USER=plantastic`, `POSTGRES_PASSWORD=plantastic` inline. The ticket says "Docker Compose reads from `.env` for any overrides" — this means the Compose file should use variable substitution with defaults so `.env` can override without breaking the default experience.

## Key Patterns

### Recipe style
Justfile uses `#!/usr/bin/env bash` for multi-line recipes with `set -euo pipefail`. Single-line recipes use `@` prefix for quiet output. Comment above each recipe serves as `--list` description.

### Waiting for health
Docker Compose health checks exist but `docker compose up -d` returns immediately. The `dev-db` recipe needs to poll until healthy. Options:
1. `docker compose up -d --wait` (Compose v2.1+, blocks until healthy)
2. Manual loop polling `docker inspect --format='{{.State.Health.Status}}'`

`--wait` is the modern, clean approach and works with Docker Compose v2 which is standard.

### Worker service
The `worker/` directory exists but no `just dev-worker` recipe. The ticket mentions worker in `dev-stack`. Need to check what's there.

## Constraints
- Existing `up`/`down`/`down-clean` recipes overlap with requested `dev-down`/`dev-reset`. Need to decide: replace, alias, or coexist.
- The `dev` recipe already exists as a print-instructions helper. `dev-stack` is the more complete version of this concept.
- No `dev-worker` recipe exists yet — `dev-stack` instructions need to account for this.
