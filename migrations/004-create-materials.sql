-- Migration 004: Create materials table
-- Tenant-scoped material catalog with pricing and 3D extrusion data.

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
