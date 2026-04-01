-- Migration 005: Create tier_assignments table
-- Links a material to a zone within a specific tier. One material per zone per tier.

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
