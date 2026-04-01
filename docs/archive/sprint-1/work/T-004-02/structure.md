# T-004-02 Structure: CRUD Routes

## Files to create

### `crates/plantastic-api/src/extract.rs`
Custom Axum extractors.
- `TenantId(Uuid)` — extracts `X-Tenant-Id` header, returns `AppError::BadRequest` if missing/invalid

### `crates/plantastic-api/src/routes/projects.rs`
Project CRUD handlers.
- `CreateProjectRequest { address?, client_name?, client_email? }` — Deserialize
- `ProjectResponse { id, tenant_id, client_name?, client_email?, address?, status, created_at, updated_at }` — Serialize
- `create_project(TenantId, State, Json<CreateProjectRequest>) → Result<(StatusCode, Json<ProjectResponse>), AppError>`
- `list_projects(TenantId, State) → Result<Json<Vec<ProjectResponse>>, AppError>`
- `get_project(TenantId, State, Path<Uuid>) → Result<Json<ProjectResponse>, AppError>` — verifies tenant ownership
- `delete_project(TenantId, State, Path<Uuid>) → Result<StatusCode, AppError>` — verifies tenant ownership
- `routes() → Router<AppState>`

### `crates/plantastic-api/src/routes/zones.rs`
Zone CRUD handlers.
- `ZoneResponse { id, project_id, geometry: Value, zone_type, label?, sort_order, created_at, updated_at }` — Serialize
- `AddZoneRequest { geometry: Value, zone_type, label?, sort_order? }` — Deserialize
- `UpdateZoneRequest { geometry: Value, zone_type, label?, sort_order? }` — Deserialize
- `BulkZoneRequest { zones: Vec<BulkZoneEntry> }` — Deserialize
- `list_zones(TenantId, State, Path<Uuid>) �� Result<Json<Vec<ZoneResponse>>>`
- `add_zone(TenantId, State, Path<Uuid>, Json) → Result<(StatusCode, Json<ZoneResponse>)>`
- `bulk_update_zones(TenantId, State, Path<Uuid>, Json) → Result<Json<Vec<Uuid>>>`
- `update_zone(TenantId, State, Path<(Uuid, Uuid)>, Json) → Result<StatusCode>`
- `delete_zone(TenantId, State, Path<(Uuid, Uuid)>, Json) → Result<StatusCode>`
- Helper: `verify_project_tenant(pool, project_id, tenant_id) → Result<(), AppError>`
- Helper: GeoJSON Value ↔ `Polygon<f64>` conversion
- `routes() → Router<AppState>`

### `crates/plantastic-api/src/routes/materials.rs`
Material CRUD handlers.
- `CreateMaterialRequest { name, category, unit, price_per_unit, depth_inches?, extrusion, texture_key?, photo_key?, supplier_sku? }` — Deserialize
- `UpdateMaterialRequest` — same fields as Create
- `MaterialResponse { id, tenant_id, name, category, unit, price_per_unit, depth_inches?, extrusion, texture_key?, photo_key?, supplier_sku?, created_at, updated_at }` — Serialize
- `list_materials(TenantId, State) → Result<Json<Vec<MaterialResponse>>>`
- `create_material(TenantId, State, Json) → Result<(StatusCode, Json<MaterialResponse>)>`
- `update_material(TenantId, State, Path<Uuid>, Json) → Result<StatusCode>`
- `delete_material(TenantId, State, Path<Uuid>) → Result<StatusCode>`
- `routes() → Router<AppState>`

### `crates/plantastic-api/src/routes/tiers.rs`
Tier assignment handlers.
- `TierResponse { tier, assignments: Vec<AssignmentResponse> }` — Serialize
- `AssignmentResponse { zone_id, material_id, overrides? }` — Serialize
- `SetAssignmentsRequest { assignments: Vec<AssignmentInput> }` — Deserialize
- `AssignmentInput { zone_id, material_id, overrides? }` — Deserialize
- `get_all_tiers(TenantId, State, Path<Uuid>) → Result<Json<Vec<TierResponse>>>`
- `set_tier_assignments(TenantId, State, Path<(Uuid, String)>, Json) ��� Result<StatusCode>`
- `routes() → Router<AppState>`

## Files to modify

### `crates/plantastic-api/src/main.rs`
- Remove `#[allow(dead_code)]` from `mod error` and `mod state` (they're now used)
- Add `mod extract;`

### `crates/plantastic-api/src/routes/mod.rs`
- Add `pub mod projects;`, `pub mod zones;`, `pub mod materials;`, `pub mod tiers;`
- Merge all four into the router: `.merge(projects::routes())` etc.

### `crates/plantastic-api/Cargo.toml`
- Add dependencies: `pt-project`, `pt-materials` (for domain types in DTOs)
- Add `chrono` (for DateTime in responses), `uuid` (for Uuid serde), `rust_decimal` (for Decimal serde)
- Add `geojson` (for GeoJSON geometry in zone DTOs)
- Add `geo` (for Polygon type in geometry conversion)

### `crates/plantastic-api/src/error.rs`
- Add a custom Axum `JsonRejection` handler so serde errors become `AppError::BadRequest`

## Files NOT changed

- All `pt-repo` files — repo layer is complete
- All `pt-project` / `pt-materials` files — domain types are complete
- Database migrations — schema is complete
- `health.rs` — no changes needed

## Module dependency graph

```
main.rs
  ├── mod extract    (TenantId extractor)
  ├── mod error      (AppError, rejection handling)
  ├─�� mod state      (AppState)
  └── mod routes
       ├── health.rs      (unchanged)
       ├── projects.rs    (uses: extract, error, state, pt_repo::project, pt_repo::tenant)
       ��── zones.rs       (uses: extract, error, state, pt_repo::zone, pt_repo::project, geojson)
       ├── materials.rs   (uses: extract, error, state, pt_repo::material)
       └── tiers.rs       (uses: extract, error, state, pt_repo::tier_assignment, pt_repo::project)
```

## Public interface boundaries

- API DTOs are `pub` within the crate (for tests) but not exported externally
- Route handler functions are private (only `routes()` is pub)
- `TenantId` extractor is `pub(crate)` — used by all route modules
- The `verify_project_tenant` helper is `pub(crate)` in zones.rs, reused by tiers.rs

## Ordering of changes

1. Cargo.toml deps (unblocks everything)
2. `extract.rs` (unblocks all handlers)
3. `error.rs` rejection handling (nice to have early)
4. `projects.rs` (no dependencies on other route modules)
5. `materials.rs` (no dependencies on other route modules)
6. `zones.rs` (needs project tenant verification helper)
7. `tiers.rs` (needs project tenant verification)
8. `routes/mod.rs` (wires everything together)
9. `main.rs` (remove dead_code allows, add extract mod)
10. Integration tests
