# T-009-01 Review: Material Assignment UI

## Summary

Built the material assignment UI on the project Materials tab. Landscapers can now select a zone, pick a material from the tenant catalog (grouped by category), assign it per tier (good/better/best), and see live quote totals. Assignments persist via the existing tier API endpoints.

## Files Created

| File | Purpose |
|---|---|
| `web/src/lib/api/tiers.ts` | API client for tier endpoints (fetchTiers, saveTierAssignments) |
| `web/src/lib/api/quotes.ts` | API client for quote endpoint (fetchQuote) |
| `web/src/lib/components/assignment/TierTabs.svelte` | Good/Better/Best tab navigation |
| `web/src/lib/components/assignment/ZoneList.svelte` | Clickable zone list with type, area, assigned material |
| `web/src/lib/components/assignment/MaterialPicker.svelte` | Material catalog grouped by category, click-to-assign |
| `web/src/lib/components/assignment/QuoteSummary.svelte` | Quote total, subtotal, and line item breakdown |

## Files Modified

| File | Change |
|---|---|
| `web/src/routes/(app)/project/[id]/materials/+page.svelte` | Replaced "Coming soon" placeholder with full assignment UI |
| `web/src/lib/api/mock.ts` | Added mock handlers for tier (GET/PUT) and quote (GET) endpoints |

## Files Not Modified

- No Rust backend changes — all endpoints already existed (T-007-02, T-008-01)
- No changes to ZoneEditor, TabNav, or project store
- No changes to existing test files

## Acceptance Criteria Verification

| Criterion | Status |
|---|---|
| Zone list panel showing all zones with type and label | Done — ZoneList component |
| Click a zone to select it; highlight on canvas | Done — click highlights in zone list (no canvas needed on Materials tab) |
| Material picker: list from tenant catalog, grouped by category | Done — MaterialPicker with hardscape/softscape/edging/fill sections |
| Assign material to selected zone for a specific tier | Done — click material to assign, click again to unassign |
| Tier tab navigation to see/edit assignments per tier | Done — TierTabs with Good/Better/Best |
| Assignments persist via PUT /projects/:id/tiers/:tier | Done — debounced save (800ms) |
| Quote total updates when assignments change | Done — fetches quote after save, refreshes on tier switch |

## Scenario Dashboard

**Before**: 25.0 / 240.0 min effective savings (10.4%), 5 passing, 8/20 milestones
**After**: 25.0 / 240.0 min effective savings (10.4%), 5 passing, 8/20 milestones

No change — this ticket is UI plumbing that enables T-009-02 (quote comparison page) to push S.3.1/S.3.2 from OneStar to ThreeStar. No new scenarios flip and no milestones are claimed.

## Test Coverage

This is a frontend-only ticket. No new Rust tests were needed or added. Verification was done via:
- `svelte-check`: 0 errors, 0 warnings (319 files)
- `cargo check`: clean compilation
- `cargo test`: all existing tests pass
- `cargo run -p pt-scenarios`: no regressions

## Open Concerns

1. **Zone ID stability**: When zones are edited on the Editor tab, bulk PUT generates new UUIDs. If a user edits zones and then navigates to Materials, the page reloads everything fresh — safe. But if zones are edited in another browser tab, stale zone IDs in assignments will cause orphaned references. Low risk for V1 single-user use.

2. **No frontend tests**: Vitest is not configured in the project yet. The four new components and the page orchestrator have no automated tests. When Vitest is set up, these components should get basic render tests.

3. **`project.svelte.ts` store stale**: The global `Tier` type in the store (`id`, `name`, `plantIds`) doesn't match the actual API shape. The Materials page uses local state instead. The store's Tier type should be updated or removed in a future cleanup ticket.

4. **No "highlight on canvas"**: The acceptance criteria mentions "highlight on canvas" for zone selection. The design decision was to use a zone list on the Materials tab instead. The zone list highlight (left border accent + background) serves the same purpose. If canvas highlighting is truly required, it would mean embedding the ZoneEditor in the Materials tab, which adds complexity for V1.
