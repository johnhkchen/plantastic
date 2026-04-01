---
id: S-032
epic: E-014
title: Scan Processing CLI & Sample Pipeline
status: open
priority: high
tickets: [T-032-01, T-032-02]
---

## Goal

Run the real Powell & Market PLY through the existing pt-scan pipeline, measure performance, produce terrain mesh and plan-view PNG. Establish the baseline before adding intelligence.

## Acceptance Criteria

- CLI example processes the sample PLY end-to-end
- Performance measured: parse time, downsample, RANSAC, mesh gen
- Terrain GLB and plan-view PNG written to assets/scans/samples/
- Metadata report: point counts, ground plane, obstacle stats
- `just process-scan <path>` recipe for easy re-runs
