# T-003-01 Progress: PostGIS Migrations

## Completed

- [x] Updated `justfile` db-migrate to exclude `*.down.sql` files
- [x] Migration 001: tenants table (up + down)
- [x] Migration 002: projects table with tenant FK, GEOGRAPHY point, status CHECK (up + down)
- [x] Migration 003: zones table with project FK CASCADE, GEOMETRY polygon, GIST index (up + down)
- [x] Migration 004: materials table with tenant FK, category/unit CHECKs, JSONB extrusion (up + down)
- [x] Migration 005: tier_assignments with FKs, UNIQUE constraint (up + down)
- [x] Migration 006: plants table, platform-level, arrays, CHECK constraints (up + down)
- [x] Removed migrations/.gitkeep
- [x] Verified: `cargo fmt --check` — clean
- [x] Verified: `cargo clippy` — no warnings
- [x] Verified: `cargo test --workspace` — 72 tests pass
- [x] Verified: scenarios dashboard — 40.0/240.0 min, 2 pass, 0 fail, no regressions

## Deviations from Plan

None. All steps executed as planned.

## Files Changed

### Created (12 files)
- `migrations/001-create-tenants.sql`
- `migrations/001-create-tenants.down.sql`
- `migrations/002-create-projects.sql`
- `migrations/002-create-projects.down.sql`
- `migrations/003-create-zones.sql`
- `migrations/003-create-zones.down.sql`
- `migrations/004-create-materials.sql`
- `migrations/004-create-materials.down.sql`
- `migrations/005-create-tier-assignments.sql`
- `migrations/005-create-tier-assignments.down.sql`
- `migrations/006-create-plants.sql`
- `migrations/006-create-plants.down.sql`

### Modified (1 file)
- `justfile` — db-migrate excludes `*.down.sql`

### Deleted (1 file)
- `migrations/.gitkeep`
