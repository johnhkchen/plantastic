# T-003-02 Structure: sqlx Repository Layer

## New Crate: `crates/pt-repo`

### `crates/pt-repo/Cargo.toml`
Dependencies: sqlx (postgres, runtime-tokio, tls-rustls, chrono, uuid, json), pt-project, pt-materials, geojson, serde, serde_json, uuid, chrono, rust_decimal, thiserror.
Dev-dependencies: tokio (for async tests), pt-test-utils.

### Module Layout

```
crates/pt-repo/src/
├── lib.rs              # Re-exports: pool, error, all repo modules
├── pool.rs             # create_pool() — PgPool with Lambda-tuned config
├── error.rs            # RepoError enum (NotFound, Conflict, Database, Conversion)
├── convert.rs          # Enum string ↔ domain type converters, geometry helpers
├── tenant.rs           # TenantRepo: create, get_by_id
├── project.rs          # ProjectRepo: create, get_by_id, list_by_tenant, update_status, delete
├── zone.rs             # ZoneRepo: list_by_project, add, update, delete, bulk_upsert
├── material.rs         # MaterialRepo: list_by_tenant, create, update, delete
└── tier_assignment.rs  # TierAssignmentRepo: get_by_project_and_tier, set_assignments
```

### Integration Tests

```
crates/pt-repo/tests/
├── common/
│   └── mod.rs          # test_pool(), setup_test_db(), run_migrations()
├── project_test.rs     # Project CRUD + status transitions against real Postgres
├── zone_test.rs        # Zone CRUD + geometry round-trip (PostGIS → geo::Polygon)
├── material_test.rs    # Material CRUD + JSONB extrusion round-trip
├── tier_test.rs        # TierAssignment set/get + constraint enforcement
└── round_trip_test.rs  # Full create-project-with-zones → retrieve → verify geometry
```

## File Details

### `lib.rs`
Public API surface:
- `pub mod pool` → `create_pool()`
- `pub mod error` → `RepoError`
- `pub mod tenant` → `TenantRepo`
- `pub mod project` → `ProjectRepo`
- `pub mod zone` → `ZoneRepo`
- `pub mod material` → `MaterialRepo`
- `pub mod tier_assignment` → `TierAssignmentRepo`

Re-exports the main types for ergonomic imports.

### `pool.rs`
Single function: `create_pool(database_url: &str) -> Result<PgPool, RepoError>`.
Lambda-tuned: max_connections=5, min_connections=0, idle_timeout=30s, acquire_timeout=3s.

### `error.rs`
```rust
pub enum RepoError {
    NotFound,
    Conflict(String),
    Database(sqlx::Error),
    Conversion(String),
}
impl From<sqlx::Error> for RepoError  // auto-convert
```
Implements `std::error::Error` and `Display`.

### `convert.rs`
Private helpers used by repo modules:
- `parse_project_status(s: &str) -> Result<ProjectStatus, RepoError>`
- `project_status_to_str(s: ProjectStatus) -> &'static str`
- `parse_zone_type(s: &str) -> Result<ZoneType, RepoError>`
- `zone_type_to_str(t: ZoneType) -> &'static str`
- `parse_tier_level(s: &str) -> Result<TierLevel, RepoError>`
- `tier_level_to_str(t: TierLevel) -> &'static str`
- `parse_material_category(s: &str) -> Result<MaterialCategory, RepoError>`
- `category_to_str(c: MaterialCategory) -> &'static str`
- `parse_unit(s: &str) -> Result<Unit, RepoError>`
- `unit_to_str(u: Unit) -> &'static str`
- `polygon_to_geojson_string(p: &Polygon<f64>) -> String`
- `geojson_string_to_polygon(s: &str) -> Result<Polygon<f64>, RepoError>`

### `tenant.rs`
Minimal — just enough to support integration tests that need a tenant.
- `TenantRow { id, name, logo_url, brand_color, contact, created_at, updated_at }`
- `create(pool, name) -> Result<Uuid>` — inserts a tenant, returns ID
- `get_by_id(pool, id) -> Result<TenantRow>` — fetches by ID

### `project.rs`
- `ProjectRow { id, tenant_id, client_name, client_email, address, scan_ref, baseline, status, created_at, updated_at }`
- `CreateProject { tenant_id, client_name, client_email, address }` — input struct
- `create(pool, input) -> Result<Uuid>`
- `get_by_id(pool, id) -> Result<ProjectRow>`
- `list_by_tenant(pool, tenant_id) -> Result<Vec<ProjectRow>>`
- `update_status(pool, id, new_status) -> Result<()>`
- `delete(pool, id) -> Result<()>`

### `zone.rs`
- `ZoneRow { id, project_id, geometry_geojson, zone_type, label, sort_order, created_at, updated_at }`
- `CreateZone { project_id, geometry: Polygon<f64>, zone_type, label, sort_order }`
- `list_by_project(pool, project_id) -> Result<Vec<ZoneRow>>`
- `add(pool, input) -> Result<Uuid>`
- `update(pool, id, geometry, zone_type, label, sort_order) -> Result<()>`
- `delete(pool, id) -> Result<()>`
- `bulk_upsert(pool, project_id, zones: Vec<CreateZone>) -> Result<Vec<Uuid>>`

Geometry conversion: insert uses `ST_GeomFromGeoJSON($1)`, select uses `ST_AsGeoJSON(geometry)::text`.

### `material.rs`
- `MaterialRow { id, tenant_id, name, category, unit, price_per_unit, depth_inches, extrusion, texture_key, photo_key, supplier_sku, created_at, updated_at }`
- `CreateMaterial { tenant_id, name, category, unit, price_per_unit, depth_inches, extrusion, texture_key, photo_key, supplier_sku }`
- `list_by_tenant(pool, tenant_id) -> Result<Vec<MaterialRow>>`
- `create(pool, input) -> Result<Uuid>`
- `update(pool, id, input) -> Result<()>`
- `delete(pool, id) -> Result<()>`

### `tier_assignment.rs`
- `TierAssignmentRow { id, project_id, tier, zone_id, material_id, overrides, created_at, updated_at }`
- `SetAssignment { zone_id, material_id, overrides }`
- `get_by_project_and_tier(pool, project_id, tier) -> Result<Vec<TierAssignmentRow>>`
- `set_assignments(pool, project_id, tier, assignments: Vec<SetAssignment>) -> Result<()>` — DELETE existing + INSERT new in a transaction (bulk upsert)

## Workspace Changes

### `Cargo.toml` (workspace root)
Add to `[workspace.dependencies]`:
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "chrono", "uuid", "json"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
thiserror = "2"
```

### No changes to existing crates
pt-project, pt-materials, pt-geo remain untouched. The repo crate adapts to them, not the reverse.

## Ordering Constraints
1. Workspace deps must be added first (Cargo.toml)
2. error.rs and pool.rs have no internal dependencies — can be written first
3. convert.rs depends on pt-project and pt-materials types
4. tenant.rs is standalone (needed by other repos' tests)
5. project.rs, zone.rs, material.rs, tier_assignment.rs can be written in any order
6. Integration tests depend on all of the above plus a running Postgres
