---
id: S-035
epic: E-014
title: Multimodal Plan-View Analysis
status: open
priority: medium
depends_on: [S-034]
tickets: [T-035-01, T-035-02]
---

## Goal

Feed the plan-view PNG to Claude's vision capability via BAML for site-level analysis. Identifies hardscape boundaries, canopy areas, and suggests zone placements. Cross-references with scan features and satellite baseline.

## Acceptance Criteria

- BAML schema: AnalyzePlanView function with image + lot dimensions + address
- Returns SiteAnalysis with features, suggested zones, and observations
- BAML schema: ReconcileSiteData function merging scan + satellite + visual
- Annotated plan-view PNG with classified feature labels overlaid
- Mock fixtures from real analysis of Powell & Market plan view
