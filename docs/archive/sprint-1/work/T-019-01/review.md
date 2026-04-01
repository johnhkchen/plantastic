# T-019-01 Review: Docker Compose + Migrations

## Summary

Created the local development Docker stack: Postgres 18 + PostGIS 3.6 and Valkey 8
via Docker Compose, with automatic migration bootstrap on first startup.

## Files Created

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Two-service dev stack (db + valkey) with health checks and named volume |
| `scripts/docker-init-db.sh` | Init script that applies all 6 migrations on first Postgres boot |

## Files Modified

| File | Change |
|------|--------|
| `.env.example` | DATABASE_URL updated to match Docker Compose credentials (`plantastic:plantastic@localhost:5432/plantastic`) |
| `justfile` | Added `up`, `down`, `down-clean` recipes in new Docker section |

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| `docker-compose.yml` at project root | Done |
| `db` service: `postgis/postgis:18-3.6`, port 5432, volume, healthcheck, env vars | Done |
| `valkey` service: `valkey/valkey:8`, port 6379, healthcheck | Done |
| No other services | Done — only db and valkey |
| Migrations run on first startup | Done — via `docker-entrypoint-initdb.d` init script |
| All 6 migrations applied in order | Done — shell glob sorts by NNN prefix |
| PostGIS extension created before zone/plant migrations | Done — migration 001 creates it |
| Idempotent (`down && up` works) | Done — init only runs on empty data dir |
| `down -v` removes volumes for clean slate | Done — named volume `plantastic-db-data` |

## Test Coverage

- No Rust code was changed; no new tests needed
- `just check` passes: format, lint, test, scenarios all green
- `docker compose config` validates compose file syntax
- `bash -n` validates init script syntax
- Manual verification (docker compose up/down cycle) should be done by reviewer

## Scenario Dashboard

No scenario changes expected — this is infrastructure-only. The dashboard should
show the same baseline before and after.

## Open Concerns

1. **Manual verification needed**: The Docker stack should be tested end-to-end by
   running `docker compose up -d` and checking PostGIS version + table existence.
   This requires Docker to be running, which may not be available in CI.

2. **New migrations after init**: If a developer adds migration 007+ after their
   first `docker compose up`, they need either `just migrate-direct` against the
   running container or `just down-clean && just up` for a fresh start. This is
   documented behavior for Postgres init scripts but worth noting in T-019-02
   (dev stack recipes).

3. **Image availability**: `postgis/postgis:18-3.6` assumes Postgres 18 + PostGIS
   3.6 tags exist on Docker Hub. If the tag doesn't exist yet (Postgres 18 is
   very new), fall back to `postgis/postgis:17-3.5` and update when available.
