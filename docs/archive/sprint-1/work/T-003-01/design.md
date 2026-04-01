# T-003-01 Design: PostGIS Migrations

## Decision 1: Migration File Convention

### Options
**A) Numbered up/down pairs**: `001-up-create-tenants.sql`, `001-down-create-tenants.sql`
**B) Numbered forward-only with separate down dir**: `migrations/001-create-tenants.sql`, `migrations/down/001-drop-tenants.sql`
**C) Numbered with descriptive names, down as suffix**: `001-create-tenants.sql`, `001-create-tenants.down.sql`

### Decision: Option C — `.down.sql` suffix
- `just db-migrate` uses `ls migrations/*.sql | sort` — `.down.sql` files sort after `.sql` but are visually paired.
- We need to update `db-migrate` to exclude `*.down.sql` files: `ls migrations/*.sql | grep -v '.down.sql' | sort`.
- Clean: each migration and its rollback are adjacent in file listings.
- Easy to find: the down migration is always `{name}.down.sql`.

## Decision 2: PostGIS Extension Placement

### Options
**A) Migration 000 creates the extension**: Explicit, self-documenting, works with any runner.
**B) Rely on `just db-reset`**: Current behavior — `db-reset` creates the extension.

### Decision: Option A — Migration 000 for PostGIS
- `CREATE EXTENSION IF NOT EXISTS postgis` is idempotent.
- Migration files should be self-contained — applying 001-006 on a fresh `public` schema must work without `db-reset`.
- `db-reset` already does it too — no conflict, just belt-and-suspenders.

Wait — the ticket specifies exactly 6 migrations (001-006) with defined contents. Adding 000 is outside scope. PostGIS extension creation stays in `db-reset`. The first migration (001) will include `CREATE EXTENSION IF NOT EXISTS postgis` at the top as a safety net, since it's idempotent.

## Decision 3: Enum Storage

### Options
**A) PostgreSQL ENUM types**: Strong typing, but hard to alter (adding values requires `ALTER TYPE ... ADD VALUE`, can't remove).
**B) VARCHAR with CHECK constraints**: Flexible, easy to alter, enforces valid values.
**C) Plain VARCHAR**: No enforcement at DB level, rely on application.

### Decision: Option B — VARCHAR with CHECK
- Matches serde `snake_case` strings exactly (no translation layer).
- CHECK constraints provide DB-level validation without the rigidity of PG enums.
- Easy to add new values: `ALTER TABLE ... DROP CONSTRAINT ...; ALTER TABLE ... ADD CONSTRAINT ...`.
- Works well with sqlx — maps directly to/from Rust string representations.
- Applied to: `zone_type`, `status`, `tier`, `category`, `unit`.

## Decision 4: Decimal Precision

### Decision: `DECIMAL(12,4)` for prices, `DECIMAL(12,4)` for overrides
- pt-quote rounds line_total to 2dp and quantities to 4dp.
- Storing prices at 4dp gives headroom for per-unit pricing that might not be round cents (e.g., $0.0825/unit).
- `DECIMAL(12,4)` supports values up to 99,999,999.9999 — more than sufficient for landscaping.
- Consistent precision across price_per_unit and price_override avoids confusion.
- `depth_inches` stays as `NUMERIC` (no specific precision — fractional inches).

## Decision 5: Down Migration Strategy

### Decision: DROP TABLE with CASCADE
- Each down migration drops its table with `CASCADE` to clean up dependent objects.
- Down migrations run in reverse order (006-down first, 001-down last).
- For the `CREATE EXTENSION` in 001, the down migration does NOT drop PostGIS (other schemas might use it).

## Decision 6: Timestamp Pattern

### Decision: `TIMESTAMPTZ NOT NULL DEFAULT now()`
- All tables get `created_at` and `updated_at` columns.
- `updated_at` is set by the application on writes (not a trigger — keeps it simple, matches Rust domain behavior).
- No DB-level trigger for updated_at — the repository layer handles it. Triggers add hidden behavior that's hard to debug.

## Decision 7: UUID Primary Keys

### Decision: `UUID PRIMARY KEY DEFAULT gen_random_uuid()`
- Matches the Rust `Uuid::new_v4()` pattern.
- `gen_random_uuid()` is built into Postgres 13+ (no extension needed).
- Application can supply IDs (Rust generates them) or let the DB generate defaults.

## Decision 8: JSONB Fields

Fields stored as JSONB:
- `tenants.contact` — structured contact info, shape may evolve.
- `projects.scan_ref` — reference to scan artifacts, structure TBD.
- `projects.baseline` — satellite pre-population data, structure TBD.
- `materials.extrusion` — tagged union matching `ExtrusionBehavior` serde output.
- `tier_assignments.overrides` — matches `AssignmentOverrides` serde output.

All nullable. The JSONB format matches the Rust serde output directly — sqlx can serialize/deserialize these with `serde_json::Value` or typed structs.

## Decision 9: Plants Table Columns

The ticket says "sun/climate/size fields" without specifying exact columns. Based on the specification (plant intelligence section) and the solar-sim prototype:
- `sun_requirement VARCHAR NOT NULL` — full_sun / partial_shade / full_shade / partial_sun
- `water_need VARCHAR NOT NULL` — low / moderate / high
- `climate_zones TEXT[] NOT NULL DEFAULT '{}'` — array of Sunset zones (e.g., {"14", "15", "16"})
- `mature_height_ft NUMERIC` — expected mature height
- `mature_width_ft NUMERIC` — expected mature spread
- `tags TEXT[] NOT NULL DEFAULT '{}'` — flexible categorization (native, drought-tolerant, etc.)
- `photo_url VARCHAR` — nullable

## Schema Overview

```
tenants (001)
  └── projects (002) [tenant_id FK]
        ├── zones (003) [project_id FK CASCADE]
        │     └── tier_assignments (005) [zone_id FK CASCADE]
        └── tier_assignments (005) [project_id FK CASCADE]
  └── materials (004) [tenant_id FK]
        └── tier_assignments (005) [material_id FK]

plants (006) — standalone, no FK relationships
```

## What Was Rejected
- **PostgreSQL native ENUMs**: Too rigid for a fast-moving schema. Adding a new ZoneType or MaterialCategory would require careful migration ordering.
- **Database triggers for updated_at**: Adds hidden behavior. The Rust domain model already manages timestamps — let the repo layer set them explicitly.
- **Separate migration framework**: The `psql`-based approach in the justfile is simple and works. No need for sqlx-migrate or refinery when we have 6 migrations and `db-reset` as the nuclear option.
- **Integer/serial primary keys**: UUIDs are already the convention in all domain types. No benefit to introducing a second ID scheme.
