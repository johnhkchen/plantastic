# T-003-01 Plan: PostGIS Migrations

## Implementation Steps

### Step 1: Update justfile db-migrate

Modify `just db-migrate` to exclude `*.down.sql` files from the migration run.

**Change**: One line in `justfile` — add `grep -v '\.down\.sql'` to the pipeline.

**Verify**: `just db-migrate` should only list forward migrations (once they exist).

### Step 2: Write Migration 001 — tenants

Create `migrations/001-create-tenants.sql` and `migrations/001-create-tenants.down.sql`.

**Up**: CREATE EXTENSION IF NOT EXISTS postgis, CREATE TABLE tenants.
**Down**: DROP TABLE IF EXISTS tenants CASCADE.

**Verify**: Apply on fresh database, inspect with `\d tenants`.

### Step 3: Write Migration 002 — projects

Create `migrations/002-create-projects.sql` and `migrations/002-create-projects.down.sql`.

**Up**: CREATE TABLE projects with tenant_id FK, status CHECK, GEOGRAPHY point, indexes.
**Down**: DROP TABLE IF EXISTS projects CASCADE.

**Verify**: FK to tenants exists, CHECK constraint rejects invalid status values.

### Step 4: Write Migration 003 — zones

Create `migrations/003-create-zones.sql` and `migrations/003-create-zones.down.sql`.

**Up**: CREATE TABLE zones with project_id FK CASCADE, GEOMETRY(POLYGON, 4326), GIST index.
**Down**: DROP TABLE IF EXISTS zones CASCADE.

**Verify**: Spatial index exists (`\di`), CASCADE delete works from projects.

### Step 5: Write Migration 004 — materials

Create `migrations/004-create-materials.sql` and `migrations/004-create-materials.down.sql`.

**Up**: CREATE TABLE materials with tenant_id FK, category/unit CHECKs, JSONB extrusion.
**Down**: DROP TABLE IF EXISTS materials CASCADE.

**Verify**: CHECK constraints reject invalid category/unit values.

### Step 6: Write Migration 005 — tier_assignments

Create `migrations/005-create-tier-assignments.sql` and `migrations/005-create-tier-assignments.down.sql`.

**Up**: CREATE TABLE tier_assignments with FKs to projects/zones/materials, UNIQUE constraint.
**Down**: DROP TABLE IF EXISTS tier_assignments CASCADE.

**Verify**: UNIQUE constraint prevents duplicate (project, tier, zone) tuples.

### Step 7: Write Migration 006 — plants

Create `migrations/006-create-plants.sql` and `migrations/006-create-plants.down.sql`.

**Up**: CREATE TABLE plants (platform-level, no tenant FK), text arrays, CHECK constraints.
**Down**: DROP TABLE IF EXISTS plants CASCADE.

**Verify**: No FK to tenants, arrays work, CHECK rejects invalid sun/water values.

### Step 8: Remove .gitkeep

Delete `migrations/.gitkeep` — no longer needed now that real files exist.

### Step 9: Full validation

Run `just check` (fmt + lint + test + scenarios) to ensure no regressions. The migrations are pure SQL so they won't affect Rust compilation, but verify the workspace still builds and all tests pass.

## Testing Strategy

This ticket produces SQL files, not Rust code. Testing is:

1. **Syntax validation**: Each SQL file must parse without errors when applied via `psql`.
2. **FK integrity**: Inserting a project with a nonexistent tenant_id must fail.
3. **CHECK constraints**: Inserting invalid enum values must fail.
4. **Spatial**: PostGIS extension is available, GEOMETRY and GEOGRAPHY columns accept valid WKT/GeoJSON.
5. **UNIQUE constraint**: Duplicate (project_id, tier, zone_id) in tier_assignments must fail.
6. **Down migrations**: Each down migration drops its table cleanly; re-applying the up migration works.
7. **Full round-trip**: `just db-reset` applies all migrations on a clean database without errors.

Automated Postgres integration tests will come in T-003-02 (repository layer). For this ticket, validation is manual via `just db-reset` against a local Postgres instance, or structural review of the SQL.

## Commit Strategy

Single commit with all 12 migration files + justfile change. These are tightly coupled (FK ordering), so atomic commit makes sense.
