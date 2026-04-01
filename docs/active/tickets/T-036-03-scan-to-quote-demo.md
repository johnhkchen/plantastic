---
id: T-036-03
story: S-036
title: scan-to-quote-demo
type: task
status: open
priority: high
phase: done
depends_on: [T-036-02, T-032-03]
---

## Context

The capstone: run the entire pipeline on the Powell & Market scan and produce a three-tier quote for planters between the two trees. This proves the full product loop: scan → detect → measure → design → quote.

## Acceptance Criteria

- CLI example or `just` recipe: `just scan-to-quote <ply-path>`
- Full pipeline:
  1. process_scan() → ClassifiedCloud
  2. cluster obstacles → 2 clusters (trunks)
  3. classify features (BAML or mock) → 2 tree trunks
  4. measure gap → plantable area in sqft
  5. estimate planters (BAML or mock) → 3 style options with plant lists
  6. feed each style into pt-quote → 3 Quote objects with line items and totals
  7. print the three-tier quote summary
- Output includes:
  - Site summary: "2 tree trunks detected, X sqft plantable area between"
  - Per tier: style name, plant list with quantities, soil volume, total cost
  - Viewer-ready: terrain GLB written for Bevy loading
- All LLM calls use ClaudeCliGenerator (subscription) or mock
- This is the demo script — should complete in < 2 minutes including LLM calls
- Should work with mock generators in < 10 seconds (CI-friendly)

## Implementation Notes

- This is a stitching ticket — it calls code from pt-scan, pt-features/clustering, pt-proposal/BAML, pt-quote, pt-scene
- The output should read like a real landscaping proposal:
  ```
  POWELL & MARKET PLANTER — SCAN ANALYSIS
  Site: 2 tree trunks, 8.2 ft gap, 41 sqft plantable area

  GOOD — Low-Maintenance Succulents
    27 × Echeveria 'Lola' (4" spacing)     $324.00
    2.5 cu yd planting soil                 $112.50
    Total: $436.50

  BETTER — Ornamental Grasses
    20 × Carex praegracilis (6" spacing)    $300.00
    3.0 cu yd planting soil                 $135.00
    Total: $435.00

  BEST — Seasonal Color Display
    55 × Mixed annuals (4" spacing)         $440.00
    3.0 cu yd planting soil                 $135.00
    Steel edging, 26 linear ft              $84.50
    Total: $659.50
  ```
- The numbers must come from pt-quote computation, not LLM output
- This demo script is what we show investors, put in the README, and record for the landing page
