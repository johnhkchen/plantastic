---
id: T-011-01
story: S-011
title: pt-satellite-baseline
type: task
status: open
priority: medium
phase: done
depends_on: [T-010-01, T-010-02]
---

## Context

Given an address, produce a pre-populated project baseline from satellite and municipal data. This combines Meta canopy height data, SF gov parcel data, and pt-solar's radiance grid to give the landscaper a head start before the site visit.

Prior art exists in the satellite/LiDAR prototype that proved these data sources are viable.

## Acceptance Criteria

- Address → geocoded lat/lng (geocoding API or embedded for known test addresses)
- Lot boundary polygon from municipal parcel data (SF Bay Area)
- Tree detection from Meta canopy height data: location, estimated height, estimated spread
- Baseline sun exposure grid using pt-solar across the lot polygon
- Output: a ProjectBaseline struct containing lot polygon, detected trees, sun grid reference
- Integration test with known SF address producing plausible baseline
- S.1.2 scenario registered and passing at ★☆☆☆☆
- Claim milestone: "pt-satellite: address → lot + canopy + sun baseline"
