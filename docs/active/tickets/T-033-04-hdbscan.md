---
id: T-033-04
story: S-033
title: hdbscan-clustering
type: task
status: open
priority: high
phase: ready
depends_on: [T-033-03]
---

## Context

DBSCAN with a single epsilon over-segments the Powell & Market scan (12 clusters, expect 2-3). HDBSCAN runs DBSCAN across a range of epsilon values and extracts the most stable clusters from the resulting dendrogram. It handles varying density without manual tuning.

## Background

HDBSCAN algorithm:
1. Compute core distances for each point (distance to K-th nearest neighbor)
2. Build mutual reachability graph: d_mreach(a,b) = max(core(a), core(b), d(a,b))
3. Construct minimum spanning tree on mutual reachability distances
4. Build cluster hierarchy (dendrogram) by processing MST edges in order
5. Extract stable clusters using excess-of-mass method

No training, no GPU. O(n log n) with a KD-tree for core distances + MST.

## Acceptance Criteria

- Replace or augment DBSCAN with HDBSCAN in pt-scan clustering
- Use `hdbscan` crate 0.12 (added to workspace, pure Rust, no ndarray)
- `hdbscan_cluster(points: &[Point3], features: &[PointFeatures], config: &HdbscanConfig) -> ClusterResult`
- HdbscanConfig: min_cluster_size (default 100-500), min_samples (default 10)
- The `hdbscan` crate takes `Vec<Vec<f64>>` — build feature vectors from points + eigenvalue features
- Cluster in augmented feature space: [x, y, z, planarity, linearity, sphericity] (normalized)
  - Spatial coordinates weighted by a configurable factor vs feature coordinates
- Powell & Market validation: should produce 2-4 clusters (brick path(s), tree trunk(s), possibly one structure)
- Unit tests: same cases as DBSCAN (two blobs, noise, single cluster, empty)
- Performance: < 1s for 122K points on M-series Mac
- Falls back gracefully if all points are noise (returns empty clusters)

## Implementation Notes

- The MST can be built with Prim's algorithm using the KD-tree for nearest-neighbor queries in mutual reachability space
- The excess-of-mass extraction is the key step — it picks clusters that are "most stable" across epsilon values
- Consider using the `hdbscan` crate if one exists, or implement from the well-documented algorithm
- Feature space weighting is critical: too much weight on XYZ = over-segmentation (same as DBSCAN), too much on features = merges distant objects. Start with equal weighting and tune.
- The dendrogram is also valuable output — it shows the merge hierarchy, useful for interactive split/merge in the viewer later
