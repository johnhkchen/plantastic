# T-003-02 Research: sqlx Repository Layer

## Domain Types (what gets persisted)

### pt-project crate (`crates/pt-project/src/`)
- **Project** (`project.rs`): `id: Uuid`, `scan_ref: Option<String>`, `zones: Vec<Zone>`, `tiers: Vec<Tier>`, `status: ProjectStatus`, `created_at/updated_at: DateTime<Utc>`. Missing from domain but in DB schema: `tenant_id`, `client_name`, `client_email`, `address`, `location`, `baseline`.
- **Zone** (`types.rs`): `id: ZoneId(Uuid)`, `geometry: Polygon<f64>`, `zone_type: ZoneType`, `label: Option<String>`. DB also has `sort_order`.
- **ZoneType**: enum `{Bed, Patio, Path, Lawn, Wall, Edging}`, serde snake_case. DB stores as VARCHAR CHECK.
- **ProjectStatus**: enum `{Draft, Quoted, Approved, Complete}`, serde snake_case. DB stores as VARCHAR CHECK.
- **TierLevel**: enum `{Good, Better, Best}`. DB tier_assignments.tier stores as VARCHAR CHECK.
- **MaterialAssignment** (`types.rs`): `zone_id: ZoneId`, `material_id: MaterialId`, `overrides: Option<AssignmentOverrides>`. Maps to tier_assignments table.
- **AssignmentOverrides**: `price_override: Option<Decimal>`, `depth_override_inches: Option<f64>`. DB stores as JSONB.

### pt-materials crate (`crates/pt-materials/src/`)
- **Material** (`types.rs`): `id: MaterialId(Uuid)`, `name: String`, `category: MaterialCategory`, `unit: Unit`, `price_per_unit: Decimal`, `depth_inches: Option<f64>`, `texture_ref/photo_ref/supplier_sku: Option<String>`, `extrusion: ExtrusionBehavior`. Missing from domain but in DB: `tenant_id`.
- **ExtrusionBehavior**: tagged enum stored as JSONB in DB.
- **MaterialCategory**: `{Hardscape, Softscape, Edging, Fill}`, snake_case → VARCHAR CHECK.
- **Unit**: `{SqFt, CuYd, LinearFt, Each}`, snake_case → VARCHAR CHECK.

### Tenant (no crate yet)
- DB schema: `tenants(id UUID, name, logo_url, brand_color, contact JSONB, timestamps)`.
- No Rust type exists. The repo layer needs at least a minimal Tenant struct or just use raw queries for now.

## Database Schema (T-003-01 delivered)

Six migrations in `migrations/`:
1. **tenants**: UUID PK, name, logo_url, brand_color, contact JSONB, timestamps. Also creates PostGIS extension.
2. **projects**: UUID PK, tenant_id FK, client_name, client_email, address, location GEOGRAPHY(POINT, 4326), scan_ref JSONB, baseline JSONB, status VARCHAR CHECK, timestamps. Index on tenant_id.
3. **zones**: UUID PK, project_id FK CASCADE, geometry GEOMETRY(POLYGON, 4326) NOT NULL, zone_type VARCHAR CHECK, label, sort_order INT, timestamps. GIST spatial index on geometry.
4. **materials**: UUID PK, tenant_id FK, name, category VARCHAR CHECK, unit VARCHAR CHECK, price_per_unit DECIMAL(12,4), depth_inches NUMERIC, extrusion JSONB, texture_key, photo_key, supplier_sku, timestamps. Index on tenant_id.
5. **tier_assignments**: UUID PK, project_id FK CASCADE, tier VARCHAR CHECK, zone_id FK CASCADE, material_id FK, overrides JSONB, timestamps. UNIQUE(project_id, tier, zone_id). Index on (project_id, tier).
6. **plants**: UUID PK, common_name, botanical_name, sun/water/climate fields, tags array, photo_url, timestamps. Index on botanical_name.

## Domain ↔ DB Gaps

1. **Project domain type is missing multi-tenant fields**: `tenant_id`, `client_name`, `client_email`, `address`, `location`, `baseline` are in DB but not in `Project` struct. The repo layer needs intermediate "row" types that include these.
2. **Zone domain type lacks `sort_order`**: DB has it, domain doesn't. Need to decide: add to domain or handle in repo only.
3. **Material domain type lacks `tenant_id`**: DB scopes materials per tenant. Repo layer must accept tenant_id as a parameter.
4. **GeoJSON ↔ PostGIS**: Zone geometry is `geo::Polygon<f64>` in Rust, `GEOMETRY(POLYGON, 4326)` in DB. Need conversion via WKB or GeoJSON text. sqlx doesn't have native PostGIS support — must use `ST_AsGeoJSON`/`ST_GeomFromGeoJSON` in SQL or a WKB codec.
5. **Enum string mapping**: ZoneType/ProjectStatus/TierLevel/MaterialCategory/Unit are serde snake_case in Rust. DB stores the same strings via VARCHAR CHECK. Need `sqlx::Type` or manual string conversion.
6. **JSONB fields**: scan_ref (Project), baseline (Project), contact (Tenant), extrusion (Material), overrides (TierAssignment) — stored as JSONB, need serde round-trip via `sqlx::types::Json<T>`.

## Connection Pool for Lambda

Lambda reuses execution environments across invocations but has limited concurrency. T-003-02 acceptance criteria specifies: `max_connections=5, min_connections=0, idle_timeout short`. sqlx's `PgPoolOptions` supports all of these directly.

## Existing Patterns & Constraints

- **No apps/ crate yet** — T-004-01 (Axum skeleton) depends on this ticket. The repo layer should be a standalone crate (`crates/pt-repo`) so T-004-01 can import it.
- **No mocking across crate boundaries** — integration tests must hit real Postgres.
- **geojson crate** already in workspace deps — available for PostGIS geometry conversion.
- **sqlx not yet in workspace** — needs to be added to `Cargo.toml` workspace dependencies.
- **pt-test-utils** provides `timed()` and `run_with_timeout()` for test timing enforcement.

## Scenario & Milestone Mapping

- Milestone "PostGIS schema + sqlx repository layer" (progress.rs) unlocks S.INFRA.1, S.INFRA.2.
- This ticket delivers the "sqlx repository layer" half of that milestone.
- S.INFRA.1 (full stack round-trip) and S.INFRA.2 (tenant isolation) won't turn green until T-004-01 (routes) is also done, but this ticket is a prerequisite.

## Key Risks

1. **sqlx compile-time checking** requires a live DB at build time (or offline mode with `sqlx-data.json`). For CI without Postgres, offline mode is essential.
2. **PostGIS geometry encoding** — sqlx doesn't natively understand PostGIS. Must use SQL-side functions (`ST_GeomFromGeoJSON`, `ST_AsGeoJSON`) or the `geozero` crate for WKB.
3. **Integration test infrastructure** — tests need a Postgres+PostGIS instance. Docker compose or a test-harness setup function.
