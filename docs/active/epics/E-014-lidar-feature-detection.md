---
id: E-014
title: LiDAR Feature Detection & Site Intelligence
status: open
priority: high
sprint: 2
---

## Context

The scan pipeline (pt-scan) processes PLY files into classified point clouds and terrain meshes, but it has no idea *what* the obstacles are. A tree, a fence, a bench, a trolley pole — all just "points above the ground plane." The "wow" moment is when the system looks at a scan and says: "I see a 25-foot London Plane tree, a concrete sidewalk slab, and a metal transit shelter."

This is the most differentiating capability in the product. It combines:
1. Geometric analysis (clustering, bounding boxes, height/spread, color histograms)
2. LLM interpretation (BAML-powered domain-expert classification)
3. Multimodal vision (plan-view image analysis)
4. Cross-referencing (satellite baseline reconciliation)

## Architecture

```
PLY file
  → pt-scan::process_scan() (existing)
  → ClassifiedCloud { ground, obstacles }
  │
  ├─ Tier 1: Geometric clustering (DBSCAN)
  │    → FeatureCandidate[] { bbox, height, spread, color, shape }
  │    → BAML ClassifyFeatures() → ClassifiedFeature[]
  │
  ├─ Tier 2: Plan-view analysis (multimodal)
  │    → plan_view.png (existing)
  │    → BAML AnalyzePlanView() → SiteAnalysis
  │
  └─ Tier 3: Satellite reconciliation
       → pt-satellite baseline + scan features + plan view
       → BAML ReconcileSiteData() → ReconciledSite
```

## Sample Data

`assets/scans/samples/` — Powell & Market cable car turnaround (SiteScape, 20.5M vertices, binary LE PLY with RGB). Urban environment: trees, transit infrastructure, hardscape, elevation changes.

## Stories

- S-032: Scan Processing CLI & Sample Pipeline
- S-033: Geometric Feature Clustering
- S-034: BAML Feature Classification
- S-035: Multimodal Plan-View Analysis

## Success Criteria

- Process real 20M-point PLY in < 60s
- Cluster obstacles into distinct features with bounding boxes
- BAML classifies features with species/type identification and confidence scores
- Plan-view PNG with feature labels overlaid
- S.1.1 advances integration level (scan → classified features → visual output)
- All BAML calls mockable (ClaudeCliGenerator for dev, MockProposalGenerator pattern for CI)
- `just check` passes
