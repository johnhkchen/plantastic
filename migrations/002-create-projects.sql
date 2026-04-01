-- Migration 002: Create projects table
-- Each project belongs to a tenant and tracks a single landscaping job.

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
