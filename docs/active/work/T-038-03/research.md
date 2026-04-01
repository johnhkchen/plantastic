# T-038-03 Research: db-scenario-recipe

## What exists

### Docker Compose (`docker-compose.yml`)
- PostgreSQL 17 + PostGIS 3.5 via `imresamu/postgis:17-3.5`
- Host port **5433** → container port 5432 (`"5433:5432"`)
- Credentials: `plantastic:plantastic@plantastic`
- Healthcheck: `pg_isready -U plantastic -d plantastic` (5s interval, 5 retries)
- Migrations auto-applied on first start via `scripts/docker-init-db.sh`
- Valkey on port 6379

### Justfile recipes (relevant)
- `dev-db` — `docker compose up -d --wait`, prints `localhost:5432` (WRONG — should be 5433)
- `dev-reset` — `docker compose down -v` + `up -d --wait`, also prints 5432 (WRONG)
- `test-integration` — defaults to `DATABASE_URL=postgres://...localhost:5432/plantastic` (WRONG)
- `scenarios` — `cargo run -p pt-scenarios 2>/dev/null`, no DATABASE_URL injection
- No `scenarios-db` or `scenarios-ci` recipe exists yet

### Port mismatch
Docker Compose maps `5433:5432`. Three justfile recipes (`dev-db`, `dev-reset`, `test-integration`) still reference port 5432. This was likely introduced when the port was changed (possibly by T-038-02) but the justfile wasn't updated. The ticket explicitly specifies port 5433.

### `.env.example`
- `DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic` — port 5432 (stale)
- `TEST_DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic_test` — port 5432

### Scenario harness (`tests/scenarios/`)
- `infrastructure.rs` checks `std::env::var("DATABASE_URL")` — returns `Blocked` if missing
- `scenarios` recipe runs without DATABASE_URL, so infra scenarios always show BLOCKED
- This is the expected behavior per AC: "just scenarios (without DB) still works and shows BLOCKED"

### How `docker compose up -d` works
- Idempotent: if containers are already running, it's a no-op
- `--wait` flag: waits for healthcheck to pass before returning
- `docker compose up -d db` starts only the `db` service (not valkey) — ticket says to use this

### Dependencies
- T-038-01 (fix-pdf-assertion): done
- T-038-02 (fix-tenant-isolation): done

## Constraints
- Recipe must not fail if Docker isn't available — print helpful message
- Must not regress `just scenarios` (no-DB mode)
- Consider `scenarios-ci` for Neon branching (implementation note, not AC)

## Key files to modify
1. `justfile` — add `scenarios-db`, fix port 5433 in existing recipes
2. `.env.example` — update DATABASE_URL to port 5433
3. `CLAUDE.md` — document `just scenarios-db`

## Assumptions
- `docker compose` (v2 syntax) is available on dev machines — consistent with existing `dev-db`
- The `--wait` flag handles the health check wait, eliminating the need for a manual `pg_isready` loop
- However, `docker compose up -d db --wait` may not work with service-specific `--wait` on all versions; fallback to a manual health loop may be needed
