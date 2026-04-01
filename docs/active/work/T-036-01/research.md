# T-036-01 Research: Measure Feature Gaps

## Objective

Implement `measure_gaps()` — pure geometry that computes pairwise distances and
clear widths between `FeatureCandidate`s. The gap between two trunks is the planter
zone; this measurement grounds the estimation pipeline in reality.

## Codebase Map

### pt-scan crate (`crates/pt-scan/`)

**Core types (src/types.rs):**
- `Point { position: [f32; 3], color: Option<[u8; 3]> }` — raw scan point
- `Plane { normal: [f32; 3], d: f32 }` — ground plane (n·p + d = 0)
- `BoundingBox { min: [f32; 3], max: [f32; 3] }` — AABB
- `PointCloud { ground, obstacles, metadata: ScanMetadata }` — processed scan
- `ScanMetadata` — includes `ground_plane: Plane`

**Cluster types (src/cluster.rs):**
- `Cluster { id: u32, point_indices, centroid: [f32; 3], bbox: BoundingBox }`
- `ClusterConfig { epsilon: f32, min_points: usize }` — defaults 0.3m / 50
- `ClusterResult { clusters, noise_indices }`
- `cluster_obstacles(points, config) -> ClusterResult`

**Feature types (src/feature.rs):**
- `FeatureCandidate` — 10 fields: cluster_id, centroid[f64;3], bbox_min/max[f64;3],
  height_ft, spread_ft, point_count, dominant_color, vertical_profile, density
- `extract_candidates(clusters, points, ground_plane) -> Vec<FeatureCandidate>`
- All measurements in feet. Centroid and bbox in f64 (meters, converted from f32).
- Note: `spread_ft` = max(dx, dy) of bbox in feet. This is the max horizontal extent.

**Public re-exports (src/lib.rs):**
- `extract_candidates`, `FeatureCandidate`, `Cluster`, `ClusterConfig`, etc.
- No gap-related types exist yet.

**Processing pipeline:**
1. Parse PLY → raw points
2. Voxel downsample → outlier removal
3. RANSAC ground plane fitting → PointCloud
4. cluster_obstacles → ClusterResult
5. extract_candidates → Vec<FeatureCandidate>
6. **Gap measurement goes here** — step 6

### Example pipeline (examples/process_sample.rs)

Runs stages 1–6, prints a feature candidate table. Currently stops after candidate
extraction. Gap measurement would be stage 7, feeding into planter estimation.

### Integration tests (tests/integration.rs)

- Synthetic PLY tests with known geometry
- Real scan file test (Powell & Market) when available
- Feature candidate extraction validation

### Downstream consumers

- **pt-quote** — needs gap area for soil volume and plant count calculations
- **BAML classifier** (T-034-01) — classifies features; gaps complement classification
  by providing spatial relationships between classified features

## Key Measurements Available on FeatureCandidate

For gap computation, the relevant fields per candidate:
- `centroid: [f64; 3]` — center of cluster (meters)
- `bbox_min/bbox_max: [f64; 3]` — AABB corners (meters)
- `spread_ft: f64` — max(dx, dy) of bbox (feet)

The ticket says:
- `clear_width = centroid_distance - (spread_a/2 + spread_b/2)` — actual plantable width
- But `spread_ft` is the MAX of dx/dy, not the radius. For the gap axis, we need
  the spread along the line connecting the two centroids, not the max horizontal extent.

## Geometric Considerations

### Centroid distance
Simple 2D Euclidean distance in XY plane: `sqrt((x2-x1)² + (y2-y1)²)`.
The Z component should be excluded — vertical separation isn't a gap.

### Clear width (the plantable width)
The ticket defines: `clear_width = centroid_dist - (spread_a/2 + spread_b/2)`
where spread is the horizontal extent. But this is approximate — a cylinder's spread
is the same in all directions, but a rectangular planter is not. Using `spread_ft / 2`
as a radius is the simplest approximation, treating each feature as a circle with
diameter = spread_ft. Good enough for trunks (which are roughly circular in XY).

### Clear length
The ticket says `clear_length_ft` but doesn't define it precisely. For a rectangular
gap approximation between two roughly circular features, the length could be the
spread perpendicular to the gap axis. For the initial implementation, using the
average spread of the two features as clear_length is reasonable.

Actually, re-reading the ticket: area_sqft = clear_width × clear_length. This is
the rectangular approximation of the plantable zone between two features. The length
should be the extent of the gap in the direction perpendicular to the line connecting
the centroids. For trunk-like features, this is roughly the spread of the features
(both trunks define the "length" of the planter between them).

### Ground elevation at gap midpoint
The ground plane is `Plane { normal, d }`. The midpoint between two centroids
projected onto the ground plane gives the elevation. For a roughly-horizontal plane
(normal ≈ [0,0,1]), elevation ≈ -d. For tilted planes, we evaluate
`-(normal[0]*mx + normal[1]*my + d) / normal[2]` where (mx, my) is the midpoint.

### Distance threshold
The ticket says "configurable threshold (e.g., 30 ft)" for which pairs to consider.
For N candidates, we compute N*(N-1)/2 pairs and filter. With typical scan sizes
(2-20 features), this is trivially small.

## Conversion Conventions

- Internal computations: meters (matching centroid/bbox units)
- Output: feet (matching existing FeatureCandidate convention)
- M_TO_FT = 3.28084 (already defined in feature.rs)

## Existing Pattern

`feature.rs` is the closest analog:
- Pure function taking structured data, returning a Vec of results
- Serialize derives for JSON output
- f64 for numeric fields (BAML JSON compatibility)
- Unit tests with `pt_test_utils::timed`
- Helper functions for geometric computations

## Constraints

- No external dependencies needed — pure arithmetic
- Must be Serialize for JSON report output
- The Plane type is `Plane` (not `GroundPlane` as the ticket says)
- Candidate centroid is in meters (f64), but was converted from f32 cluster centroid
- Spread is in feet, centroid is in meters — conversions needed

## Open Questions

1. Should clear_length be defined as the average spread of the two features, or as
   a separate measurement? The ticket says `clear_length_ft` but doesn't define it.
   → Decision for Design phase.
2. Should gap ID be synthetic (pair index) or derived from feature IDs?
   → Ticket says feature_a_id/feature_b_id, so the gap is identified by its pair.
