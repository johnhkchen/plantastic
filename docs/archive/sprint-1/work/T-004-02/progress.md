# T-004-02 Progress: CRUD Routes

## Completed

### Step 1: Dependencies and extractors
- Updated `Cargo.toml` with pt-project, pt-materials, chrono, uuid, rust_decimal, geojson, geo deps
- Added `[lib]` section to expose router for tests
- Created `src/extract.rs` with `TenantId` extractor (X-Tenant-Id header → Uuid)
- Updated `src/error.rs` with `From<JsonRejection>` for serde error → 400 responses
- Updated `src/main.rs` to use library (`plantastic_api::router()`)
- Created `src/lib.rs` exporting `AppState` and `router()`

### Step 2: Project routes
- Created `src/routes/projects.rs` — POST/GET/GET:id/DELETE:id
- DTOs: `CreateProjectRequest`, `ProjectResponse`
- Tenant verification on get/delete (fetch → check tenant_id → 404 if mismatch)

### Step 3: Material routes
- Created `src/routes/materials.rs` — GET/POST/PATCH:id/DELETE:id
- DTOs: `CreateMaterialRequest`, `MaterialResponse`
- Tenant verification via list_by_tenant check on update/delete

### Step 4: Zone routes
- Created `src/routes/zones.rs` — GET/POST/PUT-bulk/PATCH:zid/DELETE:zid
- DTOs: `AddZoneRequest`, `UpdateZoneRequest`, `BulkZoneEntry`, `ZoneResponse`
- GeoJSON Value ↔ Polygon<f64> conversion helper
- `verify_project_tenant()` pub(crate) helper for reuse

### Step 5: Tier routes
- Created `src/routes/tiers.rs` — GET-all/PUT:tier
- DTOs: `TierResponse`, `AssignmentResponse`, `SetAssignmentsRequest`, `AssignmentInput`
- Tier level parsing from URL path (good/better/best → TierLevel)

### Step 6: Wiring and quality gate
- Updated `src/routes/mod.rs` to merge all route modules
- Ran `just check` — all gates passed

### Step 7: Integration tests and milestone
- Created `tests/common/mod.rs` with test infrastructure (pool, migrations, router, send helper)
- Created `tests/crud_test.rs` with 8 integration tests (all `#[ignore]` requiring Postgres)
- Updated milestone in `progress.rs` with T-004-02 delivery note

## Deviations from plan

1. **Extracted lib.rs** — Not in original plan. Binary crate modules are private, so integration tests can't import them. Created `lib.rs` exporting `AppState` and `router()`, refactored `main.rs` to use it. Clean separation.

2. **Material tenant verification** — Plan didn't specify how to verify material ownership. Since pt-repo has no `get_by_id` for materials, we `list_by_tenant` and check if the material is in the list. Works but is O(n) for the catalog. Acceptable for V1.
