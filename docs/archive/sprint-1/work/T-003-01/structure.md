# T-003-01 Structure: PostGIS Migrations

## Files Created

### Forward Migrations (12 files: 6 up + 6 down)

```
migrations/
├── 001-create-tenants.sql
├── 001-create-tenants.down.sql
├── 002-create-projects.sql
├── 002-create-projects.down.sql
├── 003-create-zones.sql
├── 003-create-zones.down.sql
├── 004-create-materials.sql
├── 004-create-materials.down.sql
├── 005-create-tier-assignments.sql
├── 005-create-tier-assignments.down.sql
├── 006-create-plants.sql
└── 006-create-plants.down.sql
```

### Files Modified

```
justfile  — update db-migrate to exclude *.down.sql files
```

## Table Schemas

### 001-create-tenants.sql

```sql
CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE tenants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    logo_url    VARCHAR(2048),
    brand_color VARCHAR(7),          -- hex: #RRGGBB
    contact     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Down: `DROP TABLE IF EXISTS tenants CASCADE;`

### 002-create-projects.sql

```sql
CREATE TABLE projects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id),
    client_name VARCHAR(255),
    client_email VARCHAR(255),
    address     VARCHAR(500),
    location    GEOGRAPHY(POINT, 4326),
    scan_ref    JSONB,
    baseline    JSONB,
    status      VARCHAR(20) NOT NULL DEFAULT 'draft'
                CHECK (status IN ('draft', 'quoted', 'approved', 'complete')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_projects_tenant_id ON projects(tenant_id);
```

Down: `DROP TABLE IF EXISTS projects CASCADE;`

### 003-create-zones.sql

```sql
CREATE TABLE zones (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    geometry    GEOMETRY(POLYGON, 4326) NOT NULL,
    zone_type   VARCHAR(20) NOT NULL
                CHECK (zone_type IN ('bed', 'patio', 'path', 'lawn', 'wall', 'edging')),
    label       VARCHAR(255),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_zones_project_id ON zones(project_id);
CREATE INDEX idx_zones_geometry ON zones USING GIST (geometry);
```

Down: `DROP TABLE IF EXISTS zones CASCADE;`

### 004-create-materials.sql

```sql
CREATE TABLE materials (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    name            VARCHAR(255) NOT NULL,
    category        VARCHAR(20) NOT NULL
                    CHECK (category IN ('hardscape', 'softscape', 'edging', 'fill')),
    unit            VARCHAR(20) NOT NULL
                    CHECK (unit IN ('sq_ft', 'cu_yd', 'linear_ft', 'each')),
    price_per_unit  DECIMAL(12,4) NOT NULL,
    depth_inches    NUMERIC,
    extrusion       JSONB NOT NULL,
    texture_key     VARCHAR(512),
    photo_key       VARCHAR(512),
    supplier_sku    VARCHAR(100),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_materials_tenant_id ON materials(tenant_id);
```

Down: `DROP TABLE IF EXISTS materials CASCADE;`

### 005-create-tier-assignments.sql

```sql
CREATE TABLE tier_assignments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tier        VARCHAR(10) NOT NULL
                CHECK (tier IN ('good', 'better', 'best')),
    zone_id     UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    material_id UUID NOT NULL REFERENCES materials(id),
    overrides   JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_tier_assignment UNIQUE (project_id, tier, zone_id)
);

CREATE INDEX idx_tier_assignments_project_tier ON tier_assignments(project_id, tier);
```

Down: `DROP TABLE IF EXISTS tier_assignments CASCADE;`

### 006-create-plants.sql

```sql
CREATE TABLE plants (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    common_name     VARCHAR(255) NOT NULL,
    botanical_name  VARCHAR(255) NOT NULL,
    sun_requirement VARCHAR(30) NOT NULL
                    CHECK (sun_requirement IN ('full_sun', 'partial_sun', 'partial_shade', 'full_shade')),
    water_need      VARCHAR(20) NOT NULL
                    CHECK (water_need IN ('low', 'moderate', 'high')),
    climate_zones   TEXT[] NOT NULL DEFAULT '{}',
    mature_height_ft NUMERIC,
    mature_width_ft  NUMERIC,
    tags            TEXT[] NOT NULL DEFAULT '{}',
    photo_url       VARCHAR(2048),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_plants_botanical_name ON plants(botanical_name);
```

Down: `DROP TABLE IF EXISTS plants CASCADE;`

## Justfile Change

Update `db-migrate` to exclude down files:

```diff
 db-migrate:
     @echo "Applying migrations..."
-    @for f in $(ls migrations/*.sql | sort); do \
+    @for f in $(ls migrations/*.sql | grep -v '\.down\.sql' | sort); do \
         echo "  → $f"; \
         psql "$DATABASE_URL" -f "$f"; \
     done
     @echo "Migrations applied."
```

## Indexes Summary

| Table | Index | Type | Purpose |
|-------|-------|------|---------|
| projects | idx_projects_tenant_id | btree | List projects by tenant |
| zones | idx_zones_project_id | btree | List zones by project |
| zones | idx_zones_geometry | GIST | Spatial queries |
| materials | idx_materials_tenant_id | btree | List materials by tenant |
| tier_assignments | idx_tier_assignments_project_tier | btree | Fetch tier data for a project |
| plants | idx_plants_botanical_name | btree | Lookup by scientific name |

## Ordering Constraints

Migrations must apply in numeric order due to FK dependencies:
1. tenants (root — no FKs)
2. projects → tenants
3. zones → projects
4. materials → tenants
5. tier_assignments → projects, zones, materials
6. plants (standalone — no FKs, could be any position)
