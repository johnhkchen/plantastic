---
id: T-024-01
story: S-024
title: material-callout-scenario
type: task
status: open
priority: high
phase: open
depends_on: [T-022-01]
---

## Context

S.4.3 (Material callouts with supplier info) is NotImplemented but both prereqs are delivered: pt-materials (T-012-01) and Bevy viewer (T-013-02). The scenario validates that zone-material assignments expose the callout data a crew foreman needs.

## Acceptance Criteria

- S.4.3 scenario test implemented in `tests/scenarios/src/suites/crew_handoff.rs`
- Test constructs:
  1. Materials with name, supplier_sku, depth_inches, photo_ref, extrusion
  2. Zones with known geometry
  3. Tier assignments mapping zones to materials
- Test verifies per-zone callout data:
  - Material name present
  - Supplier SKU present (e.g., "TRAV-12x12-NAT")
  - Install depth present and matches material's depth_inches
  - Photo ref present when material has one
  - Extrusion behavior present (tells crew how material is installed)
- Expected values computed independently in the test (not extracted from the system)
- Pass at ★☆☆☆☆ integration (computation) + ★☆☆☆☆ polish initially
- Path to ★★☆☆☆: verify callout data via GET /materials API response
- `just check` passes, no regressions

## Implementation Notes

- Use `Material::builder()` from pt-materials to construct test materials
- The callout is essentially: for each (zone, material) pair in a tier, the material's metadata fields are the callout
- This is a computation/data-model test — no new API routes or UI needed for OneStar
- Crew Handoff area goes from 0.0 to 1.0 effective minutes (5 min raw × 2/10 at int=1, pol=1)
