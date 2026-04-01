---
id: S-010
epic: E-004
title: Solar & Climate Engine
status: open
priority: high
---

# S-010: Solar & Climate Engine

## Purpose

Port the proven solar radiance and climate algorithms from the solar-sim prototype into Rust. These are pure computation crates with no I/O — the foundation for plant recommendations and site assessment.

## Scope

### pt-solar
- Sun position calculation (altitude, azimuth) for any lat/lng/datetime
- Daily sun hours integration (5-minute sampling intervals, proven accuracy from solar-sim)
- Radiance grid: compute sun hours across a grid of points covering a property
- Light category classification: full sun (6+ hrs), part sun (4-6), part shade (2-4), full shade (<2)
- Shadow modeling: terrain + obstacle shadow intersection (simplified for V1 — full terrain conformance later)

### pt-climate
- Frost dates (first/last) by location
- Sunset Western Garden zone lookup
- Growing season computation (last frost → first frost)
- Hardiness zone (USDA) lookup

Both crates are pure functions, no I/O, extensively tested. Validated against known solar-sim outputs.

## Tickets

- T-010-01: pt-solar — sun position + daily hours + radiance grid
- T-010-02: pt-climate — frost dates, Sunset zones, growing season
