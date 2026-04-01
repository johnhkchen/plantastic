---
id: T-031-01
story: S-031
title: pt-scene-crate
type: task
status: open
priority: high
phase: ready
depends_on: []
---

## Context

The Bevy viewer works but has no real content to show. pt-scene generates glTF scenes from project zones + material assignments — the missing link between project data and 3D visualization. Scene export also serves as an early system integration check: if you can render a project as a 3D scene, the zone→material→visual pipeline is proven.

## Acceptance Criteria

- Create `crates/pt-scene/` crate
- `generate_scene(zones, assignments, materials, tier) -> Result<SceneOutput, SceneError>`
- SceneOutput: `glb_bytes: Vec<u8>`, `metadata: SceneMetadata`
- Zone polygons → extruded 3D meshes:
  - SitsOnTop: extrude upward by height_inches (converted to scene units)
  - Fills: extrude downward (sunken bed)
  - BuildsUp: extrude upward (edging wall)
- Material → glTF material:
  - Category → base color (Hardscape=gray, Softscape=brown, Edging=dark gray, Fill=tan)
  - texture_ref → PBR texture (if available, otherwise solid color)
- Each zone mesh named by zone label (matches viewer's tap-to-inspect: zoneTapped → zoneId)
- GLB output: valid glTF 2.0 binary (magic bytes 0x46546C67)
- Unit tests:
  - Single zone → valid GLB with correct mesh count
  - Multiple zones → all meshes present with distinct names
  - Empty zones → empty scene (no crash)
  - Extrusion height matches material spec
- `just check` passes

## Implementation Notes

- Use `gltf-json` crate for building the glTF document programmatically
- Polygon → mesh: triangulate the polygon (2D Delaunay, same approach as pt-scan), then extrude walls
- Scene coordinates: 1 unit = 1 foot (matches pt-geo)
- Zone label → mesh name → viewer tap identification — this naming convention is critical
- Don't over-engineer materials: solid colors per category is fine for ★☆☆☆☆, PBR textures come later
