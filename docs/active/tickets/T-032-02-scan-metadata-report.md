---
id: T-032-02
story: S-032
title: scan-metadata-report
type: task
status: open
priority: medium
phase: done
depends_on: [T-032-01]
---

## Context

After processing a scan, we need a structured metadata report that downstream systems (BAML classification, API responses) can consume. Currently metadata is embedded in ClassifiedCloud but not serializable or exportable.

## Acceptance Criteria

- `ScanReport` struct in pt-scan: serializable (serde JSON), includes all processing metadata
  - Input: filename, vertex count, file size, format
  - Processing: downsample ratio, outlier removal count, RANSAC iterations to convergence
  - Ground plane: normal, offset, area estimate
  - Obstacles: count, height range, spatial extent (bounding box)
  - Timing: per-stage durations
  - Output: GLB size, PNG size, triangle count, vertex count
- `process_scan` extended to return timing info (or separate `process_scan_timed`)
- CLI example writes `{name}-report.json` alongside other outputs
- Report JSON readable by future BAML functions as context
- Unit test: verify report serialization round-trip
