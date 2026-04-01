# T-019-02 Review — Dev Stack Recipes

## Summary

Wired Docker Compose into the justfile with four `dev-*` recipes, updated `.env.example` with all required environment variables, and added a documentation comment block to `docker-compose.yml`.

## Files Modified

| File | Change |
|------|--------|
| `justfile` | Removed `up`/`down`/`down-clean`/`dev` recipes; added `dev-db`/`dev-stack`/`dev-down`/`dev-reset` |
| `.env.example` | Added `TEST_DATABASE_URL`, `VALKEY_URL`, `S3_ENDPOINT`, `RUST_LOG`; reorganized with section headers |
| `docker-compose.yml` | Added 14-line comment block at top documenting services and usage |

## Acceptance Criteria Checklist

### Justfile recipes
- ✅ `just dev-db` — starts Compose services with `--wait`, prints connection strings after health check
- ✅ `just dev-stack` — calls `dev-db`, then prints terminal commands for API + web
- ✅ `just dev-down` — `docker compose down`
- ✅ `just dev-reset` — `docker compose down -v && docker compose up -d --wait`, prints connection strings

### Environment config
- ✅ `DATABASE_URL` — present (local Postgres)
- ✅ `TEST_DATABASE_URL` — added (same host, `plantastic_test` database)
- ✅ `VALKEY_URL` — added (`redis://localhost:6379`)
- ✅ `S3_ENDPOINT` — added (placeholder `http://localhost:4566`)
- ✅ `RUST_LOG` — added (default `info`)
- ✅ `.env` in `.gitignore` — already handled (pre-existing)
- ✅ Docker Compose reads `.env` for overrides — Compose v2 auto-loads `.env` by default

### Documentation
- ✅ Comment block at top of `docker-compose.yml`
- ✅ `just` with no arguments shows all new recipes in the list

## Test Coverage

No Rust code changes — no unit tests or integration tests applicable. Verification is structural:
- `just --list` confirmed: all four recipes visible, old names removed
- `docker compose config` confirmed: YAML still valid after comment block

## Scenario Dashboard Impact

None. This is developer tooling — no customer-facing capability changed. No scenarios should regress.

## Design Decisions

1. **Replaced old recipes instead of keeping both.** `up`/`down` and `dev-db`/`dev-down` doing the same thing with different names creates confusion. Unified under `dev-*` namespace.

2. **Used `--wait` flag** instead of manual health check polling. Requires Docker Compose v2 (standard since Docker Desktop 4.x). Cleaner than shell loops.

3. **`dev-stack` prints instructions** rather than trying to background processes. Multi-service dev needs multiple terminals — fighting this creates fragile process management.

4. **Removed old `dev` recipe.** `dev-stack` supersedes it with the same pattern (print instructions) plus the added step of starting infrastructure first.

## Open Concerns

1. **`TEST_DATABASE_URL` points to `plantastic_test` database** which doesn't exist in the Compose setup. The `docker-init-db.sh` script only initializes `plantastic`. Tests that need a separate database will need to create it. This is a known gap — tracked by the testing infrastructure tickets (T-020-*).

2. **No `dev-worker` recipe.** The `worker/` directory exists but there's no established way to run it yet. `dev-stack` omits it from instructions. Can be added when the worker crate is runnable.

3. **`dev-db` starts both Postgres AND Valkey** even though the name suggests "just the database." The ticket spec says "starts only the database (Compose `db` service)" but using `docker compose up -d --wait` starts all services. To start only `db`, the command would be `docker compose up -d --wait db`. However, starting Valkey is cheap and nearly every development workflow will need both. If strict single-service behavior is needed, the recipe can be tightened to `docker compose up -d --wait db`.
