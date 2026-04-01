---
id: T-015-01
story: S-015
title: ply-parsing-filtering
type: task
status: open
priority: critical
phase: done
depends_on: []
---

## Context

First half of the scan pipeline: read a PLY file and produce a clean, downsampled point cloud ready for mesh generation. SiteScape exports binary PLY with vertex positions and colors. We need to handle real-world scan data — noisy, possibly with outliers, potentially millions of points.

This is foundational — the entire product starts with a PLY file.

## Acceptance Criteria

- pt-scan crate created in crates/pt-scan/
- PLY parser: reads binary and ASCII PLY formats, extracts vertex (x, y, z) and optional (r, g, b)
- Statistical outlier removal: remove points that are far from their neighbors (configurable threshold)
- Voxel downsampling: reduce point cloud density to configurable resolution (e.g., 2cm voxels)
- RANSAC ground plane fitting: classify points as ground vs. above-ground
- Output: a PointCloud struct with ground points separated from obstacle points
- Performance: processes 5M points in < 10 seconds
- Test with a real SiteScape PLY fixture (we need to capture or source one)
- If no real PLY available yet, generate a synthetic test fixture (flat ground + some boxes)
