# T-033-01 Research: DBSCAN Clustering

## Current State

### Obstacle Points
After the scan pipeline (parse → downsample → outlier removal → RANSAC), obstacle points are stored as an undifferentiated `Vec<Point>` in `PointCloud.obstacles`. Each `Point` has `position: [f32; 3]` and optional `color: Option<[u8; 3]>`. There is no grouping, labeling, or spatial clustering applied to obstacles today.

### Pipeline Flow (lib.rs)
```
parse_ply() → voxel_downsample() → remove_outliers() → fit_ground_plane()
                                                              ↓
                                                    GroundClassification
                                                    ├── ground_indices
                                                    └── obstacle_indices
```
The obstacle indices are used to split filtered points into `PointCloud.ground` and `PointCloud.obstacles`. Clustering would operate on the obstacle points post-RANSAC.

### K-D Tree Infrastructure (filter.rs)
`kiddo::ImmutableKdTree<f32, 3>` is already used in `remove_outliers()` for k-NN queries. Key patterns:
- Build: `ImmutableKdTree::new_from_slice(&positions)` where positions is `Vec<[f32; 3]>`
- Query: `tree.nearest_n::<kiddo::SquaredEuclidean>(&pos, NonZeroUsize::new(k))`
- For DBSCAN we need **range queries** instead: `tree.within::<kiddo::SquaredEuclidean>(&pos, eps_squared)` — kiddo v5 supports this via `within()` returning all neighbors within a radius.

### Types (types.rs)
- `Point { position: [f32; 3], color: Option<[u8; 3]> }` — the core point type
- `BoundingBox { min: [f32; 3], max: [f32; 3] }` — already has `from_points()`
- `PointCloud { ground, obstacles, metadata }` — final output
- `ScanConfig` — pipeline configuration, no clustering params yet

### Report (report.rs)
`ObstacleInfo { count, height_range, bbox }` — flat aggregate stats. No per-cluster breakdown. A future enhancement could add cluster-level detail here.

### Integration Tests (tests/integration.rs)
Synthetic PLY generation via `make_synthetic_ply(ground_n, obstacle_n, outlier_n)`:
- Obstacles: box at x∈[2,3], y∈[2,3], z∈[0.3,1.0] — a single cluster
- Tests validate pipeline round-trip, metadata consistency, outlier removal

### Powell & Market Scan
`assets/scans/samples/Scan at 09.23.ply` — a real iPhone LiDAR scan. The acceptance criteria reference "Powell & Market validation: should produce exactly 2 clusters (the two tree trunks)." This is the validation dataset.

### Dependencies (Cargo.toml)
- `kiddo = "5"` — k-d tree, already present
- `nalgebra = "0.34"` — linear algebra, used in RANSAC
- `rand = "0.10"` — random sampling
- No clustering crate currently imported

### Downstream Consumers
T-033-02 (feature-candidates) depends on T-033-01. It will consume clusters to produce `FeatureCandidate` structs for BAML classification (T-034-01). The cluster output must include: cluster identity, constituent point indices, centroid, and bounding box.

## Constraints & Observations

1. **No external DBSCAN crate needed.** DBSCAN is ~50 lines with a spatial index. kiddo provides the range query. Rolling our own avoids a dependency and gives us full control over the output format.

2. **Point type is `Point`, not `Point3`.** The ticket AC says `Point3` but the codebase uses `Point`. The function signature should use `&[Point]`.

3. **kiddo `within()` returns squared distances.** We must pass `epsilon * epsilon` as the radius parameter.

4. **Noise handling matters for downstream.** BAML classification (T-034-01) may want to inspect noise points (small isolated objects). Separating noise from clusters is important.

5. **Performance budget: <5s on M-series Mac.** The Powell & Market scan has ~2.5M raw points; after downsampling + outlier removal + RANSAC, the obstacle set is much smaller (likely a few thousand). DBSCAN with k-d tree should be well within budget.

6. **Module placement.** The ticket suggests either a `cluster` module in pt-scan or a new `pt-features` crate. Given that clustering operates on `Point` (a pt-scan type) and is part of the scan processing pipeline, a module within pt-scan is cleaner. A separate crate would be premature — the boundary isn't clear until feature extraction (T-033-02) is designed.

7. **Config defaults.** Ticket specifies epsilon=0.3m, min_points=50 for outdoor scans. These are starting values; the Powell & Market validation will confirm or adjust them.
