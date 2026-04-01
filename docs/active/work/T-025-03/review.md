# T-025-03 Review: Claim Unclaimed Milestones

## Summary

Corrected milestone attribution in `tests/scenarios/src/progress.rs`. Four
milestones were attributed to the verification tickets (T-025-01, T-025-02)
that tested them end-to-end, rather than the Sprint 1 tickets that originally
built the capabilities. The `delivered_by` fields and notes now credit the
correct tickets.

## Changes

### Files Modified

**`tests/scenarios/src/progress.rs`** — 4 milestone entries updated:

| Milestone | Before | After |
|-----------|--------|-------|
| pt-project: Project/Zone/Tier model + GeoJSON serde | T-025-02 | T-002-01, T-003-02 |
| pt-quote: quantity takeoff engine | T-025-02 | T-002-02 |
| SvelteKit frontend + CF Worker proxy | T-025-02 | T-005-02, T-005-03 |
| pt-tenant: multi-tenant model + auth context | T-025-01 | T-003-02, T-004-02 |

Each note was rewritten to describe what the original delivering tickets built,
following the pattern of other milestones (e.g., "T-015-01 delivered... T-015-02
adds...").

### Files Created

- `docs/active/work/T-025-03/research.md`
- `docs/active/work/T-025-03/design.md`
- `docs/active/work/T-025-03/structure.md`
- `docs/active/work/T-025-03/plan.md`
- `docs/active/work/T-025-03/progress.md`
- `docs/active/work/T-025-03/review.md`

## Scenario Dashboard

| Metric | Before | After |
|--------|--------|-------|
| Effective savings | 76.5 min / 240.0 min (31.9%) | 78.5 min / 240.0 min (32.7%) |
| Milestones | 19 / 24 | 19 / 24 |
| Scenarios passing | 9 | 9 |

The slight improvement in effective savings (76.5 → 78.5) is due to the
dashboard recalculating with updated milestone metadata. No regressions.

## Test Coverage

No new tests needed. This is a metadata-only change to static string data.
The existing scenario dashboard compilation and rendering validates correctness.

## Quality Gate

`just check` passes: format, lint, test, scenarios all green.

## Acceptance Criteria Verification

- [x] Four milestones claimed with accurate notes ✓
- [x] Milestone count at 19/24 ✓ (was already 19/24; attribution corrected)
- [x] `just check` passes ✓

## Open Concerns

**Ticket AC mismatch:** The acceptance criteria states "Milestone count goes
from 15/24 to 19/24", but the count was already 19/24 when this ticket started
because T-025-02 had already claimed the milestones (with wrong attribution).
This ticket corrects the attribution rather than adding new claimed milestones.
The end state matches the intent: all four milestones credited to their original
delivering tickets.

## Known Limitations

None. This is a clean metadata correction with no behavioral changes.
