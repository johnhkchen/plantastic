# T-033-02 Research: Feature Candidates

## Scope

Extract geometric summaries from DBSCAN clusters into `FeatureCandidate` structs
that the BAML classifier consumes. The LLM never sees raw point data.

## Existing Codebase

### Cluster (cluster.rs)

```rust
pub struct Cluster {
    pub id: u32,
    pub point_indices: Vec<usize>,   // indices into obstacle point slice
    pub centroid: [f32; 3],          // mean position
    pub bbox: BoundingBox,           // axis-aligned bounding box
}
```

Clusters are produced by `cluster_obstacles(points: &[Point], config: &ClusterConfig) -> ClusterResult`.
The input `points` slice is `cloud.obstacles` — already ground-separated.

### Point (types.rs)

```rust
pub struct Point {
    pub position: [f32; 3],       // [x, y, z] meters
    pub color: Option<[u8; 3]>,   // optional RGB
}
```

Color is populated from PLY files that have `red`, `green`, `blue` properties.
The synthetic test PLY helper (`make_synthetic_ply`) does emit RGB.

### Ground Plane (types.rs)

```rust
pub struct Plane {
    pub normal: [f32; 3],   // unit normal
    pub d: f32,             // n·p + d = 0
}
```

**No `GroundPlane` struct or `offset_at` method exists.** The ticket's acceptance
criteria reference `ground_plane.offset_at(x,y)` but this must be implemented.
Height above ground for point p = `|normal · p.position + d|`.
For a near-horizontal ground plane (normal ≈ [0,0,1]), this simplifies to
`|p.z + d|`, but we should use the full dot product for correctness.

The ground plane is stored in:
- `PointCloud.metadata.ground_plane: Plane`
- `ScanReport.ground.plane: Plane`

### Height Calculation Pattern (lib.rs:117-126)

Already established in `process_scan_timed`:
```rust
let obstacle_heights: Vec<f32> = obstacles.iter().map(|p| {
    (plane.normal[0] * p.position[0]
        + plane.normal[1] * p.position[1]
        + plane.normal[2] * p.position[2]
        + plane.d).abs()
}).collect();
```

### Public API (lib.rs)

Exports: `Cluster, ClusterConfig, ClusterResult, BoundingBox, Plane, Point,
PointCloud, ScanConfig, ScanMetadata`, plus report types and export functions.
`FeatureCandidate` and `extract_candidates` will be new public exports.

### CLI Example (examples/process_sample.rs)

Full pipeline: PLY → process → terrain export. Prints per-stage timing.
Does NOT currently run clustering. Needs extension: cluster → extract candidates → print table.

### Integration Tests (tests/integration.rs)

611 lines. Uses `make_synthetic_ply()` for deterministic test data with known geometry.
`test_powell_market_two_clusters` tests against real scan file (optional, skips if missing).
Pattern: `timed(|| { ... })` wrapper for all tests.

### BAML Schemas (baml_src/)

Only `proposal.baml` exists — no feature classification schema yet.
The ticket says `FeatureCandidate` should match the BAML schema "exactly (same field names)".
Since no BAML schema exists yet, we define the Rust struct first; a future ticket
will add the BAML class mirroring these fields.

## Key Computations Needed

### 1. Height (height_ft)
- Per-cluster: max signed distance from ground plane among all cluster points
- Convert meters → feet: `* 3.28084`
- Must use ground plane distance, not raw z coordinate

### 2. Spread (spread_ft)
- Max horizontal extent of the cluster
- Horizontal = XY plane (assuming Z is up, which it is pre-export)
- `spread = max(bbox.max[0]-bbox.min[0], bbox.max[1]-bbox.min[1])`
- Convert meters → feet

### 3. Density (points per cubic meter)
- `point_count / volume_cubic_meters`
- Volume from bbox: `(max[0]-min[0]) * (max[1]-min[1]) * (max[2]-min[2])`
- Edge case: flat cluster (one bbox dimension ≈ 0) → use minimum dimension of 0.01m

### 4. Dominant Color
- Compute RGB histogram per cluster from `Point.color`
- Map to coarse names: "green", "brown", "gray", "white", "mixed"
- Handle `color: None` → "unknown" or skip those points

### 5. Vertical Profile
- Ticket spec: `height/spread > 3 → columnar, < 0.5 → flat, else spreading`
- Also need "conical" and "irregular" from acceptance criteria
- Conical: tapers upward (XY spread decreases with Z)
- Irregular: doesn't fit other categories
- Simple heuristic: ratio-based with taper check for conical

### 6. Centroid
- Already computed in Cluster struct as [f32; 3]
- Ticket wants [f64; 3] — cast for BAML compatibility

## Constraints

- Serde Serialize required (for BAML input serialization)
- f64 fields in FeatureCandidate (ticket spec) vs f32 in existing structs
- Color may be None for some/all points in a cluster
- Empty clusters impossible (DBSCAN min_points >= 1 by convention, default 50)
- Volume can be zero for degenerate clusters → clamp

## Dependencies

- T-033-01 (DBSCAN clustering) — merged, `Cluster` and `ClusterResult` available
- No external crate dependencies needed beyond what's already in Cargo.toml

## Files to Modify

- `crates/pt-scan/src/` — new module `feature.rs` or extend `cluster.rs`
- `crates/pt-scan/src/lib.rs` — export new types and function
- `crates/pt-scan/examples/process_sample.rs` — add clustering + candidates table
- `crates/pt-scan/tests/integration.rs` — add candidate extraction tests

## Risks

- Color classification is heuristic — real scans may have unexpected color distributions
- Vertical profile "conical" detection requires analyzing Z-slice spread, more complex than ratio
- Ground plane may not be perfectly horizontal → height calculation must use full plane equation
