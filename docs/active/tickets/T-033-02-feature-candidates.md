---
id: T-033-02
story: S-033
title: feature-candidates
type: task
status: open
priority: high
phase: done
depends_on: [T-033-01]
---

## Context

Each DBSCAN cluster needs to be summarized into a FeatureCandidate — the structured input the BAML classifier sees. The LLM never touches raw point data; it reasons about geometric summaries.

## Acceptance Criteria

- `FeatureCandidate` struct (serde Serialize for BAML input):
  - cluster_id: usize
  - centroid: [f64; 3]
  - bbox_min/bbox_max: [f64; 3]
  - height_ft: f64 (max_z - ground_plane_z at centroid, converted to feet)
  - spread_ft: f64 (max horizontal extent, converted to feet)
  - point_count: usize
  - dominant_color: String ("green", "brown", "gray", "white", "mixed")
  - vertical_profile: String ("conical", "spreading", "columnar", "flat", "irregular")
  - density: f64 (points per cubic meter)
- `extract_candidates(clusters: &[Cluster], ground_plane: &GroundPlane) -> Vec<FeatureCandidate>`
- Color classification: compute RGB histogram per cluster, map to dominant color name
- Vertical profile: analyze Z distribution shape (tall/narrow=columnar, wide/low=spreading, etc.)
- Unit tests with known synthetic clusters → expected candidates
- CLI example extended: print feature candidates table after clustering
- Powell & Market should produce ~5-20 distinct candidates (trees, poles, structures)

## Implementation Notes

- Height is measured from ground plane, not from z=0 — use ground_plane.offset_at(x,y)
- Feet conversion: 1 meter ≈ 3.28084 feet
- Color names are intentionally coarse — the LLM interprets fine distinctions
- Vertical profile heuristic: if height/spread > 3 = columnar, < 0.5 = flat, else spreading
- The FeatureCandidate struct should match the BAML schema exactly (same field names)
