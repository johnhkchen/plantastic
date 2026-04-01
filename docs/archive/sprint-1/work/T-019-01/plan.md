# T-019-01 Plan: Docker Compose + Migrations

## Step 1: Create init script

**File:** `scripts/docker-init-db.sh`

Write the shell script that applies migrations in order, skipping `.down.sql`.
Make it executable (`chmod +x`).

**Verify:** `bash -n scripts/docker-init-db.sh` (syntax check)

## Step 2: Create docker-compose.yml

**File:** `docker-compose.yml` (project root)

Define `db` and `valkey` services per the structure doc. Named volume for
Postgres data. Bind-mount migrations and init script.

**Verify:** `docker compose config` (validates YAML syntax and structure)

## Step 3: Update .env.example

**File:** `.env.example`

Change DATABASE_URL default to `postgres://plantastic:plantastic@localhost:5432/plantastic`.

## Step 4: Add justfile recipes

**File:** `justfile`

Add `up`, `down`, `down-clean` recipes in a new Docker section.

## Step 5: Verify the full stack

Run the acceptance criteria checks:

1. `docker compose up -d` — both services start
2. Wait for health checks to pass
3. `psql postgres://plantastic:plantastic@localhost:5432/plantastic -c "SELECT PostGIS_Version();"` — returns 3.6.x
4. Verify all 6 tables exist: `psql ... -c "\dt"`
5. `docker compose down` — clean stop
6. `docker compose down -v && docker compose up -d` — clean slate works
7. Re-verify PostGIS and tables after clean slate

## Step 6: Run quality gate

`just check` — format + lint + test + scenarios. No Rust code changed, so
this should pass unchanged.

## Testing Strategy

This ticket is infrastructure-only — no Rust code changes, so no unit tests.
Verification is manual (Steps 5-6 above). The acceptance criteria are
integration checks against the running Docker stack.

The existing test suite (`just test`) should continue to pass unchanged.
The scenario dashboard (`just scenarios`) should show no regressions.
