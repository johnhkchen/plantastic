---
id: S-022
epic: E-010
title: Polish Rating Framework
status: open
priority: high
tickets: [T-022-01]
---

## Goal

Add a `Polish` enum to the scenario registry alongside the existing `Integration` enum. Update the effective minutes formula to `raw × (integration + polish) / 10`. Update the dashboard to display both dimensions.

## Acceptance Criteria

- `Polish` enum with OneStar through FiveStar (mirrors Integration)
- `ScenarioOutcome::Pass` takes both `Integration` and `Polish`
- Dashboard shows both ratings per scenario (e.g., `PASS ★★★☆☆ int / ★☆☆☆☆ pol`)
- Formula: `effective = raw × (integration + polish) / 10`
- All existing scenario tests updated to pass `Polish::OneStar` as second arg
- `just check` passes
