# T-031-01 Research: pt-scene crate

## What exists

### Domain types (inputs to scene generation)

**Zone** (`crates/pt-project/src/types.rs`):
- `id: ZoneId(Uuid)`, `geometry: Polygon<f64>` (feet), `zone_type: ZoneType`, `label: Option<String>`
- ZoneType: Bed, Patio, Path, Lawn, Wall, Edging
- Polygon is `geo::Polygon<f64>`, serde via geojson_polygon helper

**Material** (`crates/pt-materials/src/types.rs`):
- `id: MaterialId(Uuid)`, `name`, `category: MaterialCategory`, `extrusion: ExtrusionBehavior`
- `texture_ref: Option<String>` for future PBR
- MaterialCategory: Hardscape, Softscape, Edging, Fill
- ExtrusionBehavior (tagged enum): SitsOnTop{height_inches}, Fills{flush}, BuildsUp{height_inches}

**MaterialAssignment** (`crates/pt-project/src/types.rs`):
- `zone_id: ZoneId`, `material_id: MaterialId`, `overrides: Option<AssignmentOverrides>`
- AssignmentOverrides: `price_override`, `depth_override_inches`

**Tier** (`crates/pt-project/src/types.rs`):
- `level: TierLevel` (Good/Better/Best), `assignments: Vec<MaterialAssignment>`

### Existing glTF generation (pt-scan)

`crates/pt-scan/src/export.rs` hand-rolls GLB:
- Constants: GLB_MAGIC=0x46546C67, GLB_VERSION=2, CHUNK_JSON=0x4E4F534A, CHUNK_BIN=0x004E4942
- Binary buffer layout: positions(f32×3) → normals(f32×3) → colors(RGBA u8×4) → indices(u32)
- JSON: asset, scenes, nodes, meshes, accessors, bufferViews, buffers
- Node naming: `"terrain"` (single mesh)
- No materials in glTF (uses vertex colors)

`crates/pt-scan/src/mesh.rs`:
- `triangulate(points: &[Point])` — delaunator 2D Delaunay, returns TerrainMesh
- `compute_normals(positions, indices)` — smooth per-vertex normals via face-normal accumulation
- Pattern: separate mesh generation from export

### Viewer expectations (apps/viewer/)

`apps/viewer/src/picking.rs`:
- PickingSetupPlugin reads `Name` component on entities
- `zoneTapped` message sends entity name as `zoneId`
- **Critical**: zone mesh names must match what the viewer expects (zone label or zone id)

`apps/viewer/src/scene.rs`:
- Loads GLB from URL via asset_server.load()
- Spawns glTF as scene bundle
- Supports tier swapping (loadScene + setTier messages)

`apps/viewer/src/bridge.rs`:
- Outbound: `{ "type": "zoneTapped", "zoneId": "..." }` — the zoneId comes from mesh Name

### Geometry module (pt-geo)

- `area::area_sqft(polygon)` — unsigned area via geo crate
- `perimeter::perimeter_ft(polygon)` — exterior ring length
- Re-exports: `geo::{Coord, LineString, Polygon, polygon}`
- All pure functions, no I/O
- Scene coordinates: 1 unit = 1 foot (matches pt-geo)

### Scenario harness

`tests/scenarios/src/suites/design.rs` S.2.4:
- Tests postMessage protocol contract (JSON shapes)
- Does NOT test scene generation (comments: "Not yet ThreeStar because real scene generation is not implemented")
- Milestone "pt-scene: 3D scene generation from project model" exists in progress.rs, `delivered_by: None`

### Triangulation approach

pt-scan uses `delaunator` crate for 2D Delaunay triangulation. For pt-scene, we need polygon triangulation (constrained to polygon boundary), not point cloud triangulation. Options:
- `geo` crate has no built-in triangulation
- `earcutr` crate — earcut algorithm, widely used for polygon triangulation, handles holes
- `delaunator` — Delaunay of point cloud (not polygon-constrained)
- Manual ear-clipping — simple but no hole support

### Dependencies available in workspace

Workspace Cargo.toml provides: geo, geojson, serde, serde_json, uuid, chrono, thiserror.
Not in workspace but needed: polygon triangulation crate, potentially `gltf-json`.

### Testing patterns

All crates use `pt_test_utils::timed()` for 10s timeout enforcement.
Dev-dependencies typically: `approx`, `pt-test-utils`.
Tests are `#[cfg(test)] mod tests` embedded in source files.

### Constraints

- Scene coordinates: 1 unit = 1 foot (pt-geo convention)
- Inches to feet conversion needed for extrusion heights (height_inches / 12.0)
- Zone label → mesh name — this is the tap-identification path in the viewer
- GLB must be valid glTF 2.0 (magic bytes 0x46546C67)
- Empty zones → empty scene (no crash)

## Key decisions to make in Design

1. Triangulation crate: earcutr vs manual ear-clipping
2. glTF construction: hand-rolled JSON (like pt-scan) vs gltf-json crate
3. Material representation: solid base colors only vs PBR stubs
4. Mesh structure: one buffer per zone vs shared buffer with multiple meshes
5. Wall extrusion: how to generate side faces for extruded polygons
