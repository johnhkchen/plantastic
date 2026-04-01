---
id: T-002-02
story: S-002
title: pt-quote-engine
type: task
status: open
priority: critical
phase: done
depends_on: [T-002-01]
---

## Context

The quote engine is the single biggest time saver in the product. It replaces hours of manual quantity takeoff with instant, accurate computation. Takes a Project with a Tier and material catalog, walks each zone, computes quantities from geometry (via pt-geo), multiplies by unit prices, and produces a Quote with line items.

Pure computation. No I/O. This is tested against known geometries and material configurations with exact expected outputs.

## Acceptance Criteria

- Quote struct: tier name, line_items vec, subtotal, optional tax, total
- LineItem struct: zone_label, material_name, quantity, unit, unit_price, line_total
- Quantity computation per unit type:
  - sq_ft materials: zone area (from pt-geo)
  - cu_yd materials: zone area × depth_inches / 12 / 27
  - linear_ft materials: zone perimeter (from pt-geo)
  - each materials: count of 1 per assignment
- Handles zones with no material assignment (skipped in quote)
- Handles multiple materials per zone (e.g., gravel base + paver surface)
- Subtotal sums all line totals; total = subtotal + tax
- Tests with known polygons: 10×10 ft square patio with $5/sq_ft pavers = $500 line item
- Tests with all three tiers producing different totals for same zones
