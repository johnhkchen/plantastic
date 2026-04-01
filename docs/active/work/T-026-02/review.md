# T-026-02 Review: Empty States

## Summary

Created a reusable `EmptyState.svelte` component and integrated it across 5 frontend pages (dashboard, catalog, editor, quote comparison, materials assignment). Each empty state provides contextual guidance with an icon, message, and actionable CTA (button or link). Replaced ad-hoc inline divs with consistent shared component. Advanced 3 scenario polish ratings from TwoStar to ThreeStar.

## Files created (1)

| File | Purpose |
|------|---------|
| `web/src/lib/components/EmptyState.svelte` | Reusable empty state with icon/message/submessage props + action slot |

## Files modified (7)

| File | Change |
|------|--------|
| `web/src/routes/(app)/dashboard/+page.svelte` | Replaced inline empty div → EmptyState + "New Project" button |
| `web/src/routes/(app)/catalog/+page.svelte` | Replaced inline empty div → EmptyState + "Add Material" button |
| `web/src/routes/(app)/project/[id]/editor/+page.svelte` | Added EmptyState in zone sidebar when no zones exist |
| `web/src/lib/components/quote/QuoteComparison.svelte` | Replaced inline empty div → EmptyState + "Go to Materials" link |
| `web/src/routes/(app)/project/[id]/materials/+page.svelte` | Added prerequisite empty states (no zones → editor, no materials → catalog) |
| `tests/scenarios/src/suites/design.rs` | S.2.1: Polish::TwoStar → Polish::ThreeStar |
| `tests/scenarios/src/suites/quoting.rs` | S.3.1, S.3.2: Polish::TwoStar → Polish::ThreeStar (all 4 code paths) |

## Scenario dashboard: before and after

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Effective savings | 76.5 min | 82.5 min | +6.0 min |
| Percentage | 31.9% | 34.4% | +2.5pp |
| Polish debt | 33.0 min | 27.0 min | -6.0 min |
| Passing scenarios | 9 | 9 | — |
| Failed scenarios | 0 | 0 | — |

## Acceptance criteria checklist

- [x] Dashboard (no projects): "Create your first project to get started" + create button
- [x] Catalog (no materials): "Add materials to your catalog before assigning them to zones" + add button
- [x] Zone editor (no zones): "Click on the plan view to draw your first zone" + visual hint
- [x] Quote page (no assignments): "Assign materials to zones to generate quotes" + link to materials
- [ ] Viewer (no scan): Deferred — viewer always loads test scene, no "no scan" condition exists yet
- [x] EmptyState.svelte reusable component with icon, message, action slot
- [x] `just check` passes

## Test coverage

Frontend UI work — no new Rust unit tests. Verification:
1. **Scenario dashboard** — 3 scenarios advanced from Polish::TwoStar to Polish::ThreeStar
2. **`just check`** — all 4 gates pass (fmt, lint, test, scenarios)
3. **No regressions** — `s_3_1_regression` and `s_3_2_regression` still pass

## Open concerns

1. **Viewer empty state deferred**: The acceptance criteria include a viewer empty state ("Upload a LiDAR scan to see a 3D preview"), but the viewer currently always loads `test_scene.glb`. Adding a dead code path for a future `hasScan` signal would be speculative. This should be implemented when pt-scan real scan detection is wired up. Not a blocker for this ticket's value delivery.

2. **Materials page link targets**: The "Go to Zone Editor" link uses a relative `href="editor"` (works because it's a sibling route under `/project/[id]/`). The "Go to Catalog" link uses absolute `href="/catalog"`. Both should work correctly with SvelteKit routing, but the catalog link bypasses `resolve()` — acceptable for a top-level route.

3. **No E2E tests**: Same concern as T-026-01 — frontend component rendering is not tested by the Rust scenario harness. Visual verification is manual until Playwright or similar is added.
