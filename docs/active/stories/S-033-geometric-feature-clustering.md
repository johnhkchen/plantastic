---
id: S-033
epic: E-014
title: Geometric Feature Clustering
status: open
priority: high
depends_on: [S-032]
tickets: [T-033-01, T-033-02, T-033-03, T-033-04, T-033-05]
---

## Goal

Cluster obstacle points into distinct features with geometric summaries. This is the structured input that feeds the BAML classification — the LLM never sees raw point clouds, only geometric summaries.

## Acceptance Criteria

- DBSCAN or similar density-based clustering on obstacle points
- Per-cluster FeatureCandidate: centroid, bounding box, height, spread, point count, dominant color, vertical profile shape
- Handles noise (unclustered points) gracefully
- Powell & Market scan produces distinct clusters for trees, poles, benches, etc.
- Unit tests with synthetic clusters (known geometry → expected candidates)
