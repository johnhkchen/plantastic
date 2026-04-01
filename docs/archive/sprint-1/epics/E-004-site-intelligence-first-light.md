---
id: E-004
title: Site Intelligence First Light
status: open
sprint: 2
---

# E-004: Site Intelligence First Light

## Goal

Build the solar radiance engine, climate data layer, and satellite pre-population pipeline as computation crates. These are pure Rust with no API/UI dependencies — they can be built in parallel with everything else.

This lights up the Site Assessment area on the dashboard for the first time. The engines exist at ★☆☆☆☆ — not yet reachable by users, but proven correct and ready to wire into the API in a future sprint.

## Target

- S.1.3 Sun exposure analysis: — → ★☆☆☆☆ (0.0 → 4.0 effective min)
- S.1.2 Satellite pre-population: — → ★☆☆☆☆ (0.0 → 5.0 effective min)

## Stories

- **S-010**: pt-solar + pt-climate — sun position, radiance grid, frost dates, Sunset zones
- **S-011**: pt-satellite — address → lot boundary + canopy + baseline project

## Success Criteria

- pt-solar computes accurate sun hours for any lat/lng/date (validated against solar-sim prototype values)
- pt-solar produces a radiance grid with correct light categories (full sun / part sun / part shade / full shade)
- pt-climate provides frost dates, Sunset Western Garden zones, and growing season for Bay Area locations
- pt-satellite produces a project baseline from an address (lot polygon, detected trees, sun exposure grid)
- S.1.3 scenario passes at ★☆☆☆☆
- S.1.2 scenario passes at ★☆☆☆☆
