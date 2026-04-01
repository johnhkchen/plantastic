# T-003-01 Review: PostGIS Migrations

## Summary

Created 6 versioned SQL migrations (plus 6 rollbacks) that define the Plantastic database schema on Postgres 16 + PostGIS 3.4. Updated the justfile to exclude down-migration files from the forward run.

## What Changed

### Files Created (12)
| File | Purpose |
|------|---------|
| `migrations/001-create-tenants.sql` | Multi-tenancy root table |
| `migrations/002-create-projects.sql` | Project aggregate with GEOGRAPHY point |
| `migrations/003-create-zones.sql` | Spatial zones with GEOMETRY polygon + GIST index |
| `migrations/004-create-materials.sql` | Tenant-scoped material catalog |
| `migrations/005-create-tier-assignments.sql` | Material-to-zone-per-tier with UNIQUE constraint |
| `migrations/006-create-plants.sql` | Platform-level plant database |
| `*.down.sql` (6 files) | DROP TABLE rollbacks for each migration |

### Files Modified (1)
- **justfile**: `db-migrate` now pipes through `grep -v '\.down\.sql'` to skip rollback files.

### Files Deleted (1)
- **migrations/.gitkeep**: Replaced by actual migration files.

## Schema Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Enum storage | VARCHAR + CHECK | Flexible; matches serde snake_case; easy to extend |
| Decimal precision | DECIMAL(12,4) | Supports sub-cent pricing; aligns with pt-quote 4dp quantities |
| Timestamps | TIMESTAMPTZ DEFAULT now() | App-managed updated_at (no triggers) |
| PKs | UUID DEFAULT gen_random_uuid() | Matches Rust Uuid::new_v4() convention |
| Variant data | JSONB | For extrusion, overrides, contact, scan_ref, baseline |
| Down migrations | `.down.sql` suffix | Paired with up files; excluded by grep in db-migrate |

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| Migration 001: tenants (UUID, name, logo, brand, JSONB contact, timestamps) | Done |
| Migration 002: projects (UUID, tenant FK, client fields, GEOGRAPHY point, JSONB scan/baseline, status, timestamps) | Done |
| Migration 003: zones (UUID, project FK CASCADE, GEOMETRY POLYGON 4326, zone_type, label, sort_order) | Done |
| Migration 004: materials (UUID, tenant FK, name, category, unit, DECIMAL price, depth, JSONB extrusion, texture/photo/sku) | Done |
| Migration 005: tier_assignments (UUID, project FK CASCADE, tier, zone FK CASCADE, material FK, JSONB overrides, UNIQUE) | Done |
| Migration 006: plants (UUID, common/botanical name, sun/climate/size, TEXT[] tags, photo_url) | Done |
| PostGIS extension enabled | Done (in 001, idempotent) |
| Spatial index on zones.geometry | Done (GIST index in 003) |
| Clean apply on fresh Postgres 16 + PostGIS 3.4 | Structural — no local PG to test; schema is standard SQL |
| Down migrations for rollback | Done (6 `.down.sql` files) |

## Test Coverage

This ticket produces SQL files, not Rust code. No new Rust tests were added or needed.

- **Rust workspace**: 72 tests pass, 0 fail. No regressions.
- **Scenarios**: 40.0/240.0 min verified savings. 2 pass, 0 fail. Unchanged from baseline.
- **SQL validation**: Structural review only — no local Postgres instance available for runtime verification. T-003-02 (repository layer) will provide integration tests that exercise these tables with real queries.

## Scenario Dashboard (Before → After)

Before: 40.0 min / 240.0 min (16.7%), 2 pass, 0 fail
After: 40.0 min / 240.0 min (16.7%), 2 pass, 0 fail

No change expected — this ticket creates persistence infrastructure, not new capabilities. The value shows up when T-003-02 (repository layer) and T-004-01 (API routes) connect domain logic to the database.

## Open Concerns

1. **No runtime validation**: These migrations haven't been applied to a real database yet. The first real test will be when T-003-02 sets up integration tests. If there's a syntax issue or PostGIS version incompatibility, it'll surface there.

2. **`just test` timeout command**: `just check` fails on macOS because `timeout` is a GNU coreutils command not present by default. This is a pre-existing issue (not introduced by this ticket). Fix: `brew install coreutils` or replace with `gtimeout`.

3. **Coordinate system note**: Zone geometry uses SRID 4326 (WGS84 degrees) in the database, but the Rust domain model (`pt-geo`) computes in feet. The repository layer (T-003-02) will need to handle this — either by storing in a projected CRS and converting, or by having the application manage the unit context. Not a blocker for this ticket.

4. **Plants table columns**: The ticket said "sun/climate/size fields" without exact column names. I chose `sun_requirement`, `water_need`, `climate_zones` (TEXT[]), `mature_height_ft`, `mature_width_ft` based on the specification's plant intelligence section. These may need adjustment when the pt-plant crate is built.

5. **Material unit serde format**: The Rust `Unit` enum serializes as `sq_ft`, `cu_yd`, `linear_ft`, `each` (snake_case). The CHECK constraint in 004 matches these exactly. If the serde format changes, the CHECK must be updated too.
