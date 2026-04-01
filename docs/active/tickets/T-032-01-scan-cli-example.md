---
id: T-032-01
story: S-032
title: scan-cli-example
type: task
status: open
priority: high
phase: research
depends_on: []
---

## Context

We have a real 20.5M-point PLY from Powell & Market (SiteScape, binary LE, RGB). Need to run it through pt-scan and measure the pipeline performance before adding intelligence layers.

## Acceptance Criteria

- Create `crates/pt-scan/examples/process_sample.rs`
- Accepts PLY path as CLI arg (default: `assets/scans/samples/Scan at 09.23.ply`)
- Runs full pipeline: parse → downsample → outlier removal → RANSAC → mesh gen → export
- Prints timing for each stage and total
- Prints metadata: point counts at each stage, ground plane, obstacle height range
- Writes outputs: `{name}-terrain.glb`, `{name}-planview.png` alongside input
- Config tuned for outdoor urban scan (5cm voxels, higher RANSAC iterations)
- Add `just process-scan path` recipe to justfile
- Performance target: < 60s for 20M points on M-series Mac

## Implementation Notes

- ScanConfig: voxel_size=0.05, outlier_k=20, outlier_threshold=2.0, ransac_iterations=1000, ransac_threshold=0.05
- The 294MB file is gitignored (*.ply in assets/scans/samples/)
- Output files also gitignored — they're artifacts, not source
- Consider printing a bounding box of the entire cloud (helps understand spatial extent)
