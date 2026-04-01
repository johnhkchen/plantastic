---
id: T-002-01
story: S-002
title: pt-project-and-pt-materials
type: task
status: open
priority: critical
phase: done
depends_on: [T-001-02]
---

## Context

These two crates define the types the entire system operates on. pt-project is the heart — Project, Zone, Tier, MaterialAssignment — serialized as GeoJSON. pt-materials defines the material catalog domain — what a landscaper sells, how it extrudes in 3D, what it costs. They're implemented together because they share boundary types (MaterialId, ZoneId) and are tested together.

## Acceptance Criteria

### pt-project
- Project struct: id, scan_ref, zones, tiers (3), status, timestamps
- Zone struct: id, geometry (geo::Polygon<f64>), zone_type enum (bed, patio, path, lawn, wall, edging), optional label
- Tier struct: name, material assignments vec
- MaterialAssignment struct: zone_id, material_id, optional overrides
- ProjectStatus enum: draft, quoted, approved, complete
- GeoJSON serialization round-trip (Project → GeoJSON → Project)
- Tests for serialization, status transitions, zone CRUD operations on a project

### pt-materials
- Material struct: id, name, category, unit, price_per_unit, depth_inches, texture_ref, photo_ref, supplier_sku, extrusion_behavior
- MaterialCategory enum: hardscape, softscape, edging, fill
- Unit enum: sq_ft, cu_yd, linear_ft, each
- ExtrusionBehavior enum: SitsOnTop { height }, Fills { flush }, BuildsUp { height }
- Serde serialization round-trip tests
- Builder or constructor pattern for creating materials with sensible defaults
