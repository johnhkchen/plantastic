---
id: T-036-01
story: S-036
title: measure-feature-gaps
type: task
status: open
priority: high
phase: implement
depends_on: [T-033-02]
---

## Context

After clustering produces feature candidates with centroids and bounding boxes, we can measure the gaps between them. For the Powell & Market scan, the gap between the two trunk clusters is the planter zone.

## Acceptance Criteria

- `measure_gaps(candidates: &[FeatureCandidate], ground_plane: &GroundPlane) -> Vec<Gap>`
- Gap struct: feature_a_id, feature_b_id, centroid_distance_ft, clear_width_ft (minus radii), clear_length_ft, area_sqft, ground_elevation
- For candidates closer than a configurable threshold (e.g., 30 ft), compute the gap
- clear_width = centroid distance - (spread_a/2 + spread_b/2) — the actual plantable width
- area_sqft = clear_width × clear_length (rectangular approximation)
- Powell & Market validation: should produce 1 gap between the 2 trunks with plausible dimensions
- Unit test: two cylinders 10ft apart, 2ft diameter each → gap width = 6ft

## Implementation Notes

- This is pure geometry — no LLM needed
- The gap measurement is what makes the planter estimation grounded in reality
- For >2 features, compute pairwise gaps and filter by distance threshold
- Ground elevation at the gap midpoint tells us if there's a grade change (drainage consideration)
- The gap area feeds directly into pt-quote for soil volume and plant count calculations
