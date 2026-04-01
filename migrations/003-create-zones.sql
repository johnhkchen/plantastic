-- Migration 003: Create zones table
-- Spatial polygons within a project. CASCADE delete when project is removed.

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
