# T-031-01 Structure: pt-scene crate

## Files created

### `crates/pt-scene/Cargo.toml`
```
[package]
name = "pt-scene"
edition/license/rust-version from workspace

[dependencies]
pt-project, pt-materials, pt-geo (path deps)
earcutr (polygon triangulation)
serde, serde_json, thiserror (workspace)

[dev-dependencies]
approx, pt-test-utils (path dep)

[lints]
workspace = true
```

### `crates/pt-scene/src/lib.rs`
Module root. Re-exports public API:
- `pub mod error;`
- `pub mod mesh;`
- `pub mod glb;`
- `pub mod scene;` (orchestrator)
- Top-level re-export: `pub use scene::generate_scene;`
- Re-export types: `pub use scene::{SceneOutput, SceneMetadata};`
- Re-export error: `pub use error::SceneError;`

### `crates/pt-scene/src/error.rs`
```rust
pub enum SceneError {
    MissingMaterial { zone_id, material_id },
    Triangulation(String),
    Export(String),
}
```
Implements Display, std::error::Error via thiserror.

### `crates/pt-scene/src/mesh.rs`
Zone polygon → extruded triangle mesh. Internal module.

Types:
```rust
pub(crate) struct ZoneMesh {
    pub positions: Vec<[f32; 3]>,   // Y-up: x=poly_x, z=poly_y, y=height
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}
```

Functions:
```rust
pub(crate) fn extrude_zone(
    polygon: &Polygon<f64>,
    extrusion: &ExtrusionBehavior,
) -> Result<ZoneMesh, SceneError>
```

Internal helpers:
- `triangulate_polygon(polygon) -> (Vec<[f32;3]>, Vec<u32>)` — earcutr on exterior ring
- `extrude_walls(ring_coords, bottom_y, top_y) -> (Vec<[f32;3]>, Vec<[f32;3]>, Vec<u32>)` — side quads
- `compute_normals(positions, indices) -> Vec<[f32;3]>` — smooth normals (same algorithm as pt-scan)

### `crates/pt-scene/src/glb.rs`
GLB binary assembly. Internal module.

Types:
```rust
pub(crate) struct GlbMesh {
    pub name: String,
    pub mesh: ZoneMesh,
    pub base_color: [f32; 4],  // sRGB linear
}
```

Functions:
```rust
pub(crate) fn to_glb(meshes: &[GlbMesh]) -> Result<Vec<u8>, SceneError>
```

Builds:
- One glTF material per unique base_color (deduplicates)
- One mesh primitive per zone (positions + normals + indices)
- One node per zone (named, referencing its mesh)
- One scene containing all nodes
- Binary buffer: all zone data concatenated, bufferViews with offsets

Constants (same as pt-scan): GLB_MAGIC, GLB_VERSION, CHUNK_JSON, CHUNK_BIN.

### `crates/pt-scene/src/scene.rs`
Orchestrator — the public API.

Types:
```rust
pub struct SceneOutput {
    pub glb_bytes: Vec<u8>,
    pub metadata: SceneMetadata,
}

pub struct SceneMetadata {
    pub zone_count: usize,
    pub triangle_count: usize,
    pub tier: TierLevel,
}
```

Function:
```rust
pub fn generate_scene(
    zones: &[Zone],
    assignments: &[MaterialAssignment],
    materials: &[Material],
    tier: TierLevel,
) -> Result<SceneOutput, SceneError>
```

Logic:
1. If zones is empty, return empty GLB (valid glTF with no meshes)
2. For each assignment, find the zone and material
3. Determine extrusion from material
4. Call `mesh::extrude_zone()` for geometry
5. Map material category → base color
6. Collect into `Vec<GlbMesh>`
7. Call `glb::to_glb()` for binary
8. Return SceneOutput with metadata

## Files modified

### `tests/scenarios/src/progress.rs`
Update the "pt-scene" milestone: set `delivered_by: Some("T-031-01")` and write the note.

## Module dependency graph

```
scene.rs (public API)
├── mesh.rs (polygon → extruded triangles)
├── glb.rs  (meshes → GLB binary)
└── error.rs (shared error type)
```

No circular dependencies. mesh.rs and glb.rs are independent; scene.rs orchestrates both.

## Interface boundaries

- **Input boundary:** `Zone`, `MaterialAssignment`, `Material`, `TierLevel` from pt-project/pt-materials
- **Output boundary:** `SceneOutput` (owned bytes + metadata)
- **No I/O:** pure computation, no filesystem, no network
- **Thread-safe:** all functions take shared references, return owned data
