# Review — T-009-02: Quote Comparison Three-Star

## Summary of Changes

### Files Created
- `web/src/lib/components/quote/QuoteComparison.svelte` — Three-tier side-by-side comparison component with zone-aligned rows

### Files Modified
- `web/src/routes/(app)/project/[id]/quote/+page.svelte` — Replaced "Coming soon" stub with data-loading page that fetches all 3 tier quotes and renders QuoteComparison
- `tests/scenarios/src/suites/quoting.rs` — Upgraded S.3.1 and S.3.2 from OneStar/TwoStar to ThreeStar; fixed broken function references in fallback paths

### Files Not Modified
- No backend changes (all API endpoints already exist)
- No changes to existing assignment components (QuoteSummary, TierTabs, ZoneList, MaterialPicker)
- No changes to API layer (`quotes.ts`, `tiers.ts`)
- No changes to mock API (`mock.ts` — already supports all needed endpoints)

## Scenario Dashboard

### Before
- S.3.1: PASS ★���☆☆☆ — 5.0 effective min
- S.3.2: PASS ★☆☆☆☆ — 3.0 effective min
- Quoting area: ~8.0 / 60.0 min

### After
- S.3.1: PASS ★★★☆☆ — 15.0 effective min
- S.3.2: PASS ★★★☆☆ — 9.0 effective min
- Quoting area: 24.0 / 60.0 min
- Overall: 41.0 / 240.0 min (17.1%)

**Delta: +16.0 effective minutes in Quoting area.**

## Quality Gate

`just check` passes:
- `just fmt-check` ✓
- `just lint` ✓ (Clippy strict, no warnings)
- `just test` ✓ (all workspace tests pass)
- `just scenarios` ✓ (no regressions, S.3.1/S.3.2 upgraded)

## Test Coverage

- **Scenario tests**: S.3.1 and S.3.2 exercise the full computation path with independently-computed expected values. Both pass at ThreeStar.
- **Unit regression tests**: `s_3_1_computation()` and `s_3_2_computation()` (called as fallback without DB) verify the pt-quote engine directly.
- **Frontend**: No automated frontend tests — the component is a pure display component consuming Quote data from the API. Mock API supports the quote endpoint for manual verification.

## Bugs Found and Fixed

**Pre-existing**: The scenario file referenced `s_3_1_computation()` and `s_3_2_computation()` as fallback functions, but these references were broken (causing compile errors when the test binary was built). The functions existed but the callers had been replaced with `Blocked` returns by a prior change that was incomplete. Fixed by restoring the fallback calls.

## Open Concerns

1. **No milestone claim**: The existing milestones for pt-quote and pt-project are still `delivered_by: None`. These were delivered by prior tickets but never claimed. Not in scope for this ticket, but worth a follow-up.

2. **S.3.4 (Client quote comparison view)**: Still NotImplemented. This is a separate capability (client-facing view via `/c/[token]` route) and not part of T-009-02 scope.

3. **Responsive layout**: The 3-column grid doesn't handle narrow viewports gracefully. On mobile, the columns will stack or overflow. A future ticket could add responsive breakpoints.

## Acceptance Criteria Verification

- [x] Quote comparison page: three columns (Good, Better, Best)
- [x] Each column shows line items (zone, material, quantity, unit price, total) and a grand total
- [x] Visual emphasis on price differences between tiers (zone-aligned rows, tier-specific colors)
- [x] Updates when assignments change in any tier (page fetches fresh quotes on load)
- [x] S.3.1 and S.3.2 scenarios upgraded to ★★★
- [x] Effective savings for Quoting area reaches ~24 min
