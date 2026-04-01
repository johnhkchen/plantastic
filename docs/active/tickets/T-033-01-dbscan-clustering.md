---
id: T-033-01
story: S-033
title: dbscan-clustering
type: task
status: open
priority: high
phase: done
depends_on: [T-032-01]
---

## Context

Obstacle points from pt-scan are an undifferentiated cloud. DBSCAN (Density-Based Spatial Clustering of Applications with Noise) groups spatially proximate points into distinct features — each cluster becomes a candidate for BAML classification.

## Acceptance Criteria

- Add `cluster` module to pt-scan (or new `pt-features` crate if cleaner)
- `cluster_obstacles(obstacles: &[Point3], config: &ClusterConfig) -> Vec<Cluster>`
- Cluster struct: cluster_id, points (indices), centroid, bounding_box
- ClusterConfig: epsilon (neighborhood radius), min_points (cluster minimum)
- Noise points (not in any cluster) assigned to a special noise cluster or discarded
- Default config tuned for outdoor scans: epsilon=0.3m, min_points=50
- **Powell & Market validation: should produce exactly 2 clusters (the two tree trunks)**
- Unit tests:
  - Two well-separated groups → two clusters
  - Sparse noise between groups → noise, not merged
  - Single dense group → one cluster
  - Empty input → empty output
- Performance: < 5s for Powell & Market obstacle set on M-series Mac

## Implementation Notes

- Use k-d tree (kiddo crate, already a pt-scan dep) for range queries — O(n log n)
- DBSCAN is O(n²) worst case but O(n log n) with spatial index
- Epsilon should be configurable per scan — urban scans have tighter features than rural
- Consider outputting noise points separately (they may be scan artifacts or small objects)
