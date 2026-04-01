# T-015-02 Structure: Mesh Generation & Export

## Files Modified

### `crates/pt-scan/Cargo.toml`
Add dependencies:
- `delaunator = "0.5"` — 2D Delaunay triangulation
- `meshopt = "0.4"` — mesh simplification (native C bindings)
- `image = { version = "0.25", default-features = false, features = ["png"] }` — PNG export
- `serde_json = { workspace = true }` — glTF JSON chunk construction

### `crates/pt-scan/src/lib.rs`
Add module declarations:
```rust
pub mod mesh;
pub mod export;
```
Add re-exports:
```rust
pub use mesh::{TerrainMesh, MeshConfig};
pub use export::{generate_terrain, TerrainOutput, TerrainMetadata, ExportConfig};
```

### `crates/pt-scan/src/error.rs`
Add error variants:
- `MeshGeneration(String)` — triangulation or decimation failure
- `ExportError(String)` — glTF or PNG export failure

## Files Created

### `crates/pt-scan/src/mesh.rs` (~200 lines)

Core mesh generation module.

**Public types:**
```rust
pub struct TerrainMesh {
    pub positions: Vec<[f32; 3]>,    // vertex positions
    pub normals: Vec<[f32; 3]>,      // per-vertex normals
    pub colors: Vec<[u8; 3]>,        // per-vertex RGB
    pub indices: Vec<u32>,           // triangle indices (len % 3 == 0)
}

pub struct MeshConfig {
    pub target_triangles: usize,     // default: 50_000
}
```

**Public functions:**
```rust
/// 2D Delaunay triangulation of ground points.
/// Projects onto XY, triangulates, uses original 3D positions.
pub fn triangulate(points: &[Point]) -> Result<TerrainMesh, ScanError>

/// Decimate mesh to target triangle count using meshopt.
/// Preserves vertex colors via nearest-neighbor remapping.
pub fn decimate(mesh: &TerrainMesh, target: usize) -> TerrainMesh

/// Compute smooth per-vertex normals from face normals.
fn compute_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]>
```

**Internal flow:**
1. `triangulate()`: extract XY from points → `delaunator::triangulate()` → build positions/indices/colors from result → `compute_normals()` → `TerrainMesh`
2. `decimate()`: call `meshopt::simplify()` with positions + indices → remap colors via nearest original vertex → recompute normals → `TerrainMesh`

### `crates/pt-scan/src/export.rs` (~250 lines)

Export module: glTF binary and PNG plan view.

**Public types:**
```rust
pub struct TerrainOutput {
    pub mesh_glb: Vec<u8>,           // binary glTF 2.0
    pub plan_view_png: Vec<u8>,      // top-down orthographic PNG
    pub metadata: TerrainMetadata,   // JSON-serializable metadata
}

pub struct TerrainMetadata {
    pub bbox: BoundingBox,
    pub elevation_range: [f32; 2],   // [min_z, max_z]
    pub original_point_count: usize,
    pub decimated_triangle_count: usize,
    pub vertex_count: usize,
    pub processing_time_ms: u64,
}

pub struct ExportConfig {
    pub mesh: MeshConfig,
    pub pixels_per_foot: f32,        // default: 10.0
    pub plan_view_width: Option<u32>,  // override auto-computed width
    pub canopy_overlay: bool,        // render obstacles as dark overlay
}
```

**Public functions:**
```rust
/// Full terrain generation pipeline: triangulate → decimate → export.
/// Takes PointCloud from process_scan() and produces all output artifacts.
pub fn generate_terrain(
    cloud: &PointCloud,
    config: &ExportConfig,
) -> Result<TerrainOutput, ScanError>
```

**Internal functions:**
```rust
/// Serialize TerrainMesh to binary glTF 2.0 (.glb) format.
fn to_glb(mesh: &TerrainMesh) -> Vec<u8>

/// Render top-down orthographic projection as PNG.
fn to_plan_view_png(
    mesh: &TerrainMesh,
    obstacles: &[Point],
    bbox: &BoundingBox,
    config: &ExportConfig,
) -> Vec<u8>

/// Rasterize a single triangle onto the image buffer.
fn rasterize_triangle(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    v0: [f32; 2], v1: [f32; 2], v2: [f32; 2],
    c0: [u8; 3], c1: [u8; 3], c2: [u8; 3],
)
```

**glTF structure:**
```
Scene 0
  └─ Node 0
       └─ Mesh 0
            └─ Primitive 0
                 ├─ POSITION  → accessor 0 → bufferView 0
                 ├─ NORMAL    → accessor 1 → bufferView 1
                 ├─ COLOR_0   → accessor 2 → bufferView 2
                 └─ indices   → accessor 3 → bufferView 3
```

Binary buffer layout:
```
[positions: N×12 bytes][normals: N×12 bytes][colors: N×3 bytes + padding][indices: M×4 bytes]
```

### `crates/pt-scan/tests/integration.rs` (extend)

Add new test functions:

```rust
#[test]
fn test_terrain_generation_pipeline()
// PLY → process_scan → generate_terrain → verify outputs

#[test]
fn test_glb_format_validity()
// Verify GLB header, chunk structure, JSON parseable

#[test]
fn test_plan_view_png_dimensions()
// Verify PNG dimensions match expected from bbox + pixels_per_foot

#[test]
fn test_metadata_consistency()
// Verify metadata fields match actual mesh properties
```

### `tests/scenarios/src/suites/site_assessment.rs` (modify)

Update `s_1_1_scan_processing()`:
- After existing PointCloud validation, call `generate_terrain()`
- Verify `mesh_glb` starts with glTF magic bytes
- Verify `plan_view_png` starts with PNG magic bytes
- Verify metadata triangle count ≤ target
- Keep at `OneStar` (still pure computation, no API)

### `tests/scenarios/src/progress.rs` (modify)

Update milestone note for "pt-scan: PLY parsing + mesh generation":
- Update `delivered_by` to `Some("T-015-02")` (latest deliverer)
- Expand note to cover mesh generation + export capabilities

## Module Boundaries

```
pt-scan/
├── src/
│   ├── lib.rs          # re-exports, process_scan()
│   ├── types.rs        # Point, PointCloud, BoundingBox, etc.
│   ├── error.rs        # ScanError (+ new variants)
│   ├── parser.rs       # PLY parsing
│   ├── filter.rs       # voxel downsample, outlier removal
│   ├── ransac.rs       # ground plane fitting
│   ├── mesh.rs         # NEW: triangulation + decimation
│   └── export.rs       # NEW: glTF + PNG + metadata export
└── tests/
    └── integration.rs  # existing + new tests
```

## Dependency Graph

```
export.rs ──→ mesh.rs ──→ types.rs
    │              │
    │              ├──→ delaunator (triangulation)
    │              └──→ meshopt (decimation)
    │
    ├──→ image (PNG)
    ├──→ serde_json (glTF JSON)
    └──→ types.rs (BoundingBox, Point)
```

## Interface Contract

The public API addition is one function + supporting types:
```rust
// Caller composes the two pipeline stages:
let cloud = pt_scan::process_scan(reader, &scan_config)?;
let output = pt_scan::generate_terrain(&cloud, &export_config)?;
// output.mesh_glb    → write to terrain.glb
// output.plan_view_png → write to planview.png
// output.metadata    → serialize to metadata.json
```

No changes to existing public API. `process_scan()` signature and return type unchanged.
