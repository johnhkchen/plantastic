# T-019-01 Design: Docker Compose + Migrations

## Decision: Init Script via docker-entrypoint-initdb.d

### Options Considered

**Option A: Mount migrations + shell init script into `/docker-entrypoint-initdb.d/`**

Postgres runs scripts in this directory on first boot (empty data dir). We mount
the existing `migrations/` directory and provide a small shell script that applies
them in order via `psql`.

Pros:
- Zero additional tooling (no sqlx-cli in container)
- Uses Postgres's native init mechanism
- Migrations are the single source of truth — no duplication
- Idempotent by design (init only runs on empty data dir)

Cons:
- Init script is a new file to maintain (trivial — ~10 lines)

**Option B: Custom Dockerfile with sqlx-cli baked in**

Build a custom image that includes sqlx-cli and runs `sqlx migrate run` on start.

Pros:
- Uses the same migration runner as production

Cons:
- Requires building a custom image (slower startup, more to maintain)
- sqlx-cli is a Rust tool — installing it in the container adds ~5 min build time
- Violates the "no other services" principle — keeps things simple

**Option C: Separate migration service in docker-compose**

Add a short-lived `migrate` service that depends on `db` being healthy, runs
migrations, then exits.

Pros:
- Clean separation of concerns
- Runs every `docker compose up`, not just first boot

Cons:
- Adds a third service (ticket says only db + valkey)
- Extra complexity for a dev-only setup
- Migrations already idempotent, so running on every boot is unnecessary overhead

### Decision: Option A

The init script approach is simplest, requires no custom images, and leverages
Postgres's built-in mechanism. The `docker-entrypoint-initdb.d/` approach is the
standard pattern for bootstrapping Postgres containers.

The only nuance: init scripts run only on first boot. If a developer adds a new
migration after initial setup, they need `docker compose down -v && docker compose up -d`
or `just migrate-direct` against the running container. This is acceptable for a
dev stack — the `just migrate-direct` path already exists.

## Database Name

The ticket AC says `POSTGRES_DB=plantastic`. The `.env.example` currently defaults
to `plantastic_dev`. We'll update `.env.example` to use credentials that match the
Docker Compose setup: `postgres://plantastic:plantastic@localhost:5432/plantastic`.

## Init Script Design

A single shell script `scripts/docker-init-db.sh` mounted at
`/docker-entrypoint-initdb.d/01-migrations.sh`. It:

1. Loops over `migrations/*.sql`, excluding `.down.sql`
2. Applies each via `psql` in sorted order
3. Uses the `POSTGRES_DB` / `POSTGRES_USER` env vars already set by the container

The migrations directory is bind-mounted read-only into the container so the
script can access the SQL files.

## Health Checks

- **db**: `pg_isready -U plantastic -d plantastic` with 5s interval, 5s timeout, 5 retries
- **valkey**: `valkey-cli ping` with same intervals

## Volume Strategy

Named volume `plantastic-db-data` for Postgres data persistence. Valkey is
ephemeral (cache only, no persistence needed for dev).

## .env.example Update

Update the DATABASE_URL default to match the Docker Compose credentials:
```
DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic
```

## Justfile Integration

Add convenience recipes:
- `just up` — `docker compose up -d` (start dev stack)
- `just down` — `docker compose down` (stop dev stack)
- `just down-clean` — `docker compose down -v` (remove volumes for clean slate)
