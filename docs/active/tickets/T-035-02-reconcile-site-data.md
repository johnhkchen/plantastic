---
id: T-035-02
story: S-035
title: reconcile-site-data
type: task
status: open
priority: medium
phase: done
depends_on: [T-035-01]
---

## Context

The ultimate intelligence layer: merge scan features, satellite baseline, and visual analysis into a reconciled site model. Identifies confirmed features (seen in both scan and satellite), new features (scan-only), removals (satellite-only), and proactive zone recommendations.

## Acceptance Criteria

- Add to `baml_src/reconcile.baml`:
  - ReconcileSiteData function: scan_features + satellite_baseline + plan_view_analysis → ReconciledSite
  - ReconciledSite: confirmed_features[], scan_only_features[], satellite_only_features[], discrepancies[], recommended_zones[]
  - Each discrepancy: description, possible_explanation, design_implication
- Three generator implementations (same trait pattern)
- Prompt reasons about temporal changes: "satellite shows a tree here but scan doesn't — was it removed recently?"
- Recommended zones informed by all data sources: sun exposure, existing features, grade, access
- Mock fixture from reconciliation of Powell & Market scan + satellite data
- `just check` passes

## Implementation Notes

- This function sees the most context of any BAML call — keep the prompt focused
- Discrepancy detection is high-value: it surfaces changes the landscaper didn't know about
- Recommended zones should reference specific features by label: "Zone between London Plane #1 and transit shelter would receive 4-6 hours sun (part shade), ideal for shade-tolerant groundcover"
- This is the capstone of the scan intelligence pipeline — it's where all the data sources converge into actionable design guidance
- Depends on pt-satellite having real data for the location (currently hardcoded SF addresses only)
