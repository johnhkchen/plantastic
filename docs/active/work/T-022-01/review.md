# T-022-01 Review: Polish Enum + Dashboard

## Summary

Added `Polish` as a second quality dimension to the scenario registry,
alongside the existing `Integration` dimension. The dashboard now shows
both ratings per scenario and a new formula that weights them equally.

## Files Modified

| File | Change |
|------|--------|
| `tests/scenarios/src/registry.rs` | Added `Polish` enum, changed `Pass(Integration)` → `Pass(Integration, Polish)`, updated formula and display |
| `tests/scenarios/src/report.rs` | Added polish debt, formula, and legend to dashboard header; fixed wildcard pattern |
| `tests/scenarios/src/suites/site_assessment.rs` | 3 Pass sites → added `Polish::OneStar` |
| `tests/scenarios/src/suites/design.rs` | 3 Pass sites → added `Polish::OneStar` |
| `tests/scenarios/src/suites/quoting.rs` | 4 Pass sites → added `Polish::OneStar`; fixed wildcard patterns |

**No files created or deleted.**

## Acceptance Criteria Verification

- [x] `Polish` enum added to `registry.rs` mirroring `Integration` (OneStar–FiveStar)
- [x] `ScenarioOutcome::Pass(Integration, Polish)` — two-arg tuple
- [x] `effective_minutes()` uses `raw × (integration.stars() + polish.stars()) / 10.0`
- [x] `status_label()` shows both: `PASS ★★★☆☆ / ★☆☆☆☆`
- [x] Dashboard displays polish alongside integration
- [x] All 10 passing scenario returns include `Polish::OneStar`
- [x] All suite files updated: site_assessment, design, quoting (crew_handoff and infrastructure have no Pass sites)
- [x] Polish debt shown in dashboard header
- [x] Formula and legend added to dashboard header
- [x] `just check` passes

## Dashboard Before/After

| Metric | Before | After |
|--------|--------|-------|
| Effective savings | 58.0 min (24.2%) | 44.5 min (18.5%) |
| Raw passing | 155.0 min | 155.0 min (unchanged) |
| Polish debt | N/A | 62.0 min |
| Pass/Fail | 8/0 | 8/0 (unchanged) |

The effective savings decrease is expected and correct. The new formula
reveals "polish debt" — the gap between current polish levels and the
maximum. No scenarios regressed (all 8 still pass, 0 failures).

## Test Coverage

No new unit tests added. The scenario harness itself is the test — running
`cargo run -p pt-scenarios` exercises every code path through the `Polish`
enum, the updated formula, the display format, and the dashboard rendering.
All 10 Pass sites are exercised. The polish debt calculation is verified by
the dashboard output (62.0 min matches manual calculation).

## Open Concerns

1. **Integration::weight() is now dead code.** Suppressed with `#[allow(dead_code)]`.
   It's still a useful utility method if anyone needs the single-dimension weight.
   Could be removed if it stays unused after several sprints.

2. **Dashboard width.** The dual star display (`PASS ★★★☆☆ / ★☆☆☆☆`) is wider
   than the old single display. Terminal widths ≥80 columns are fine. Narrower
   terminals may wrap — acceptable for a developer dashboard.

3. **Polish levels are all OneStar.** This is correct for now — no UX polish
   evaluation has been done. Future tickets should bump polish levels as UX
   work lands, creating visible progress on the dashboard.
