-- Migration 001: Create tenants table
-- Multi-tenancy root — every project and material is scoped to a tenant.

CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE tenants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    logo_url    VARCHAR(2048),
    brand_color VARCHAR(7),
    contact     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
