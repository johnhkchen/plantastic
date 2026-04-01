# T-015-02 Research: Mesh Generation & Export

## What Exists

### pt-scan crate (delivered by T-015-01)

**Location**: `crates/pt-scan/`

The crate provides the full PLY-to-PointCloud pipeline:

- **`parser.rs`**: `parse_ply(reader) -> Result<Vec<Point>>` — reads binary LE/BE and ASCII PLY via `ply-rs-bw`. Extracts x/y/z (f32) and optional RGB (u8).
- **`filter.rs`**: `voxel_downsample(points, voxel_size)` reduces density via HashMap cell averaging. `remove_outliers(points, k, threshold)` uses `kiddo` ImmutableKdTree for k-NN statistical filtering.
- **`ransac.rs`**: `fit_ground_plane(points, iterations, threshold)` — RANSAC with nalgebra cross products. Returns `GroundClassification` with ground/obstacle indices and `Plane`.
- **`lib.rs`**: `process_scan(reader, config) -> Result<PointCloud>` orchestrates parse → downsample → outlier removal → RANSAC.
- **`types.rs`**: `Point` (position [f32;3], color Option<[u8;3]>), `PointCloud` (ground Vec, obstacles Vec, metadata), `ScanMetadata` (bbox, counts, ground_plane), `BoundingBox`, `Plane`, `ScanConfig`.
- **`error.rs`**: `ScanError` enum — `InvalidPly`, `InsufficientPoints`, `NoGroundPlane`, `Io`.

**Dependencies**: indexmap, kiddo, nalgebra 0.34, ply-rs-bw, rand, serde, thiserror.
**Tests**: 15 unit + 3 integration, all passing.

### Key types for T-015-02 input

```rust
pub struct PointCloud {
    pub ground: Vec<Point>,       // classified ground points
    pub obstacles: Vec<Point>,    // above-ground points
    pub metadata: ScanMetadata,   // bbox, counts, ground_plane
}

pub struct Point {
    pub position: [f32; 3],       // meters
    pub color: Option<[u8; 3]>,   // RGB
}

pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
```

### Bevy viewer (T-013-01)

**Location**: `apps/viewer/` (standalone, excluded from workspace)

- Bevy 0.18, loads `.glb` via `AssetServer::load("models/test_scene.glb")`
- Spawns `SceneRoot` from loaded glTF
- Uses `bevy_panorbit_camera` for orbit controls
- Directional light + ambient fill
- Build target: WASM (Trunk)

**Format expectation**: Binary glTF 2.0 (.glb). Bevy's native gltf loader handles meshes, PBR materials, scenes. Vertex colors supported via `COLOR_0` accessor.

### Scenario harness

**Location**: `tests/scenarios/`

- S.1.1 currently passes at `OneStar` (pure computation — PLY → PointCloud classification)
- `progress.rs` milestone "pt-scan: PLY parsing + mesh generation" is delivered by T-015-01, notes T-015-02 will add mesh gen + export
- `site_assessment.rs` has `s_1_1_scan_processing()` test function
- ScenarioOutcome levels: NotImplemented, OneStar (pure computation), TwoStar (API-reachable), etc.

### Workspace dependencies

Root `Cargo.toml` has workspace deps including: `serde`, `serde_json`, `thiserror`. No mesh/glTF/image crates currently in workspace deps.

## What T-015-02 Must Produce

Per the ticket acceptance criteria:

1. **Delaunay triangulation** of ground points → triangle mesh
2. **Mesh decimation** to configurable target (default ~50k triangles)
3. **Vertex color preservation** through triangulation
4. **glTF binary (.glb) export** of terrain mesh
5. **Top-down orthographic PNG** (plan view) with configurable resolution
6. **Metadata JSON**: bbox, elevation range, point counts, triangle count, processing time
7. **End-to-end test**: PLY → (terrain.glb, planview.png, metadata.json)
8. **Bevy cross-validation**: generated glTF loads in viewer
9. **S.1.1 scenario** registered and passing at ★☆☆☆☆
10. **Milestone claim**: update "pt-scan: PLY parsing + mesh generation"

## Technical Constraints

### Triangulation is 2.5D

Ground points lie on a near-planar surface (z ≈ 0 after RANSAC). The natural approach: project points onto XY plane, compute 2D Delaunay triangulation, then use original 3D positions for the mesh vertices. This avoids degenerate tetrahedralization.

### Point count ranges

- Raw PLY: ~5M points (iPhone LiDAR)
- After voxel downsample (2cm): ~500K points
- After outlier removal + RANSAC: ground typically ~300-400K, obstacles ~50-100K
- Target mesh: ~50K triangles (for iPad rendering)

So triangulation on ~300-400K points → decimation to ~50K triangles.

### glTF binary format

GLB consists of:
- 12-byte header: magic(0x46546C67) + version(2) + total_length
- JSON chunk: chunk_length + chunk_type(0x4E4F534A) + JSON bytes (padded to 4-byte alignment with spaces)
- Binary chunk: chunk_length + chunk_type(0x004E4942) + binary data (padded with 0x00)

JSON chunk defines: asset, buffers, bufferViews, accessors, meshes, nodes, scenes. Binary chunk contains vertex positions, normals, indices, and vertex colors as interleaved or separate buffer views.

### PNG plan view

Top-down orthographic projection: map each triangle's XY extent onto image pixels, rasterize with Z-based or vertex-color-based coloring. The `image` crate (standard Rust image library) can write PNG.

Configurable resolution in pixels per foot. Typical yard: 30×50 ft → at 10 px/ft = 300×500 px.

### Mesh decimation approaches

- **Vertex clustering**: fast O(n), simple, lower quality — group vertices into cells, collapse
- **Quadric Error Metrics (QEM)**: O(n log n), better quality — Garland & Heckbert 1997
- **meshopt**: C library with Rust bindings, battle-tested, includes `simplify_sloppy` for fast decimation

## Crate candidates (Rust ecosystem)

| Need | Crate | Notes |
|------|-------|-------|
| 2D Delaunay | `spade 3` | Mature, incremental Delaunay, handles degeneracies. ~O(n log n). |
| 2D Delaunay | `delaunator 0.5` | Port of Mapbox's delaunator. Very fast, simpler API, no incremental. |
| Mesh decimation | `meshopt 0.4` | Rust bindings for meshoptimizer. `simplify` + `simplify_sloppy`. |
| glTF export | `gltf-json 1` | Low-level JSON types for glTF 2.0. Build JSON manually, pack binary. |
| glTF export | manual GLB | GLB format is simple enough (header + JSON chunk + binary chunk). |
| PNG generation | `image 0.25` | Standard Rust image library. `ImageBuffer<Rgb<u8>>`, write PNG. |

## Existing patterns to follow

- Pure computation, no I/O: pt-scan functions take slices/refs and return owned data
- All types derive `Serialize, Deserialize` for JSON round-trip
- Error types via `thiserror` with domain-specific variants
- Dev-dependencies: `approx`, `pt-test-utils` for `timed()` wrapper
- Tests compute expected values independently (rule 2)
- No mocking across crate boundaries (rule 3)

## Risks and open questions

1. **Delaunay on 300K+ points**: `spade` and `delaunator` both handle this but performance needs validation. `delaunator` is typically 2-3x faster for batch construction.
2. **Decimation quality**: `meshopt::simplify` preserves topology well but requires position/index buffers. Needs vertex attribute remapping for colors.
3. **Color interpolation during decimation**: When vertices are collapsed, their colors must be blended. `meshopt` doesn't handle vertex attributes beyond position — colors need manual remapping after decimation.
4. **Normal computation**: glTF meshes should have normals. Compute per-face normals from cross products, then average at vertices (smooth shading on terrain).
5. **PNG rasterization**: Need a simple triangle rasterizer for the plan view. The `image` crate handles pixel writing; triangle fill needs barycentric interpolation or scanline approach.
6. **Coordinate system**: PLY uses +Y up (SiteScape convention) vs glTF uses +Y up. May need axis swap — verify with Bevy viewer.
