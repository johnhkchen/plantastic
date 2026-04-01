---
id: T-010-01
story: S-010
title: pt-solar-engine
type: task
status: open
priority: high
phase: done
depends_on: []
---

## Context

Port the solar radiance calculation from the solar-sim prototype (TypeScript/SunCalc) into Rust. The algorithms are proven — 5-minute sampling, sub-1% error at classification boundaries, ~2ms for a full year at a single point. This is a translation and validation task, not a research task.

Prior art: /Volumes/ext1/swe/repos/solar-sim/src/lib/solar/ (position.ts, sun-hours.ts, shade.ts)

## Acceptance Criteria

- Sun position (altitude, azimuth) for any lat/lng/datetime — validated against SunCalc reference values
- Daily sun hours: integrate sun altitude > 0 across day at 5-min intervals
- Annual sun hours: daily aggregation across a date range
- Light category classification: full sun (6+), part sun (4-6), part shade (2-4), full shade (<2)
- Radiance grid: compute sun hours across a grid of lat/lng points
- Performance: full year single point < 5ms, residential grid < 500ms
- Pure computation, no I/O
- Tests validated against known solar-sim outputs for SF Bay Area locations
- S.1.3 scenario registered and passing at ★☆☆☆☆
- Claim milestone: "pt-solar: sun position + radiance grid"
