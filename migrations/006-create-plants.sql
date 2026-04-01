-- Migration 006: Create plants table
-- Platform-level plant database. No tenant FK — shared across all tenants.

CREATE TABLE plants (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    common_name      VARCHAR(255) NOT NULL,
    botanical_name   VARCHAR(255) NOT NULL,
    sun_requirement  VARCHAR(30) NOT NULL
                     CHECK (sun_requirement IN ('full_sun', 'partial_sun', 'partial_shade', 'full_shade')),
    water_need       VARCHAR(20) NOT NULL
                     CHECK (water_need IN ('low', 'moderate', 'high')),
    climate_zones    TEXT[] NOT NULL DEFAULT '{}',
    mature_height_ft NUMERIC,
    mature_width_ft  NUMERIC,
    tags             TEXT[] NOT NULL DEFAULT '{}',
    photo_url        VARCHAR(2048),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_plants_botanical_name ON plants(botanical_name);
