---
id: T-035-01
story: S-035
title: baml-analyze-plan-view
type: task
status: open
priority: medium
phase: research
depends_on: [T-034-02]
---

## Context

Claude is multimodal — it can analyze the plan-view PNG directly. Combined with the address and classified features, it produces site-level analysis: identifying hardscape boundaries, lawn areas, suggesting zone placements.

## Acceptance Criteria

- Add to `baml_src/analyze.baml`:
  - AnalyzePlanView function: image (base64) + lot_dimensions + address + classified_features → SiteAnalysis
  - SiteAnalysis: features[], suggested_zones[], site_observations[]
  - Each suggested zone: label, zone_type, rationale, approximate_area_sqft
- Three generator implementations (same trait pattern)
- Run on Powell & Market annotated plan view, capture fixture
- Prompt emphasizes spatial reasoning: "the open area between Feature 3 and Feature 7 would work well as a patio zone"
- `just check` passes

## Implementation Notes

- BAML supports image inputs via base64 encoding in the prompt
- The image should be the annotated plan view (with feature labels) for richer context
- Site observations are free-form insights: "the two London Planes create a natural shade corridor", "the grade change near the curb suggests drainage consideration"
- Suggested zones are the system proactively helping the landscaper — not just analyzing, but recommending
- This is where the product shifts from "scan processor" to "AI landscape designer"
