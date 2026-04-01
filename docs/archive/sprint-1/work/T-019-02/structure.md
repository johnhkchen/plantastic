# T-019-02 Structure — Dev Stack Recipes

## Files Modified

### 1. `justfile`

**Remove recipes:**
- `up` (line ~98)
- `down` (line ~101)
- `down-clean` (line ~104)
- `dev` (line ~145)

**Add recipes in the "Docker / Dev Environment" section:**

```
dev-db       — start only Postgres+Valkey, wait for health, print connection string
dev-stack    — start compose services + print commands for API/web/worker
dev-down     — stop all compose services
dev-reset    — nuke volumes and restart (clean slate)
```

**Recipe signatures:**

`dev-db`:
- `docker compose up -d --wait`
- Print `DATABASE_URL` and `VALKEY_URL` connection strings
- Single bash block

`dev-stack`:
- Call `just dev-db` (starts infra, waits)
- Print instructions for running API, web, worker in separate terminals
- Mention `just dev-down` to stop everything

`dev-down`:
- `docker compose down`
- Direct replacement for old `down`

`dev-reset`:
- `docker compose down -v && docker compose up -d --wait`
- Print connection strings after restart

**Keep unchanged:**
- `dev-api`, `dev-web` — still needed as standalone recipes
- `migrate`, `migrate-direct`, `db-migrate`, `db-reset` — database management stays

### 2. `.env.example`

Add variables (keeping existing ones, reorganizing with section headers):

```
# Database
DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic
TEST_DATABASE_URL=postgres://plantastic:plantastic@localhost:5432/plantastic_test

# Valkey (Redis-compatible)
VALKEY_URL=redis://localhost:6379

# API
PORT=3000
RUST_LOG=info

# S3 / Object Storage
S3_BUCKET=plantastic-dev
S3_ENDPOINT=http://localhost:4566
AWS_REGION=us-west-2
# AWS_ACCESS_KEY_ID=...
# AWS_SECRET_ACCESS_KEY=...

# Frontend (web/)
VITE_MOCK_API=false
# API_URL=http://localhost:3000
```

### 3. `docker-compose.yml`

Add comment block at top (before `services:`):

```yaml
# Plantastic local development services
#
# Services:
#   db      — PostgreSQL 18 + PostGIS 3.6 (port 5432)
#   valkey  — Valkey 8, Redis-compatible cache (port 6379)
#
# Usage:
#   just dev-db      Start database + cache, wait for health
#   just dev-stack   Start everything + print next steps
#   just dev-down    Stop services
#   just dev-reset   Nuke volumes + restart fresh
#
# Migrations are applied automatically on first start via
# scripts/docker-init-db.sh mounted into the Postgres entrypoint.
```

No structural changes to services — only the comment block added.

## Files NOT Modified
- `.gitignore` — already correct
- `.doppler.yaml` — unrelated
- `scripts/*` — no changes needed
- `migrations/*` — no changes needed

## Public Interface Changes
- `just` (no args) will show new `dev-*` recipes in the list
- `just up`/`just down`/`just down-clean`/`just dev` will stop working (removed)
- `just dev-db`/`just dev-stack`/`just dev-down`/`just dev-reset` are the replacements
