# T-038-03 Review: db-scenario-recipe

## Summary

Added `just scenarios-db` recipe that starts the Docker Compose Postgres, waits for health, and runs the scenario dashboard with DATABASE_URL injected. Also fixed stale port 5432 references across justfile and .env.example (should be 5433 to match docker-compose.yml mapping).

## Files modified

| File | Change |
|------|--------|
| `justfile` | Added `scenarios-db` recipe; fixed port 5432→5433 in `dev-db`, `dev-reset`, `test-integration` |
| `.env.example` | Updated DATABASE_URL and TEST_DATABASE_URL from port 5432→5433 |
| `CLAUDE.md` | Added `just scenarios-db` to Key Commands |

## Acceptance criteria verification

| Criterion | Status |
|-----------|--------|
| `just scenarios-db` starts Docker Compose DB if not running | ✓ `docker compose up -d db` (idempotent) |
| Waits for health check | ✓ `pg_isready` loop, 30 attempts, 1s sleep |
| Runs scenarios with DATABASE_URL on port 5433 | ✓ Injects `DATABASE_URL=postgres://plantastic:plantastic@localhost:5433/plantastic` |
| Prints summary | ✓ Scenario dashboard output printed |
| `just scenarios` (without DB) still works, shows BLOCKED | ✓ Verified — 87.5/240.0 min, infra scenarios BLOCKED |
| Update .env.example with port 5433 | ✓ Both DATABASE_URL and TEST_DATABASE_URL updated |
| Document in CLAUDE.md | ✓ Added to Key Commands block |

## Scenario dashboard

**Before:** 87.5 / 240.0 min (36.5%) — 10 pass, 0 fail, 4 not implemented, 3 blocked
**After:** 87.5 / 240.0 min (36.5%) — identical (this ticket is tooling, not capability)

No regression.

## Test coverage

This ticket modifies only developer tooling files (justfile, .env.example, CLAUDE.md). No Rust code was changed, so no unit or integration tests are applicable. Verification is functional:
- `just scenarios` runs without regression ✓
- `just --list` shows `scenarios-db` with correct description ✓
- No stale port 5432 references remain in justfile or .env.example ✓

Full verification of `just scenarios-db` end-to-end requires Docker and is a manual test.

## Port fix (bonus)

The docker-compose.yml maps `"5433:5432"` (host:container), but three justfile recipes and .env.example referenced port 5432. This was a latent bug — `just dev-db` would print the wrong connection string, and `just test-integration` would default to the wrong port. Fixed as part of this ticket since the AC explicitly mentions port 5433.

## Open concerns

1. **`scenarios-ci` not implemented** — the ticket's implementation notes mention considering a `just scenarios-ci` recipe for Neon branching. This is not in the AC and was deferred. Can be a follow-up ticket if needed.

2. **Docker Compose v2 assumption** — the recipe uses `docker compose` (v2 CLI plugin syntax), consistent with all existing recipes in the justfile. Docker Compose v1 (`docker-compose`) is not supported.

3. **Manual end-to-end verification pending** — `just scenarios-db` was not tested with a live Docker daemon in this session. The recipe logic mirrors the existing `dev-db` pattern and the health check loop is straightforward, but human verification with Docker running is recommended.
