# T-031-01 Design: pt-scene crate

## Decision 1: Triangulation — earcutr

**Chosen:** `earcutr` crate (Rust port of Mapbox earcut).

**Why:** We need polygon-constrained triangulation (fill a polygon with triangles), not point-cloud Delaunay. earcutr handles simple polygons with holes, is battle-tested (Mapbox lineage), pure Rust, no unsafe, fast for the polygon sizes we'll see (< 100 vertices). The alternative — manual ear-clipping — would be reimplementing what earcutr already does, without hole support.

**Rejected:** `delaunator` (used by pt-scan) triangulates point clouds, not polygons. We'd get triangles outside the polygon boundary. `gltf-json` includes no triangulation.

## Decision 2: glTF construction — hand-rolled JSON (like pt-scan)

**Chosen:** Build the glTF JSON with `serde_json::json!()` and assemble binary buffer manually, same pattern as `pt-scan/export.rs`.

**Why:** The ticket says "solid colors per category is fine for ★☆☆☆☆." We need: one node per zone (named by label), one material per category (base color), positions + normals + indices per mesh. This is ~100 lines of JSON assembly. Adding `gltf-json` (with its complex builder API) is overhead for no gain at this stage.

**Rejected:** `gltf-json` crate — adds a dependency for a thin schema layer we don't need. Hand-rolled JSON follows the established pt-scan pattern, is easier to debug, and keeps the crate lightweight.

## Decision 3: Materials — solid base colors per category

**Chosen:** Map MaterialCategory to a solid RGBA base color in glTF PBR metallic-roughness.

| Category  | Base Color (sRGB)     | Rationale |
|-----------|----------------------|-----------|
| Hardscape | (180, 180, 180, 255) | Gray stone |
| Softscape | (139, 90, 43, 255)   | Brown earth |
| Edging    | (100, 100, 100, 255) | Dark gray |
| Fill      | (210, 180, 140, 255) | Tan sand |

PBR settings: metallicFactor=0.0, roughnessFactor=0.8 (matte).
Future PBR textures (texture_ref) can be added in a follow-up ticket.

## Decision 4: Buffer layout — one shared buffer, multiple meshes

**Chosen:** Single binary buffer containing all zone mesh data. Each zone gets its own mesh (with its own primitive) referencing slices of the shared buffer via bufferViews. One scene node per zone, named by zone label (or zone_id if no label).

**Why:** glTF spec supports one buffer with multiple accessors/views pointing to different offsets. This produces a single contiguous .glb file. The viewer spawns all nodes as children — each named node is independently pickable.

Per-zone node structure:
```
Scene
├── Node "Back patio" → Mesh 0 (material: Hardscape gray)
├── Node "Front bed" → Mesh 1 (material: Softscape brown)
└── Node "Edging" → Mesh 2 (material: Edging dark gray)
```

## Decision 5: Extrusion geometry

For each zone polygon:
1. **Top face:** Triangulate the polygon (earcutr), lift vertices to extrusion height
2. **Bottom face:** Same triangulation at y=0 (reversed winding for correct normals)
3. **Side walls:** For each edge of the polygon exterior ring, emit a quad (2 triangles) connecting top and bottom edges

Extrusion heights (converted from inches to feet):
- **SitsOnTop{height_inches}:** top at +h, bottom at 0. (h = height_inches / 12.0)
- **BuildsUp{height_inches}:** same as SitsOnTop (extrude upward)
- **Fills{flush}:** if flush, top at 0 and bottom at -DEFAULT_FILL_DEPTH; this is a visual-only fill

Default fill depth: 4 inches (0.333 ft) — just enough to see in 3D.

Coordinate mapping: polygon is XY in feet (pt-geo convention). In glTF, we use X=polygon_x, Z=polygon_y (ground plane is XZ), Y=height. This matches typical 3D conventions (Y-up).

## Decision 6: Zone naming for viewer picking

Each glTF node is named with the zone's label if present, otherwise the zone ID string. The viewer's picking plugin reads the `Name` component which comes from the glTF node name. The `zoneTapped` message sends this name as `zoneId`.

Convention: `zone.label.clone().unwrap_or_else(|| zone.id.to_string())`

## Public API

```rust
pub fn generate_scene(
    zones: &[Zone],
    assignments: &[MaterialAssignment],
    materials: &[Material],
    tier: TierLevel,
) -> Result<SceneOutput, SceneError>
```

- `SceneOutput { glb_bytes: Vec<u8>, metadata: SceneMetadata }`
- `SceneMetadata { zone_count, triangle_count, tier }`
- `SceneError` via thiserror: `EmptyZone`, `MissingMaterial`, `Triangulation`, `Export`

The function filters assignments by the given tier (caller passes the tier's assignments), looks up each assigned material, generates geometry per zone, and assembles the glTF.
