---
id: S-036
epic: E-014
title: BAML Planter Estimation
status: open
priority: high
depends_on: [S-033, S-034]
tickets: [T-036-01, T-036-02, T-036-03]
---

## Goal

Given classified scan features and the measured gap between them, use BAML to suggest planter zones and estimate plant count, soil volume, and material cost for multiple styles. This completes the scan → measure → design → quote loop.

## The Proof

Two tree trunks on Powell & Market, brick path between. The system should:
1. Measure the gap (centroid-to-centroid minus trunk radii)
2. Suggest a planter zone sized to the available space
3. Estimate three planter styles with plant counts, soil volume, and rough cost
4. Feed the estimates into pt-quote for proper three-tier pricing

## Acceptance Criteria

- BAML EstimatePlanter: takes gap dimensions + sun exposure + climate zone → planter options
- Each option: style name, plant list with quantities, soil volume (cu yd), estimated cost
- Plant quantities derived from standard spacing rules (not hallucinated):
  - Ornamental grasses: 1 per 2 sqft
  - Seasonal annuals: 1 per 0.75 sqft (4" spacing)
  - Succulents: 1 per 1.5 sqft
- Soil volume computed from zone area × planter depth (not LLM-computed)
- The LLM chooses *which* plants and *why*, the math computes *how many*
- Results feed into existing pt-quote engine for proper pricing
- Mock fixture from real estimation of the Powell & Market gap
