# T-038-03 Structure: db-scenario-recipe

## Files modified

### 1. `justfile`

**Add recipe:** `scenarios-db` in the Scenarios section (after line ~107, after existing `scenarios` recipe).

Recipe structure:
- Bash shebang script (consistent with other multi-line recipes like `test`, `dev-db`)
- Docker availability check → helpful error message
- `docker compose up -d db` (start DB only, idempotent)
- `pg_isready` health loop (30 attempts, 1s sleep)
- `DATABASE_URL=...localhost:5433/plantastic cargo run -p pt-scenarios`

**Fix port references:** Change 5432 → 5433 in three locations:
- `test-integration` recipe (line 75): default DATABASE_URL
- `dev-db` recipe (line 164): printed connection string
- `dev-reset` recipe (line 192): printed connection string

### 2. `.env.example`

**Line 11:** `DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic`
→ `DATABASE_URL=postgres://plantastic:plantastic@localhost:5433/plantastic`

**Line 20:** `TEST_DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic_test`
→ `TEST_DATABASE_URL=postgres://plantastic:plantastic@localhost:5433/plantastic_test`

Add comment on DATABASE_URL line noting it matches Docker Compose port mapping.

### 3. `CLAUDE.md` (project root: `/Users/johnchen/swe/repos/plantastic/CLAUDE.md`)

Add `just scenarios-db` to the Key Commands code block, between `just scenarios` and `just lint`.

## Files NOT modified
- `docker-compose.yml` — port mapping is already correct (5433:5432)
- `tests/scenarios/` — no code changes needed
- No new files created

## Module boundaries
No new modules or crates. All changes are to build/dev tooling files.

## Ordering
1. Fix port references first (justfile + .env.example) — these are independent bug fixes
2. Add `scenarios-db` recipe — new functionality
3. Update CLAUDE.md — documentation
