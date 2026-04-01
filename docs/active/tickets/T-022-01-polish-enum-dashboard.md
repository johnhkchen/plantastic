---
id: T-022-01
story: S-022
title: polish-enum-dashboard
type: task
status: open
priority: high
phase: open
depends_on: []
---

## Context

The scenario registry currently has a single `Integration` dimension. This ticket adds `Polish` as a second dimension and updates the formula, dashboard display, and all existing scenario return values.

## Acceptance Criteria

- Add `Polish` enum to `registry.rs` mirroring `Integration` (OneStar–FiveStar)
- `ScenarioOutcome::Pass(Integration, Polish)` — two-arg tuple
- Update `effective_minutes()`: `raw × (integration.stars() + polish.stars()) as f64 / 10.0`
- Update `status_label()` to show both: `PASS ★★★☆☆ / ★☆☆☆☆`
- Update `report.rs` dashboard to display polish alongside integration
- Update all 8 passing scenario `test_fn` returns to include `Polish::OneStar`
- Update all suite files: `site_assessment.rs`, `design.rs`, `quoting.rs`, `crew_handoff.rs`, `infrastructure.rs`
- Add polish level descriptions to dashboard legend or header
- `just check` passes

## Implementation Notes

- The `Pass` variant changes from `Pass(Integration)` to `Pass(Integration, Polish)` — this is a breaking change to the enum, so every `match` arm touching `Pass` needs updating
- Keep `Polish` labels distinct from `Integration` in display — suggest `int` / `pol` suffix
- The dashboard header should show the new formula so readers understand the weighting
- Consider showing the "polish debt" (gap between current and potential) in the dashboard summary

## Files to Modify

- `tests/scenarios/src/registry.rs` — add `Polish` enum, update `ScenarioOutcome::Pass`
- `tests/scenarios/src/report.rs` — update dashboard rendering
- `tests/scenarios/src/suites/*.rs` — update all `Pass(Integration::X)` to `Pass(Integration::X, Polish::OneStar)`
