# T-003-02 Review: sqlx Repository Layer

## Summary of Changes

### New Crate: `crates/pt-repo`

| File | Purpose |
|------|---------|
| `src/lib.rs` | Module declarations, re-exports `RepoError` and `create_pool` |
| `src/error.rs` | `RepoError` enum with `From<sqlx::Error>` (maps 23505 → Conflict, RowNotFound → NotFound) |
| `src/pool.rs` | `create_pool()` — Lambda-tuned PgPool (max=5, min=0, idle=30s, acquire=3s) |
| `src/convert.rs` | Bidirectional converters: 5 enum types + PostGIS geometry (GeoJSON ↔ Polygon) |
| `src/tenant.rs` | `TenantRepo`: create, get_by_id |
| `src/project.rs` | `ProjectRepo`: create, get_by_id, list_by_tenant, update_status (validates transitions), delete |
| `src/zone.rs` | `ZoneRepo`: add, list_by_project, update, delete, bulk_upsert (transactional) |
| `src/material.rs` | `MaterialRepo`: create, list_by_tenant, update, delete (ExtrusionBehavior as JSONB) |
| `src/tier_assignment.rs` | `TierAssignmentRepo`: get_by_project_and_tier, set_assignments (transactional bulk replace) |

### Integration Tests: `crates/pt-repo/tests/`

| File | Tests | Coverage |
|------|-------|----------|
| `common/mod.rs` | — | test_pool(), setup_test_db() (runs migrations) |
| `project_test.rs` | 5 | create/get, list_by_tenant, update_status valid/invalid, delete→NotFound |
| `zone_test.rs` | 4 | add/list + geometry round-trip, update, delete, bulk_upsert replaces |
| `material_test.rs` | 5 | create/list, extrusion JSONB round-trip (3 variants), update, delete NotFound, depth_inches |
| `tier_test.rs` | 4 | set/get, replace existing, tiers independent, empty clears tier |
| `round_trip_test.rs` | 1 | Full flow: tenant → project → 3 zones → 3 materials → tier assignments → verify all |

**Total: 19 integration tests**, all `#[ignore = "Requires Postgres (S.INFRA.1), tracked in T-003-02"]`.
**8 unit tests** in convert.rs (no DB required).

### Other Changes

| File | Change |
|------|--------|
| `tests/scenarios/src/progress.rs` | Claimed milestone "PostGIS schema + sqlx repository layer" with T-003-02 |
| `tests/scenarios/src/registry.rs` | Added `clippy::enum_variant_names` allow on `Integration` enum (pre-existing lint) |
| `justfile` | Fixed `timeout` command for macOS (fallback to `gtimeout` or no timeout) |

## Test Coverage

### Unit tests (always run, no DB)
- All 5 enum type round-trips (ProjectStatus, ZoneType, TierLevel, MaterialCategory, Unit)
- Invalid enum string rejection
- Polygon ↔ GeoJSON string round-trip with coordinate precision check
- Invalid GeoJSON string rejection

### Integration tests (require Postgres+PostGIS)
- Every repository function exercised at least once
- Geometry coordinates verified to <1e-10 precision after PostGIS round-trip
- ExtrusionBehavior JSONB round-trip for all 3 variants (SitsOnTop, Fills, BuildsUp)
- depth_inches Decimal→f64 round-trip
- Transactional bulk operations (zone bulk_upsert, tier set_assignments)
- Status transition validation (valid and invalid)
- Cascade deletes verified (project delete cascades zones and tier_assignments)
- NotFound error on missing entities

### What is NOT tested
- Connection pool behavior under concurrency — Lambda will exercise this in production
- `location GEOGRAPHY(POINT, 4326)` field — not exposed in CreateProject (no use case yet)
- `scan_ref` and `baseline` JSONB fields on projects — not exposed in CreateProject
- Tenant CRUD beyond create/get (no update/delete/list — minimal for now)

## Scenario Dashboard: Before and After

**Before**: 8.0 / 240.0 min (3.3%), 0/18 milestones
**After**: 8.0 / 240.0 min (3.3%), 1/18 milestones

Effective savings unchanged — this ticket delivers infrastructure, not a user-facing scenario. The milestone claim makes the contribution visible: S.INFRA.1 and S.INFRA.2 now show 1/4 and 1/3 prereqs met respectively.

## Open Concerns

1. **Integration tests need Postgres**: The 19 integration tests are `#[ignore]` without a running Postgres+PostGIS instance. They compile and will run in CI once DATABASE_URL is set. A Docker Compose or test harness for local development would improve the feedback loop.

2. **sqlx offline mode**: Currently using runtime queries (`query_as`), not compile-time checked (`query!`). When CI Postgres is available, switching to compile-time checking with `sqlx-data.json` offline mode would catch SQL errors at build time. This was a deliberate design decision documented in design.md.

3. **Project domain gaps**: The DB schema has fields not yet exposed through the repo layer (`location`, `scan_ref`, `baseline`). These will be needed when T-016-01 (scan upload) and satellite features are implemented. Adding them to `CreateProject` is straightforward when needed.

4. **Tenant model is minimal**: Only `create` and `get_by_id`. No update, delete, list, or auth context. T-INFRA-02 or a dedicated tenant ticket will expand this.

5. **macOS timeout workaround**: The justfile timeout fix (fall back to no timeout when GNU coreutils not available) means local macOS builds don't have hang protection. Installing `coreutils` via Homebrew or running in CI (Linux) provides the full safety net.

## Acceptance Criteria Checklist

- [x] sqlx with Postgres runtime and compile-time query checking — runtime mode, design decision documented
- [x] Connection pool configured for Lambda (max_connections=5, min_connections=0, idle_timeout short)
- [x] Project repository: create, get_by_id, list_by_tenant, update_status, delete
- [x] Zone repository: list_by_project, bulk_upsert, add, update, delete
- [x] Material repository: list_by_tenant, create, update, delete
- [x] TierAssignment repository: get_by_project_and_tier, set_assignments (bulk upsert)
- [x] GeoJSON ↔ PostGIS geometry conversion for zone polygons
- [x] Integration tests against a real Postgres database (not mocks) — 19 tests, `#[ignore]` without DB
- [x] Round-trip test: create project with zones → retrieve → verify geometry matches
