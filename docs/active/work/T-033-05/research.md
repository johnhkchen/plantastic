# T-033-05 Research: Rerun Debug Visualization

## Ticket Summary

Add a Rerun-based debug visualization example to pt-scan for inspecting point cloud
segmentation. Dev-only dependency — not shipped in production.

## Codebase Map

### pt-scan crate (`crates/pt-scan/`)

**Key types for visualization:**
- `Point { position: [f32; 3], color: Option<[u8; 3]> }` — core point type
- `PointCloud { ground: Vec<Point>, obstacles: Vec<Point>, metadata: ScanMetadata }` — pipeline output
- `PointFeatures { planarity, linearity, sphericity, omnivariance, normal, curvature }` — per-point eigenvalue features
- `ClusterResult { clusters: Vec<Cluster>, noise_indices: Vec<usize> }` — HDBSCAN output
- `Cluster { id: u32, point_indices: Vec<usize>, centroid: [f32; 3], bbox: BoundingBox }` — single cluster
- `FeatureCandidate { cluster_id, height_ft, spread_ft, point_count, density, dominant_color, vertical_profile, centroid, bbox_min, bbox_max }` — classified cluster summary
- `BoundingBox { min: [f32; 3], max: [f32; 3] }` — axis-aligned bounds

**Pipeline functions used by existing examples:**
1. `process_scan_timed(reader, config) -> (PointCloud, ScanReport)` — parse → downsample → outlier → RANSAC
2. `compute_point_features(points, k) -> Vec<PointFeatures>` — eigenvalue features (k=30 typical)
3. `hdbscan_cluster(points, features, config) -> ClusterResult` — 6D augmented clustering
4. `extract_candidates(clusters, points, ground_plane) -> Vec<FeatureCandidate>` — cluster summaries

**Existing examples:**
- `process_sample.rs` — full pipeline with DBSCAN (old), uses `cluster_obstacles()`
- `scan_to_quote.rs` — scan-to-quote demo, also uses DBSCAN

Neither example currently uses HDBSCAN or eigenvalue features in the visualization.
Both use `ClusterConfig::default()` (DBSCAN), not `HdbscanConfig`.

### Justfile recipes

- `process-scan` and `scan-to-quote` — existing examples
- No `debug-scan` recipe yet

### Sample data

- `assets/scans/samples/Scan at 09.23.ply` — default sample
- `assets/scans/samples/powell-market-downsampled.ply` — urban planter

### Rerun SDK

Rerun (`rerun` crate on crates.io) provides:
- `RecordingStream` — the logging handle
- `rerun::Points3D` — 3D point cloud archetype with positions, colors, radii
- `rerun::Boxes3D` — 3D bounding boxes with centers, half-sizes, labels, colors
- `rerun::Scalar` — scalar timeline values (for feature heatmaps via Scalars or per-point)
- `rerun::spawn()` — launch viewer and connect automatically
- Timeline/sequence stepping via `set_time_sequence("step", N)`
- Entity paths for hierarchical organization (e.g., `/raw/points`, `/features/planarity`)

Rerun is ~50 MB compile dependency — acceptable for dev, must be dev-only.

### Constraints

- `rerun` must be a dev-dependency only (Cargo feature or dev-dependencies)
- Cannot be behind a Cargo feature that's enabled by default (would inflate Lambda)
- Example binary (`examples/`) is the right approach — never compiled for `--release` production
- The example should use HDBSCAN (not DBSCAN) since that's what T-033-04 delivered
- Must work with `rr::spawn()` for local use — no .rrd file management needed

### Dependencies / blockers

- T-033-04 (HDBSCAN) is the dependency — already merged on main
- `compute_point_features()` and `hdbscan_cluster()` are both public and available
