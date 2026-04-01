---
id: E-014
title: LiDAR Feature Detection & Site Intelligence
status: open
priority: high
sprint: 2
---

## Context

The scan pipeline (pt-scan) processes PLY files into classified point clouds and terrain meshes, but it has no idea *what* the obstacles are. Everything above the ground plane is just "obstacle points." The goal is to cluster those points into distinct features, use BAML to classify them efficiently, and export the result to the Bevy viewer — proving the full PLY → design pipeline.

### Why BAML for feature detection?

Traditional point cloud classification uses expensive compute: PointNet/PointNet++ neural networks, random forest classifiers trained on labeled datasets, or hand-tuned geometric heuristics that break on new environments. BAML offers a different approach:

1. **Cheap geometric clustering first** — DBSCAN with a spatial index is O(n log n). This produces a small number of feature candidates (5-50 per scan).
2. **LLM classifies the candidates, not the points** — the LLM sees structured summaries (height, spread, color, shape), not millions of coordinates. One API call classifies all features.
3. **The LLM suggests clustering refinement** — "clusters 3 and 4 are probably the same tree canopy, merge them" or "cluster 7 is too large, likely two objects." This replaces iterative parameter tuning.
4. **Domain knowledge is free** — the LLM knows that a 6-ft vertical cylinder with brown color at Powell & Market is probably a tree trunk, without training data.

This is compute-efficient because the heavy work (parsing, downsampling, ground plane) is pure geometry, and the expensive reasoning (what *is* this?) is a single structured LLM call on a few KB of JSON, not a GPU inference pass on millions of points.

### Sample data

`assets/scans/samples/` — Powell & Market cable car turnaround (SiteScape, 20.5M vertices, binary LE PLY with RGB). **Minimal scene: two tree trunks and brick paths.** No canopy, no complex structures. This is the ideal baseline:
- Does RANSAC correctly separate brick paths from trunks?
- Does clustering produce exactly 2 clusters?
- Can BAML identify "tree trunk" from just a vertical cylinder with bark color?
- Does the full pipeline → Bevy viewer render something sensible?

### End-to-end goal

```
PLY file
  → pt-scan::process_scan()          [existing, geometry]
  → DBSCAN clustering on obstacles   [new, geometry]
  → FeatureCandidate[] summaries     [new, structured data]
  → BAML ClassifyFeatures()          [new, single LLM call]
  → ClassifiedFeature[]              [labels, confidence, reasoning]
  → pt-scene::generate_scene()       [existing stub, wire up]
  → glTF .glb                        [features as named meshes]
  → Bevy viewer                      [existing, loads glTF via postMessage]
```

## Stories

- S-032: Scan Processing CLI & Sample Pipeline
- S-033: Geometric Feature Clustering
- S-034: BAML Feature Classification
- S-035: Multimodal Plan-View Analysis

## Success Criteria

- Process real 20M-point PLY end-to-end: PLY → classified features → glTF → Bevy viewer
- Powell & Market scan produces exactly 2 feature clusters (the two trunks)
- BAML classifies both as tree trunks with reasonable confidence
- Annotated plan-view PNG shows labeled features on the brick path ground plane
- BAML call is a single request with <5KB of structured JSON (not raw points)
- S.1.1 advances integration level
- All BAML calls mockable (ClaudeCliGenerator for dev, fixture-based for CI)
- `just check` passes
