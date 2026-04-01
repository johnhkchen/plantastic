# T-003-01 Research: PostGIS Migrations

## What Exists

### Migration Infrastructure
- `/migrations/` directory exists but is empty (`.gitkeep` only).
- `just db-migrate` iterates `migrations/*.sql` in sorted order via `psql $DATABASE_URL`.
- `just db-reset` drops the public schema, recreates it, enables PostGIS, then calls `db-migrate`.
- No migration framework (sqlx-migrate, refinery, diesel) is in use — raw SQL files applied by `psql`.
- The naming convention is implicit: lexicographic sort of `*.sql` determines order.

### Domain Models (Source of Truth for Schema)

**pt-project** (`crates/pt-project/src/`):
- `Project`: id (Uuid), scan_ref (Option<String>), zones (Vec<Zone>), tiers (Vec<Tier>), status (ProjectStatus), created_at/updated_at (DateTime<Utc>).
- `Zone`: id (ZoneId=Uuid), geometry (geo::Polygon<f64>), zone_type (ZoneType enum: Bed/Patio/Path/Lawn/Wall/Edging), label (Option<String>).
- `Tier`: level (TierLevel: Good/Better/Best), assignments (Vec<MaterialAssignment>).
- `MaterialAssignment`: zone_id (ZoneId), material_id (MaterialId), overrides (Option<AssignmentOverrides>).
- `AssignmentOverrides`: price_override (Option<Decimal>), depth_override_inches (Option<f64>).
- `ProjectStatus`: Draft/Quoted/Approved/Complete.
- Zones use GeoJSON serde via `crate::serde_helpers::geojson_polygon`.

**pt-materials** (`crates/pt-materials/src/types.rs`):
- `Material`: id (MaterialId=Uuid), name (String), category (MaterialCategory: Hardscape/Softscape/Edging/Fill), unit (Unit: SqFt/CuYd/LinearFt/Each), price_per_unit (Decimal), depth_inches (Option<f64>), texture_ref/photo_ref/supplier_sku (Option<String>), extrusion (ExtrusionBehavior).
- `ExtrusionBehavior`: tagged enum with `{"type":"sits_on_top","height_inches":1.5}` serde format — maps naturally to JSONB.

### Fields in Ticket Not in Domain Models
The ticket adds several fields that don't yet exist in the Rust domain types:
- **Tenant**: entirely new — id, name, logo_url, brand_color, contact (JSONB), timestamps. No Rust struct yet.
- **Project additions**: tenant_id, client_name, client_email, address, location (GEOGRAPHY POINT), baseline (JSONB). Current Rust `Project` has none of these — they'll be added when the repository layer maps rows to domain objects.
- **Zone additions**: sort_order. Not in current `Zone` struct.
- **Material additions**: tenant_id. Not in current `Material` struct.
- **Plants**: entirely new table. No Rust struct yet. Platform-level (no tenant FK).

### Downstream Consumer: T-003-02 (Repository Layer)
- Will use sqlx with compile-time query checking against this schema.
- Needs: `GeoJSON ↔ PostGIS geometry` conversion for zone polygons.
- CRUD for: projects (by tenant), zones (by project), materials (by tenant), tier_assignments (by project+tier).
- Integration tests against real Postgres — schema must be correct and complete.

### Database Tooling
- Target: Postgres 16 + PostGIS 3.4.
- `DATABASE_URL` env var for connection.
- `just db-reset` handles extension creation (`CREATE EXTENSION IF NOT EXISTS postgis`).
- No Docker Compose yet — local Postgres assumed.

### Conventions Observed
- All IDs are UUIDs with `gen_random_uuid()` as default.
- Timestamps use `TIMESTAMPTZ` with `now()` defaults.
- Enums stored as VARCHAR (consistent with serde `rename_all = "snake_case"`).
- JSONB for structured variant data (extrusion, overrides, contact, scan_ref, baseline).
- Foreign keys with `ON DELETE CASCADE` for project-scoped data (zones, tier_assignments).
- `DECIMAL(10,2)` for monetary values (matches Rust `Decimal` with 2dp rounding in pt-quote).

### Spatial Types
- `GEOMETRY(POLYGON, 4326)` for zones — 2D shapes on the WGS84 datum, supports spatial indexing and operations.
- `GEOGRAPHY(POINT, 4326)` for project location — geodetic distance calculations for "projects near me" queries.
- PostGIS GIST index on zone geometry for spatial queries.

### Down Migration Strategy
- `just db-reset` is the nuclear option (drops entire schema).
- Per-migration rollbacks need separate files or embedded `-- down` sections.
- Convention to decide: separate `*-down.sql` files vs. combined files.

## Key Constraints
1. PostGIS must be enabled before any spatial columns are created.
2. `just db-reset` already handles `CREATE EXTENSION IF NOT EXISTS postgis` before calling `db-migrate`.
3. Migration order matters: tenants before projects (FK), projects before zones (FK), zones before tier_assignments (FK).
4. The `UNIQUE(project_id, tier, zone_id)` constraint on tier_assignments enforces one material per zone per tier.
5. All enum values in the DB must match the serde `snake_case` serialization exactly.

## Open Questions
1. **Down migration file convention**: separate `001-down.sql` files or a different pattern? The justfile only has `db-migrate` (forward) and `db-reset` (nuclear). Ticket says "down migrations for rollback" — need a convention.
2. **Decimal precision**: `DECIMAL(10,2)` for price_per_unit matches 2dp currency, but pt-quote rounds quantities to 4dp. The overrides include `price_override (Decimal)` — same precision?
3. **Zone geometry SRID**: Ticket says 4326 (WGS84 degrees). Current domain model uses feet for computation. The repo layer will need to handle coordinate system context. Not a schema concern — just noting.
