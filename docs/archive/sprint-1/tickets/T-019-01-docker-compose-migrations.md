---
id: T-019-01
story: S-019
title: docker-compose-migrations
type: task
status: open
priority: high
phase: done
depends_on: []
---

## Context

Create a Docker Compose file with Postgres 18 + PostGIS 3.6 and Valkey. Migrations should run automatically on first startup so `docker compose up` gives you a ready-to-use database.

## Acceptance Criteria

### Docker Compose
- `docker-compose.yml` at project root
- Service: `db` — `postgis/postgis:18-3.6` image
  - Port 5432 exposed
  - Volume for data persistence across restarts
  - Health check on `pg_isready`
  - Environment: POSTGRES_DB=plantastic, POSTGRES_USER=plantastic, POSTGRES_PASSWORD=plantastic
- Service: `valkey` — `valkey/valkey:8` image
  - Port 6379 exposed
  - Health check on `valkey-cli ping`
- No other services — API, frontend, worker run natively

### Migration bootstrap
- Migrations run on first startup (init script or entrypoint)
- All 6 migration files applied in order
- PostGIS extension created before zone/plant migrations
- Idempotent — `docker compose down && docker compose up` works without manual cleanup
- Verify: `docker compose up -d && psql $TEST_DATABASE_URL -c "SELECT PostGIS_Version();"` returns 3.6.x

### Developer ergonomics
- `docker compose up -d` starts services in background
- `docker compose down` stops everything
- `docker compose down -v` removes volumes (clean slate)
- Startup time < 10 seconds on warm pull
