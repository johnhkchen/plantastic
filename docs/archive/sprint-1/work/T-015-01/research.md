# T-015-01 Research: PLY Parsing & Point Cloud Filtering

## Ticket Summary

Create `pt-scan` crate in `crates/pt-scan/`. Parse PLY files (binary + ASCII), filter
point clouds (outlier removal, voxel downsampling), fit ground planes (RANSAC), and
separate ground from obstacle points. Performance target: 5M points < 10 seconds.

## Codebase Context

### Workspace Structure

Monorepo with `crates/*` as workspace members. All crates share edition 2021, MIT license,
rust-version 1.75, and workspace-level clippy lints (deny correctness/suspicious, deny
unsafe_code). Each crate has a `[lints] workspace = true` section.

Existing crates follow a pattern: `lib.rs` re-exports public types, internal modules stay
`pub(crate)`. Dev dependencies include `pt-test-utils` for `timed()` / `run_with_timeout()`.

### Relevant Existing Crates

- **pt-geo**: geometry math (area, perimeter, volume). Uses `geo` crate. Pure computation.
  pt-scan does NOT depend on this — different coordinate systems (survey/LiDAR vs geographic).
- **pt-project**: domain model. Has `ScanRef` concept (pointer to PLY + terrain.glb + planview.png).
  pt-scan produces what ScanRef points to, but the crate itself is standalone.
- **pt-test-utils**: `timed(|| { ... })` enforces 10s timeout; `run_with_timeout(dur, || { ... })`
  for custom timeouts. Used by all domain crate tests.

### Scenario and Milestone Context

- **S.1.1** ("Scan processing"): 30 min savings, currently `NotImplemented`. Requires pt-scan.
- **Milestone**: "pt-scan: PLY parsing + mesh generation" — `delivered_by: None`, unlocks S.1.1.
- T-015-01 covers parsing + filtering. T-015-02 covers mesh generation + export.
  After T-015-01, the milestone stays partially delivered. S.1.1 remains NotImplemented
  until T-015-02 completes. But T-015-01 should claim its portion via a milestone note.

### Specification Requirements (from docs/specification.md)

PLY in → terrain mesh (glTF) + plan view (PNG) + metadata (JSON). Processing pipeline:
1. Statistical outlier removal
2. RANSAC ground plane fitting
3. Voxel downsampling
4. Delaunay triangulation (T-015-02)
5. Mesh decimation (T-015-02)
6. Orthographic projection (T-015-02)
7. glTF binary export (T-015-02)

Steps 1-3 are T-015-01 scope. Steps 4-7 are T-015-02.

## PLY File Format

PLY (Polygon File Format / Stanford Triangle Format) stores 3D point clouds.
SiteScape exports binary little-endian PLY with vertex positions and colors.

Two format variants: ASCII (text, slow to parse) and binary (little/big endian, fast).
Header is always ASCII: `ply`, `format`, `element vertex N`, `property float x/y/z`,
optional `property uchar red/green/blue`, `end_header`, then data.

Typical SiteScape scan: 1-10M vertices, each with (x,y,z) as float32 and (r,g,b) as uint8.
At 15 bytes/vertex (3×f32 + 3×u8), a 5M-point file is ~75 MB.

## Rust PLY Ecosystem

### ply-rs (original) — v0.1.3
- **Status**: Abandoned (last update Aug 2020). Has CVE-2020-25573 via `linked-hash-map`.
- **Do not use.**

### ply-rs-bw — v3.0.3 (recommended)
- Maintained fork of ply-rs. Last update March 2026. 346K downloads.
- Fixes CVE, Rust 2024 edition, `#![forbid(unsafe_code)]`.
- API: `Parser::<DefaultElement>::new()` → `read_ply(&mut reader)` → header + payload.
- `DefaultElement` is `HashMap<String, Vec<Property>>` — loosely typed.
- Also supports `PropertyAccess` trait for typed vertex parsing into custom structs.
- Supports ASCII + binary big/little endian.

### serde-ply — v0.2.2
- Serde-based approach: `#[derive(Deserialize)]` on vertex struct, `from_reader()`.
- Newer, smaller community (8.9K downloads). Nice ergonomics but less proven.

**Decision**: Use `ply-rs-bw`. Proven API, maintained, fixes CVE, dominant adoption.

## Spatial Query: KD-Tree

Statistical outlier removal and potentially RANSAC need efficient nearest-neighbor queries.

### kiddo — v5.3.0 (recommended)
- Best-in-class Rust KD-tree. 5.2M downloads, actively maintained.
- `ImmutableKdTree<f32, 3>` — built from slice, fast queries, low memory.
- `nearest_n(&query, k)` — k-nearest neighbors, returns distances + indices.
- `nearest_one(&query)` — single nearest neighbor.
- Point type: `[f32; 3]` arrays (no nalgebra dependency required for basic use).

### nalgebra — v0.34
- Standard Rust linear algebra. Needed for RANSAC plane fitting (cross products, normals).
- `Vector3<f32>` for point math, but `kiddo` wants `[f32; 3]`.
- Use nalgebra for plane math, raw arrays for KD-tree.

### rand — v0.10
- RANSAC random sampling. `rand::seq::index::sample()` for 3-point selection per iteration.

## Algorithms to Implement

### 1. Statistical Outlier Removal (~35 LOC)
For each point: find k nearest neighbors (k=30 typical), compute mean distance.
Compute global mean μ and stddev σ of all mean distances.
Remove points where mean_distance > μ + multiplier × σ (multiplier=2.0 typical).
Uses: kiddo `nearest_n()`.

### 2. Voxel Downsampling (~25 LOC)
Quantize each point (x,y,z) to grid cell: `(floor(x/voxel_size), ...)`.
HashMap<(i32,i32,i32), accumulator> — average position and color per cell.
No external dependency beyond `std::collections::HashMap`.

### 3. RANSAC Ground Plane Fitting (~60 LOC)
Repeat N iterations (1000 typical):
  - Sample 3 random points
  - Compute plane normal via cross product: n = (p1-p0) × (p2-p0)
  - Count inliers: points within distance_threshold of plane (0.02m typical)
  - Keep best plane (most inliers)
Classify: ground (within threshold) vs. above-ground (beyond threshold).
Uses: nalgebra for cross product/dot product, rand for sampling.

## Performance Considerations

Target: 5M points < 10 seconds for full pipeline (outlier removal + downsampling + RANSAC).

- KD-tree build for 5M points: ~1-2 seconds with kiddo ImmutableKdTree.
- k-NN queries (k=30) for 5M points: ~3-5 seconds (most expensive step).
- Voxel downsampling: O(n) HashMap insert, sub-second.
- RANSAC 1000 iterations on 5M points: ~1-2 seconds (just distance checks).
- Total estimate: ~6-9 seconds. Tight but achievable.
- If outlier removal is too slow at 5M points, consider running it after downsampling
  (fewer points = faster k-NN), or reducing k to 20.

## Test Fixture Strategy

No real SiteScape PLY available yet. Ticket says: generate synthetic test fixture.

Synthetic fixture design:
- Flat ground plane at z=0 with noise (σ=0.01m)
- A few box-shaped obstacles (z=0.3 to z=1.0)
- Some outlier points far from the surface
- ~10K points for unit tests (fast), option for larger fixture in benchmarks
- Write as binary little-endian PLY (matches SiteScape format)

This validates the full pipeline: outliers get removed, ground gets separated from
obstacles, downsampling reduces count while preserving shape.

## Constraints and Risks

1. **unsafe_code = "deny"** — cannot use unsafe for performance. kiddo and nalgebra
   handle this internally. Our code stays safe.
2. **5M point performance** — k-NN is the bottleneck. May need to downsample first,
   then do outlier removal on the reduced set. Or accept that outlier removal on
   raw 5M points is the expensive step and tune k lower.
3. **No real PLY fixture yet** — synthetic data validates logic but may miss real-world
   edge cases (NaN coords, degenerate triangles, non-standard PLY properties).
4. **Binary PLY parsing memory** — 5M × 15 bytes = 75MB raw. ply-rs-bw loads entire
   element list into memory. Fine for Lambda with 1-3GB memory config.
5. **f32 precision** — LiDAR data in meters, precision to ~0.001m. f32 has 7 digits
   of precision, sufficient for survey-scale coordinates.

## What T-015-02 Will Need From T-015-01

T-015-02 (mesh generation + export) needs:
- `PointCloud` struct with separated ground/obstacle points
- Each point must have position `[f32; 3]` and optional color `[u8; 3]`
- Ground points must be clean (no outliers) and reasonably dense for triangulation
- The `PointCloud` should carry bounding box and point count metadata

## Summary

Four external dependencies: `ply-rs-bw` (PLY parsing), `kiddo` (KD-tree), `nalgebra`
(linear algebra), `rand` (RANSAC sampling). Three hand-rolled algorithms: statistical
outlier removal, voxel downsampling, RANSAC ground plane fitting. Synthetic test
fixture for validation. Performance is tight at 5M points but achievable with careful
ordering (downsample before outlier removal if needed).
