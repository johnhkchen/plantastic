# T-004-02 Research: CRUD Routes

## What exists

### API skeleton (T-004-01)
- `plantastic-api` binary crate at `crates/plantastic-api/`
- `main.rs`: dual-mode startup (Lambda via `lambda_http::run`, local via `axum::serve`)
- `state.rs`: `AppState { pool: PgPool }` — shared via `State` extractor
- `error.rs`: `AppError { NotFound, BadRequest, Conflict, Internal }` with `IntoResponse` + `From<RepoError>`
- `routes/mod.rs`: assembles router — currently only merges `health::routes()`
- `routes/health.rs`: `GET /health` → `{ status: "ok", version }` — pattern to follow
- Middleware: `TraceLayer::new_for_http()` + `CorsLayer::permissive()`
- `#[allow(dead_code)]` on `mod error` and `mod state` — placed for this ticket

### Repository layer (T-003-02)
All functions are in `pt_repo::` submodules, take `&PgPool` as first arg, return `Result<_, RepoError>`.

**Project:** `create(&CreateProject) → Uuid`, `get_by_id(Uuid) → ProjectRow`, `list_by_tenant(Uuid) → Vec<ProjectRow>`, `update_status(Uuid, ProjectStatus)`, `delete(Uuid)`
- `CreateProject`: `{ tenant_id, client_name?, client_email?, address? }`
- `ProjectRow`: `{ id, tenant_id, client_name?, client_email?, address?, scan_ref?, baseline?, status, created_at, updated_at }`

**Zone:** `list_by_project(Uuid) → Vec<ZoneRow>`, `add(&CreateZone) → Uuid`, `update(id, &Polygon, ZoneType, label?, sort_order)`, `delete(Uuid)`, `bulk_upsert(project_id, &[CreateZone]) → Vec<Uuid>`
- `CreateZone`: `{ project_id, geometry: Polygon<f64>, zone_type, label?, sort_order }`
- `ZoneRow`: `{ id, project_id, geometry: Polygon<f64>, zone_type, label?, sort_order, created_at, updated_at }`
- Geometry is stored as PostGIS `GEOMETRY(POLYGON, 4326)`, converted via GeoJSON strings

**Material:** `list_by_tenant(Uuid) → Vec<MaterialRow>`, `create(&CreateMaterial) → Uuid`, `update(id, &CreateMaterial)`, `delete(Uuid)`
- `CreateMaterial`: `{ tenant_id, name, category, unit, price_per_unit, depth_inches?, extrusion, texture_key?, photo_key?, supplier_sku? }`
- `MaterialRow`: all fields plus `{ id, created_at, updated_at }`

**Tier assignments:** `get_by_project_and_tier(project_id, TierLevel) → Vec<TierAssignmentRow>`, `set_assignments(project_id, TierLevel, &[SetAssignment])`
- `SetAssignment`: `{ zone_id, material_id, overrides?: Value }`
- `TierAssignmentRow`: `{ id, project_id, tier, zone_id, material_id, overrides?: Value, created_at, updated_at }`

**Tenant:** `create(name) → Uuid`, `get_by_id(Uuid) → TenantRow`

### Domain types (serde support)
- `pt_project::ZoneType` — `#[serde(rename_all = "snake_case")]`: bed, patio, path, lawn, wall, edging
- `pt_project::TierLevel` — `#[serde(rename_all = "snake_case")]`: good, better, best
- `pt_project::ProjectStatus` — `#[serde(rename_all = "snake_case")]`: draft, quoted, approved, complete
- `pt_project::Zone` — Serialize/Deserialize with `#[serde(with = "geojson_polygon")]` for geometry
- `pt_materials::MaterialCategory` — `#[serde(rename_all = "snake_case")]`: hardscape, softscape, edging, fill
- `pt_materials::Unit` — `#[serde(rename_all = "snake_case")]`: sq_ft, cu_yd, linear_ft, each
- `pt_materials::ExtrusionBehavior` — `#[serde(tag = "type", rename_all = "snake_case")]`

### Database schema
- `projects`: FK to `tenants(id)`, has `client_name`, `client_email`, `address`, `status`
- `zones`: FK to `projects(id) ON DELETE CASCADE`, PostGIS geometry, zone_type, label, sort_order
- `materials`: FK to `tenants(id)`, all material fields, UNIQUE not explicit beyond PK
- `tier_assignments`: FK to `projects(id) ON DELETE CASCADE`, FK to `zones(id) ON DELETE CASCADE`, FK to `materials(id)`, UNIQUE on `(project_id, tier, zone_id)`

### Dependencies in Cargo.toml
`plantastic-api` already depends on: axum, tower-http, lambda_http, tokio, tracing, serde, serde_json, dotenvy, sqlx, pt-repo. Dev-deps: pt-test-utils, tower (with "util"), http-body-util.

### Existing test pattern
- `health_test.rs`: creates a minimal router (no DB), uses `tower::ServiceExt::oneshot` to send requests
- Integration tests for pt-repo use `#[ignore = "Requires Postgres"]` + real pool + migrations

## What's missing

1. **Route handler modules** — no `routes/projects.rs`, `routes/zones.rs`, `routes/materials.rs`, `routes/tiers.rs`
2. **Request/response DTOs** — need serde structs for JSON input/output at the API boundary (separate from repo input types)
3. **Tenant extraction** — ticket says "placeholder: tenant_id from header or hardcoded for V1". No middleware/extractor yet.
4. **Input validation** — ticket requires "meaningful error messages". Need to validate required fields, valid UUIDs, etc.
5. **Geometry serialization at API boundary** — zones contain `Polygon<f64>` which needs GeoJSON in JSON requests/responses
6. **Router wiring** — `routes/mod.rs` needs to merge the new route modules
7. **UUID path extraction** — Axum `Path` extractor for `:id` params
8. **pt-project and pt-materials dependencies** — `plantastic-api` Cargo.toml only has `pt-repo`, may need domain crate deps for types in DTOs

## Constraints & risks

1. **No auth yet** — V1 uses a header-based tenant_id placeholder. Must not leak data across tenants.
2. **Geometry round-trip** — GeoJSON in requests must match the PostGIS storage format. The `pt_project::Zone` type already handles this via serde, but we need to ensure API DTOs use the same approach.
3. **Bulk zone PUT** — The repo's `bulk_upsert` deletes all existing zones and re-inserts. This has cascading implications for tier_assignments (FK on zones). The frontend needs to send complete zone sets.
4. **Tier routes** — `GET /projects/:id/tiers` needs to return all three tiers. The repo only fetches one tier at a time, so we call it three times.
5. **No `get_by_id` for zones or materials** — The repo has `list_by_project` and `list_by_tenant` but no single-entity gets. Zone PATCH/DELETE and material PATCH/DELETE only need the ID (which they have from the path), so this is fine.
6. **Project tenant scoping** — `get_by_id` doesn't filter by tenant. Need to verify tenant ownership after fetching, or add a tenant-scoped query.

## Patterns to follow

- Health route pattern: function returns `Router<AppState>`, merged in `routes/mod.rs`
- Error conversion chain: `RepoError → AppError → HTTP response` via `From` impl and `?` operator
- Request parsing: `Json<T>` extractor for body, `Path<Uuid>` for URL params, `State(state)` for pool
- Response: `Result<Json<T>, AppError>` or `Result<StatusCode, AppError>`
