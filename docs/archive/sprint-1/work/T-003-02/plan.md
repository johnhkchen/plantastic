# T-003-02 Plan: sqlx Repository Layer

## Step 1: Add workspace dependencies + create crate skeleton

- Add sqlx, tokio, thiserror to workspace `[workspace.dependencies]`
- Create `crates/pt-repo/Cargo.toml` with all dependencies
- Create `crates/pt-repo/src/lib.rs` with module declarations
- **Verify**: `cargo check -p pt-repo` compiles

## Step 2: Implement error.rs and pool.rs

- `RepoError` enum with `NotFound`, `Conflict(String)`, `Database(sqlx::Error)`, `Conversion(String)`
- `impl From<sqlx::Error>` for automatic conversion
- `impl Display` and `impl Error`
- `create_pool()` with Lambda-tuned settings
- **Verify**: `cargo check -p pt-repo`

## Step 3: Implement convert.rs

- String ↔ enum converters for ProjectStatus, ZoneType, TierLevel, MaterialCategory, Unit
- `polygon_to_geojson_string()` and `geojson_string_to_polygon()` using geojson crate
- Unit tests for all converters (round-trip each enum variant, geometry conversion)
- **Verify**: `cargo test -p pt-repo` (unit tests only, no DB needed)

## Step 4: Implement tenant.rs

- TenantRow struct
- `create()` and `get_by_id()` functions
- **Verify**: `cargo check -p pt-repo`

## Step 5: Implement project.rs

- ProjectRow and CreateProject structs
- `create()`, `get_by_id()`, `list_by_tenant()`, `update_status()`, `delete()`
- update_status validates transition via `ProjectStatus::can_transition_to()`
- **Verify**: `cargo check -p pt-repo`

## Step 6: Implement zone.rs

- ZoneRow and CreateZone structs
- `list_by_project()`, `add()`, `update()`, `delete()`, `bulk_upsert()`
- Geometry: `ST_GeomFromGeoJSON()` on insert, `ST_AsGeoJSON(geometry)::text` on select
- bulk_upsert: delete existing zones for project, insert new ones, all in a transaction
- **Verify**: `cargo check -p pt-repo`

## Step 7: Implement material.rs

- MaterialRow and CreateMaterial structs
- `list_by_tenant()`, `create()`, `update()`, `delete()`
- Extrusion stored as JSONB via `sqlx::types::Json<serde_json::Value>`
- **Verify**: `cargo check -p pt-repo`

## Step 8: Implement tier_assignment.rs

- TierAssignmentRow and SetAssignment structs
- `get_by_project_and_tier()`, `set_assignments()`
- set_assignments: transaction — DELETE WHERE project_id + tier, then INSERT batch
- Overrides stored as JSONB via `sqlx::types::Json<serde_json::Value>`
- **Verify**: `cargo check -p pt-repo`

## Step 9: Integration test infrastructure

- `tests/common/mod.rs`: `test_pool()` connects to DATABASE_URL, `setup_test_db()` runs migrations
- All integration tests marked `#[ignore = "Requires Postgres (S.INFRA.1)"]` so `cargo test` passes without a DB
- When DATABASE_URL is set, tests run via `cargo test -- --ignored`
- **Verify**: `cargo test -p pt-repo` passes (ignored tests don't fail)

## Step 10: Integration tests

- `project_test.rs`: create → get → list_by_tenant → update_status → delete → get returns NotFound
- `zone_test.rs`: add zone with geometry → list → verify geometry coordinates match → update → delete
- `material_test.rs`: create → list_by_tenant → update → delete → verify extrusion JSONB round-trip
- `tier_test.rs`: set_assignments → get → verify → set again (replace) → verify unique constraint
- `round_trip_test.rs`: create project → add zones with known polygons → assign materials to tiers → fetch everything → verify geometry coordinates match originals
- **Verify**: all tests pass with a running Postgres+PostGIS

## Step 11: Claim milestone + final check

- Update `tests/scenarios/src/progress.rs`: set `delivered_by: Some("T-003-02")` for "PostGIS schema + sqlx repository layer" milestone
- Add note describing what was delivered
- Run `just check` — format, lint, test, scenarios must all pass
- **Verify**: `just check` green

## Testing Strategy Summary

| Layer | What | DB required? |
|-------|------|-------------|
| Unit (convert.rs) | Enum round-trips, geometry conversion | No |
| Integration (tests/) | Full CRUD against real Postgres | Yes (ignored without DB) |
| Scenario (progress.rs) | Milestone claim only — scenarios stay NotImplemented until T-004-01 | No |
