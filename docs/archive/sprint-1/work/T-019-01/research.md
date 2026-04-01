# T-019-01 Research: Docker Compose + Migrations

## Current State

### No Docker Infrastructure Exists

The project has zero Docker files. Developers currently need a manually-provisioned
Postgres+PostGIS instance (local install, Neon branch, or Railway). This ticket
creates the canonical local dev stack.

### Migration Files

Six ordered SQL migrations in `migrations/`:

| File | Purpose | PostGIS? |
|------|---------|----------|
| 001-create-tenants.sql | `CREATE EXTENSION IF NOT EXISTS postgis;` + tenants table | Yes — creates extension |
| 002-create-projects.sql | projects table with `GEOGRAPHY(POINT, 4326)` column | Uses PostGIS types |
| 003-create-zones.sql | zones table with `GEOMETRY(POLYGON, 4326)` + GIST index | Uses PostGIS types |
| 004-create-materials.sql | materials catalog, no spatial types | No |
| 005-create-tier-assignments.sql | tier assignments (good/better/best) | No |
| 006-create-plants.sql | shared plant database | No |

Each has a corresponding `.down.sql` for rollback. The naming convention is
`NNN-description.sql` — these are plain SQL, not sqlx timestamped migrations.

### Migration Execution Methods

Three existing paths to run migrations:

1. **`just migrate`** — Doppler injects `DATABASE_URL`, calls `scripts/migrate.sh`
   which uses `sqlx migrate run --source migrations`
2. **`just migrate-direct`** — same script, expects `DATABASE_URL` in env
3. **`just db-migrate`** — loops over `migrations/*.sql` files via `psql` directly

For Docker bootstrap, method 3's approach (plain psql) is simplest because it
doesn't require sqlx-cli inside the container. Postgres's `/docker-entrypoint-initdb.d/`
mechanism runs `.sql` and `.sh` files on first init only — perfect for idempotent bootstrap.

### Connection Configuration

- `.env.example` defaults to `postgres://localhost:5432/plantastic_dev`
- `plantastic-api/src/main.rs` reads `DATABASE_URL`, falls back to that same default
- Pool settings in `pt-repo/src/pool.rs`: max 5 connections, 30s idle timeout

The docker-compose database URL needs to match. The `.env.example` default uses
`plantastic_dev` as the database name but the ticket AC specifies `plantastic`.
We'll use `plantastic` per the AC and update `.env.example` to match.

### Valkey (Redis Alternative)

The ticket requires Valkey 8. No existing code references Redis/Valkey yet — this
is forward-looking infrastructure. The service just needs to be available on 6379.

### Image Availability

- `postgis/postgis:18-3.6` — Docker Hub. PostGIS 3.6 on Postgres 18 (currently
  the latest tag). PostGIS extension is pre-installed, just needs `CREATE EXTENSION`.
- `valkey/valkey:8` — Docker Hub. Drop-in Redis replacement.

Both images are multi-arch (amd64 + arm64), good for M-series Mac devs.

### Constraints

- Postgres init scripts in `/docker-entrypoint-initdb.d/` run only on first container
  start (empty data directory). `docker compose down -v` removes volumes, triggering
  re-init on next `up`.
- The `CREATE EXTENSION IF NOT EXISTS postgis;` in migration 001 is idempotent.
- Migrations are plain SQL — no Rust tooling needed inside the container.
- The `just db-migrate` recipe already handles running `.sql` files in order via
  shell glob, filtering out `.down.sql` files.

### What Does NOT Exist

- No `Dockerfile` (none needed — we use stock images)
- No `docker-compose.yml`
- No init scripts for container bootstrap
- No Valkey configuration or usage in application code
