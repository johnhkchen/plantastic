---
id: S-011
epic: E-004
title: Satellite Pre-population
status: open
priority: medium
dependencies:
  - S-010
---

# S-011: Satellite Pre-population

## Purpose

When a landscaper enters an address, the system pre-fills a project skeleton from satellite and municipal data before anyone visits the site. This eliminates the cold start on every new project.

## Scope

### pt-satellite
- Meta canopy height data: detect trees, estimate height and spread
- Municipal parcel data: lot boundary polygon for SF Bay Area addresses
- Combine with pt-solar: compute baseline sun exposure grid for the lot
- Output: a pre-populated project baseline (lot polygon, detected trees, sun grid)

### API integration
- POST /projects with an address triggers pre-population
- Baseline data stored on project, visible in frontend when project loads

## Tickets

- T-011-01: pt-satellite — canopy + parcel data → project baseline
- T-011-02: Pre-population API route + scenario S.1.2 at ★☆☆☆☆
