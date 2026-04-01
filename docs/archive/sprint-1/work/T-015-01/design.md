# T-015-01 Design: PLY Parsing & Point Cloud Filtering

## Decision Summary

Hand-roll all three algorithms (outlier removal, voxel downsampling, RANSAC) using
`ply-rs-bw` for PLY parsing, `kiddo` for KD-tree queries, `nalgebra` for vector math,
and `rand` for RANSAC sampling. Pipeline order: parse → downsample → outlier removal
→ RANSAC ground fitting. f32 throughout.

## Approach A: Use `threecrate-algorithms` for Everything

Pull in `threecrate-algorithms` which provides `statistical_outlier_removal()`,
`segment_plane_ransac()`, and voxel grid filtering out of the box.

**Pros**: Least code to write. Already tested. Uses nalgebra + kiddo internally.
**Cons**: Young crate (2.9K downloads, Feb 2025). Adds a large transitive dependency
tree for ~100 LOC of logic. Internal data structures may not match our `PointCloud`
shape, requiring conversion overhead. Less control over algorithm parameters. If the
crate breaks or becomes unmaintained, we're stuck.

**Rejected**: The algorithms are each 25-60 lines. The dependency cost exceeds the
implementation cost.

## Approach B: serde-ply + Custom Algorithms

Use `serde-ply` for PLY parsing with `#[derive(Deserialize)]` on vertex struct.
Hand-roll filtering algorithms.

**Pros**: Clean serde ergonomics — vertices deserialize directly into typed structs.
**Cons**: serde-ply has only 8.9K downloads. Less battle-tested for edge cases in
binary PLY files. If SiteScape exports non-standard properties (normals, confidence),
the rigid derive approach needs more boilerplate than DefaultElement.

**Rejected**: ply-rs-bw is the safer bet for PLY parsing reliability. The ergonomic
advantage of serde-ply is marginal when we're extracting 3-6 fields from a vertex.

## Approach C: ply-rs-bw + Hand-Rolled Algorithms (CHOSEN)

Use `ply-rs-bw` for PLY parsing. Hand-roll statistical outlier removal, voxel
downsampling, and RANSAC ground plane fitting using `kiddo`, `nalgebra`, and `rand`.

**Pros**:
- ply-rs-bw is the dominant maintained PLY crate (346K downloads).
- Each algorithm is 25-60 LOC — trivial to implement, test, and maintain.
- Full control over parameters (k, threshold, voxel_size, iterations).
- Minimal dependency surface — only add what we actually need.
- `kiddo` ImmutableKdTree is the fastest Rust KD-tree for read-heavy workloads.
- Follows existing crate pattern: pure computation, no I/O beyond file read.

**Cons**:
- More code to write than Approach A (~120 LOC for three algorithms).
- Must write our own tests for well-known algorithms.

**Why chosen**: The algorithms are simple, well-understood, and small. Hand-rolling
gives us precise control over the pipeline and avoids dependency on a young crate.
This matches the project's pattern of thin, focused crates.

## Pipeline Design

### Processing Order

```
PLY file → parse → voxel downsample → outlier removal → RANSAC → PointCloud
```

**Why downsample first**: Voxel downsampling is O(n) and reduces point count
dramatically (5M → ~500K at 2cm voxels). Running outlier removal on the reduced
set makes k-NN queries ~10x faster. This makes the 10-second budget comfortable.

Alternative order (outlier removal first, then downsample) is correct but slower —
k-NN on 5M points is the bottleneck.

### Data Flow

```
parse_ply(reader) → Vec<RawVertex>          // (x,y,z) + optional (r,g,b)
                      ↓
voxel_downsample(verts, voxel_size) → Vec<RawVertex>   // averaged per cell
                      ↓
remove_outliers(verts, k, threshold) → Vec<RawVertex>  // statistical filter
                      ↓
fit_ground_plane(verts, config) → GroundClassification  // RANSAC
                      ↓
PointCloud { ground, obstacles, metadata }              // final output
```

## Key Type Decisions

### Point Representation: `[f32; 3]` + Optional `[u8; 3]`

Use `[f32; 3]` for positions (what `kiddo` expects). Carry color as optional `[u8; 3]`.
Don't use `nalgebra::Point3` as the primary type — conversion to/from kiddo's array
type would be overhead on every spatial query. Use nalgebra only inside RANSAC for
cross-product math.

### PointCloud Output Struct

```rust
pub struct PointCloud {
    pub ground: Vec<Point>,      // classified as ground
    pub obstacles: Vec<Point>,   // classified as above-ground
    pub metadata: ScanMetadata,  // bbox, counts, plane equation
}

pub struct Point {
    pub position: [f32; 3],
    pub color: Option<[u8; 3]>,
}

pub struct ScanMetadata {
    pub bbox: BoundingBox,
    pub original_count: usize,
    pub filtered_count: usize,    // after outlier removal
    pub ground_count: usize,
    pub obstacle_count: usize,
    pub ground_plane: Plane,
}
```

### Configuration: Builder Pattern with Sensible Defaults

```rust
pub struct ScanConfig {
    pub voxel_size: f32,           // default: 0.02 (2cm)
    pub outlier_k: usize,          // default: 30
    pub outlier_threshold: f32,    // default: 2.0 (std deviations)
    pub ransac_iterations: usize,  // default: 1000
    pub ransac_threshold: f32,     // default: 0.02 (2cm)
}
```

Provide `ScanConfig::default()` with sane values. Allow callers to override.

## Algorithm Details

### Statistical Outlier Removal

```
for each point:
    find k nearest neighbors via KD-tree
    compute mean distance to those neighbors
compute global mean μ and stddev σ of all mean distances
keep points where mean_distance ≤ μ + threshold × σ
```

Input: `&[RawVertex]`, k, threshold → Output: `Vec<RawVertex>` (filtered).
Build `ImmutableKdTree<f32, 3>` from positions, query `nearest_n` for each point.

### Voxel Downsampling

```
for each point:
    cell = (floor(x / voxel_size), floor(y / voxel_size), floor(z / voxel_size))
    accumulate position sum, color sum, count into HashMap[cell]
for each cell:
    output point = averages of accumulated values
```

Input: `&[RawVertex]`, voxel_size → Output: `Vec<RawVertex>` (reduced).
Uses `HashMap<(i32, i32, i32), Accumulator>`. No external dependency.

### RANSAC Ground Plane Fitting

```
best_plane = None
for _ in 0..iterations:
    sample 3 random points (indices via rand)
    compute plane: normal = (p1-p0) × (p2-p0), d = -normal · p0
    count inliers: |normal · point + d| / |normal| ≤ threshold
    if inliers > best_count: update best_plane
classify all points against best_plane
```

Input: `&[RawVertex]`, iterations, threshold → Output: `GroundClassification`.
Uses nalgebra Vector3 for cross product. Uses rand for random index sampling.

## Error Handling

Use `thiserror` for `ScanError` enum:
- `InvalidPly(String)` — PLY parse errors, missing vertex element, missing properties
- `InsufficientPoints` — too few points for RANSAC (need ≥ 3)
- `NoGroundPlane` — RANSAC found no plane with enough inliers
- `Io(std::io::Error)` — file read errors

Return `Result<PointCloud, ScanError>` from the top-level processing function.

## Test Strategy

### Synthetic PLY Fixture

Generate a binary little-endian PLY file in test setup:
- 1000 ground points: z ∈ [0, 0.01] with uniform x,y spread over 10m × 10m
- 200 obstacle points: boxes at known positions, z ∈ [0.3, 1.0]
- 50 outlier points: scattered far from the surface (z > 5.0 or z < -5.0)
- Total: ~1250 points (fast for unit tests)

### Unit Tests

1. **PLY parsing**: Parse synthetic fixture, verify vertex count and coordinate ranges.
2. **Voxel downsampling**: Input 1000 uniform points, verify output count matches
   expected cell count. Verify averaged positions are within voxel centers.
3. **Outlier removal**: Input with known outliers, verify they're removed and
   non-outliers retained.
4. **RANSAC**: Input with known ground plane (z=0), verify fitted plane normal is
   approximately (0, 0, 1) and d ≈ 0.
5. **Full pipeline**: Parse → process → verify ground/obstacle separation matches
   expected classification.
6. **ASCII PLY**: Verify ASCII format also parses correctly.

### Property: Tests compute expected values independently

Per CLAUDE.md rule 2: expected point counts, plane normals, and bounding boxes
are computed in the test from the known fixture geometry — not by calling pt-scan
functions.

## What Was Rejected

1. **f64 precision**: f32 is sufficient for LiDAR (7 decimal digits, ~0.1mm at 100m).
   f64 doubles memory and slows KD-tree queries for no measurable benefit.
2. **Rayon parallelism**: Not needed in V1. Single-threaded pipeline meets the 10s
   target after downsampling reduces the point count. Add parallelism later if needed.
3. **Streaming PLY parse**: ply-rs-bw supports it, but loading all vertices into memory
   is simpler and 75MB is fine for Lambda (1-3GB memory). Streaming adds complexity
   without benefit here.
4. **Normal estimation**: Not needed for T-015-01. T-015-02 may need it for mesh
   quality, but ground plane normals from RANSAC are sufficient for classification.
