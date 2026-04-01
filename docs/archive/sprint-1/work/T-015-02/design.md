# T-015-02 Design: Mesh Generation & Export

## Decision 1: Delaunay Triangulation Library

### Option A: `spade 3`
- Incremental Delaunay, handles degeneracies via exact predicates
- Rich API: constrained edges, natural neighbors, refinement
- Heavier dependency, slower for batch construction
- ~O(n log n) with good constant factors

### Option B: `delaunator 0.5`
- Port of Mapbox's delaunator-rs. Batch-only (not incremental).
- Very fast: ~2x faster than spade for one-shot construction
- Minimal API: takes flat &[f64] coords, returns triangle/halfedge arrays
- Well-tested, widely used. No features we don't need.

### Decision: `delaunator`

Rationale: We do one-shot triangulation per scan — no incremental inserts, no constrained edges. `delaunator` is faster and simpler for this use case. The flat-array API maps directly to our `Vec<Point>` positions. The halfedge structure gives adjacency for free if needed later.

## Decision 2: Mesh Decimation

### Option A: `meshopt` (Rust bindings for meshoptimizer)
- Battle-tested C library, used in production by major game engines
- `simplify()`: topology-preserving QEM, `simplify_sloppy()`: fast approximation
- Takes position + index buffers, returns simplified indices
- Does NOT remap vertex attributes (colors) — we handle remapping
- Adds a native C dependency (built via cc crate)

### Option B: Hand-rolled vertex clustering
- Group vertices into spatial cells, merge each cell to centroid
- Very fast O(n), simple to implement
- Lower quality: poor triangle aspect ratios, terrain artifacts
- Color averaging is straightforward

### Option C: Hand-rolled QEM
- Implement Garland-Heckbert quadric error metrics from scratch
- Best quality, full control
- Significant implementation effort (~500 lines), bugs likely
- No real advantage over meshopt which implements the same algorithm

### Decision: `meshopt`

Rationale: meshopt is the industry standard for mesh simplification. The C dependency is acceptable (it's a single-file C library, compiles everywhere). The lack of attribute remapping is manageable: after simplification, we rebuild the vertex-to-color mapping using the original positions. Quality is much better than vertex clustering, and we avoid the bug risk of a hand-rolled QEM.

## Decision 3: glTF Export

### Option A: `gltf-json` crate
- Official Rust glTF types. Build JSON document with typed structs.
- Handles validation, index types, accessor component types.
- Verbose but safe — type system prevents malformed JSON.

### Option B: Manual GLB construction
- GLB format is simple: 12-byte header + JSON chunk + binary chunk
- Build JSON with `serde_json`, pack binary manually
- Full control, no extra dependency
- Risk of subtle format errors (padding, alignment, accessor types)

### Decision: Manual GLB construction with `serde_json`

Rationale: The GLB format is straightforward enough that adding `gltf-json` doesn't save much complexity. We already have `serde_json` in the workspace. Our glTF output is simple: one mesh, one node, one scene, no animations or textures. Manual construction gives us full control over the binary layout and avoids a dependency that's overkill for our needs. We'll define lightweight serde structs for the JSON chunk.

## Decision 4: PNG Plan View Generation

### Option A: `image` crate
- Standard Rust image library. `ImageBuffer<Rgb<u8>>`, PNG encoder built in.
- Well-maintained, no unsafe, reasonable binary size.
- We'd write a simple triangle rasterizer on top.

### Option B: `tiny-skia` (2D rendering library)
- Full 2D rendering: paths, fills, anti-aliasing
- Overkill for our needs (just filling triangles with solid colors)
- Larger dependency

### Decision: `image` crate

Rationale: We need pixel-level control (elevation coloring, canopy overlay) and PNG output. The `image` crate does exactly this with minimal overhead. We'll write a scanline triangle rasterizer (~50 lines) that maps mesh triangles onto pixels with elevation-based coloring.

## Decision 5: Module Organization

### Option A: New crate `pt-mesh`
- Separate crate for mesh processing and export
- Clean dependency boundary
- But pt-scan already owns the scan pipeline; splitting adds indirection

### Option B: New modules inside `pt-scan`
- `mesh.rs`: Delaunay triangulation + decimation → `TerrainMesh`
- `export.rs`: glTF and PNG export functions
- Keep the pipeline in one crate — scan → mesh → export is one flow
- Types stay together, tests share helpers

### Decision: New modules inside `pt-scan`

Rationale: The scan processing pipeline is a single logical unit: PLY → points → mesh → artifacts. Splitting into a separate crate would force pt-scan types to be re-exported and add a dependency edge for no gain. The crate is still focused (scan processing) and the new modules are cohesive.

## Decision 6: Pipeline API

The current `process_scan()` returns `PointCloud`. We need to extend the pipeline to produce mesh + exports. Two options:

### Option A: Extend `process_scan()` to return mesh + artifacts
- Breaking change to the return type
- Couples mesh generation to the pipeline — can't skip it

### Option B: Separate function `generate_terrain(cloud) -> TerrainOutput`
- Takes `PointCloud` (output of `process_scan`)
- Returns `TerrainOutput { mesh_glb: Vec<u8>, plan_view_png: Vec<u8>, metadata: TerrainMetadata }`
- Non-breaking: `process_scan` stays the same
- Composable: callers choose whether to generate terrain artifacts

### Decision: Option B — `generate_terrain(cloud, config) -> TerrainOutput`

Rationale: Separation of concerns. `process_scan` does point cloud processing. `generate_terrain` does mesh construction and export. They compose naturally. The scenario test can call both in sequence. Future callers (API handler for scan upload) can choose to run one or both.

## Decision 7: Color Handling During Decimation

When meshopt simplifies the mesh, it collapses vertices. The surviving vertices keep their original positions (roughly), but we need to assign colors to the decimated mesh.

### Option A: Nearest-neighbor color lookup
- For each surviving vertex, find the nearest original vertex and copy its color
- Simple, fast, exact for vertices that weren't moved

### Option B: Barycentric interpolation
- Find which original triangle contains each surviving vertex
- Interpolate colors using barycentric coordinates
- More accurate for moved vertices, more complex

### Decision: Option A — Nearest-neighbor color lookup

Rationale: meshopt's `simplify()` keeps vertex positions close to originals (bounded by error threshold). Nearest-neighbor gives visually correct results for terrain coloring. The added complexity of barycentric interpolation isn't justified for ground surface colors that are already spatially smooth.

## Decision 8: S.1.1 Scenario Advancement

The ticket says "S.1.1 scenario registered and passing at ★☆☆☆☆". S.1.1 already passes at OneStar. T-015-02 adds mesh generation + export, which is still pure computation (no API integration). The star rating should remain OneStar but the test should verify the new mesh + export pipeline.

We update the S.1.1 test to also call `generate_terrain()` and validate the outputs (glb is valid, png is non-empty, metadata fields correct). The milestone note gets updated to reflect mesh generation delivery.

## Architecture Summary

```
PLY bytes ──→ process_scan() ──→ PointCloud ──→ generate_terrain() ──→ TerrainOutput
                 (existing)                          (new)             ├─ mesh_glb: Vec<u8>
                                                                      ├─ plan_view_png: Vec<u8>
                                                                      └─ metadata: TerrainMetadata
```

New modules:
- `mesh.rs`: `triangulate()`, `decimate()`, `compute_normals()`, `TerrainMesh` type
- `export.rs`: `to_glb()`, `to_plan_view_png()`, `TerrainOutput`, `TerrainMetadata`

New dependencies:
- `delaunator` (triangulation)
- `meshopt` (decimation)
- `image` (PNG output)
