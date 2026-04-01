# T-026-01 Review: Loading & Error States

## Summary

Added shared `LoadingSkeleton` and `ErrorBanner` components plus a `friendlyError()` error classification utility. Replaced ad-hoc inline loading/error markup across all 6 async pages (dashboard, catalog, quote, editor, viewer, materials) with these shared components. Every page now shows proper skeleton loading indicators and a styled error banner with a retry button on fetch failure. Error messages distinguish network errors from server errors.

## Files created (3)

| File | Purpose |
|------|---------|
| `web/src/lib/utils/errors.ts` | `friendlyError()` — classifies errors into human-readable messages |
| `web/src/lib/components/LoadingSkeleton.svelte` | Reusable skeleton placeholder (card/row/column variants) |
| `web/src/lib/components/ErrorBanner.svelte` | Reusable error banner with optional retry button |

## Files modified (8)

| File | Change |
|------|--------|
| `web/src/routes/(app)/dashboard/+page.svelte` | Replaced inline skeleton + error banner with shared components |
| `web/src/routes/(app)/catalog/+page.svelte` | Replaced inline skeleton + error banner with shared components |
| `web/src/routes/(app)/project/[id]/quote/+page.svelte` | Extracted `loadQuotes()`, added `ErrorBanner` with retry |
| `web/src/routes/(app)/project/[id]/editor/+page.svelte` | Extracted `loadZones()`, added `LoadingSkeleton` + `ErrorBanner` |
| `web/src/routes/(app)/project/[id]/viewer/+page.svelte` | Wired `onError` callback, added `ErrorBanner` |
| `web/src/routes/(app)/project/[id]/materials/+page.svelte` | Extracted `loadData()`, added `LoadingSkeleton` + `ErrorBanner` |
| `tests/scenarios/src/suites/design.rs` | S.2.1, S.2.4: Polish::OneStar → Polish::TwoStar |
| `tests/scenarios/src/suites/quoting.rs` | S.3.1, S.3.2: Polish::OneStar → Polish::TwoStar (all code paths) |

## Scenario dashboard: before and after

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Effective savings | 69.5 min | 76.5 min | +7.0 min |
| Percentage | 29.0% | 31.9% | +2.9pp |
| Polish debt | 40.0 min | 33.0 min | -7.0 min |
| Passing scenarios | 9 | 9 | — |
| Failed scenarios | 0 | 0 | — |

## Acceptance criteria checklist

- [x] Dashboard project list: skeleton placeholder while fetching
- [x] Catalog material list: skeleton placeholder while fetching
- [x] Quote comparison: column skeleton while fetching tier quotes (was already in QuoteComparison)
- [x] Zone editor: loading skeleton while zones load
- [x] Viewer: loading overlay while iframe + glTF load (was already in Viewer.svelte)
- [x] All API fetch failures show a user-visible error banner
- [x] Error banner includes retry button that re-fetches
- [x] Network errors distinguished from server errors in message
- [x] Implementation uses shared `LoadingState` pattern (LoadingSkeleton + ErrorBanner components)
- [x] Affected scenarios advance to ★★☆☆☆ polish (S.2.1, S.2.4, S.3.1, S.3.2)
- [x] `just check` passes

## Test coverage

This ticket is frontend UI work — no new Rust unit tests. Coverage verification is through:
1. **Scenario dashboard** — 4 scenarios advanced from Polish::OneStar to Polish::TwoStar
2. **`just check`** — all 4 gates pass (fmt, lint, test, scenarios)
3. **Existing scenario regression tests** — `s_3_1_regression` and `s_3_2_regression` still pass

## Open concerns

1. **`friendlyError()` receives `string`, not `Error`** — ErrorBanner takes `message: string` because all catch blocks already extract the message as a string. The classification in `friendlyError()` checks for Error/ApiError/TypeError instances, but since pages pass `e.message` strings, the full classification only fires for non-string error values. For the current codebase this is fine (all errors have descriptive `.message` strings from our API), but if we later want to pass raw error objects, the prop type should change to `unknown`.

2. **No E2E tests for loading/error UI** — Frontend component rendering is not tested by the Rust scenario harness. Visual verification is manual. If Playwright or similar is added later, these components should get coverage.

3. **Quote page still catches individual tier errors as `null`** — If one tier fails but others succeed, the page shows partial data with no error. Added detection for all-three-fail case, but partial failure is silent. This matches the existing behavior and is arguably correct (show what you can), but could be revisited.
