---
id: T-033-05
story: S-033
title: rerun-debug-viz
type: task
status: open
priority: medium
phase: done
depends_on: [T-033-04]
---

## Context

Debugging point cloud segmentation without visualization is flying blind. Rerun (rerun.io) is purpose-built for logging and visualizing 3D point clouds with temporal stepping — perfect for comparing before/after clustering, coloring by feature values, and inspecting individual clusters.

## Acceptance Criteria

- Add `rerun` as a dev-dependency in pt-scan
- Create `examples/debug_segmentation.rs`:
  1. Load pre-downsampled PLY
  2. Log raw points (colored by RGB)
  3. Log eigenvalue features as scalar channels (planarity, linearity, sphericity heatmaps)
  4. Log HDBSCAN cluster assignments (colored by cluster ID)
  5. Log noise points separately (gray)
  6. Log feature candidates as 3D bounding boxes with labels
- Add `just debug-scan <path>` recipe that runs the example and opens Rerun viewer
- NOT a production dependency — dev/debug only, never shipped

## Implementation Notes

- `rerun` Rust SDK: `rr::RecordingStream`, `rr::Points3D`, `rr::Boxes3D`, `rr::Scalar`
- Log to a `.rrd` file or spawn the viewer directly via `rr::spawn()`
- Use timeline steps: "raw" → "features" → "clustered" → "candidates"
- Color mapping: use distinct colors per cluster (rerun has built-in colormaps)
- This is the tool for tuning HDBSCAN parameters — see the effect of min_cluster_size in real-time
- `rerun` is ~50MB dep — acceptable for dev, not for Lambda
