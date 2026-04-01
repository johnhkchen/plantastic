---
id: T-010-02
story: S-010
title: pt-climate-engine
type: task
status: open
priority: medium
phase: done
depends_on: []
---

## Context

Climate data layer: frost dates, Sunset Western Garden zones, USDA hardiness zones, growing season computation. These feed into plant recommendations and site assessment. Prior art exists in solar-sim (src/lib/climate/).

## Acceptance Criteria

- Frost date lookup (first fall, last spring) for Bay Area locations
- Sunset Western Garden zone lookup by lat/lng
- USDA hardiness zone lookup by lat/lng
- Growing season computation: days between last spring frost and first fall frost
- Data can be embedded (static lookup tables for V1) or fetched — prefer embedded for speed
- Pure computation, no network I/O at query time
- Tests for known Bay Area locations (SF, Oakland, San Jose — different microclimates)
- Claim milestone: "pt-climate" if not already represented
