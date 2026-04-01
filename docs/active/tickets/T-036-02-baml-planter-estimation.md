---
id: T-036-02
story: S-036
title: baml-planter-estimation
type: task
status: open
priority: high
phase: research
depends_on: [T-036-01, T-034-01]
---

## Context

Given a measured gap between classified features, BAML suggests planter styles with specific plant selections. The LLM handles the *what* (which plants, why they work here) while code handles the *how much* (quantities from spacing rules, soil volume from geometry).

## Acceptance Criteria

- Add to `baml_src/planter.baml`:
  - EstimatePlanter function:
    - Input: gap dimensions (width_ft, length_ft, area_sqft), adjacent features (labels), sun_hours, climate_zone, address
    - Output: PlanterEstimate with 3 style options
  - PlanterStyle: style_name, description, plant_selections[], soil_depth_inches, design_rationale
  - PlantSelection: common_name, botanical_name, spacing_inches, why_this_plant
- Code computes (NOT the LLM):
  - plant_count = floor(area_sqft / (spacing_inches/12)²)
  - soil_volume_cuyd = area_sqft × depth_inches / (12 × 27)
  - estimated_cost = plant_count × unit_price + soil_volume × soil_price
- LLM picks plants appropriate for:
  - The specific sun/shade conditions (between two trunks = partial shade)
  - The climate zone (SF: USDA 10b, Sunset 17, maritime)
  - The urban context (foot traffic tolerance, maintenance level, visual impact)
- Three generator implementations (same pattern: Baml, ClaudeCli, Mock)
- Mock fixture from real estimation of Powell & Market gap
- `just check` passes

## Implementation Notes

- The LLM choosing plants while code does math is the key discipline — same as proposal (LLM writes narrative, pt-quote computes totals)
- Default plant pricing for estimates: $15/gal (grasses), $8/4" (annuals), $12/gal (succulents), $45/cuyd (soil)
- The 3 styles should feel like real landscape design options, not generic filler
- Powell & Market specifics: partial shade (trunk canopy absent but adjacent buildings), high foot traffic, SF Parks & Rec style guidelines for Market Street streetscape
- This is where BAML earns its keep — the plant selection reasoning is the value a human landscape designer provides, and we're encoding it in a structured prompt
