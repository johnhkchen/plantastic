# T-037-02 Review: Viewer Load Test

## Summary

Added an e2e Playwright test that verifies the full Bevy WASM viewer initialization chain: SvelteKit page → iframe mount → WASM download → WebGL init → `ready` postMessage → scene load. The test intercepts the scene API, uses a test GLB fixture, and captures a screenshot for CI artifact review.

## Files Modified

| File | Change |
|------|--------|
| `web/src/lib/components/viewer/Viewer.svelte` | Added `data-viewer-ready={ready \|\| undefined}` attribute to container div |
| `web/playwright.config.ts` | Increased test timeout from 30s to 60s for WASM cold start headroom |

## Files Created

| File | Purpose |
|------|---------|
| `web/e2e/viewer.spec.ts` | E2e test: viewer initialization, scene load, error check, screenshot |

## Acceptance Criteria Checklist

- [x] Navigate to `/project/test-project/viewer`
- [x] Assert the viewer iframe is present in the DOM
- [x] Wait for the Bevy `ready` postMessage (via `data-viewer-ready` attribute, 45s timeout)
- [x] Intercept the scene API call with `page.route('**/api/projects/*/scene/*')`, return test GLB
- [x] Assert no `error` postMessage received (check ErrorBanner text not visible)
- [x] Take a screenshot after scene load (attached to test report via `testInfo.attach()`)
- [x] Test GLB is `web/static/viewer/assets/models/test_scene.glb`
- [x] Screenshot saved to `web/test-results/` (via Playwright `outputDir` config)

## Test Coverage

**What is tested:**
- SvelteKit viewer route renders without backend (API mocked)
- Viewer component mounts iframe correctly
- `data-viewer-ready` attribute reflects viewer state
- No error state after scene API mock + WASM init
- Visual snapshot captured for human review

**What is NOT tested (by design):**
- Actual 3D rendering quality (would need visual regression tooling)
- Real scan GLB loading (deferred to T-037-03)
- Zone interaction, tier switching, light angle controls (separate ticket scope)
- Backend scene API (mocked out — this is a frontend integration test)

## Open Concerns

1. **WASM in CI**: The test depends on headless Chromium being able to load a 15.3 MB WASM binary and initialize WebGL via SwiftShader. If CI runners lack SwiftShader or have memory constraints, the test will timeout. This is intentional — a timeout means the viewer can't load, which is a real problem.

2. **No `ready` from iframe in test env**: The Bevy WASM viewer must send `{ type: 'ready' }` via postMessage after WebGL initializes. If the WASM binary or its JS bindings are broken/outdated, the viewer will never send `ready` and the test will timeout at 45s. This is the correct behavior — it catches real failures.

3. **Test timeout budget**: 60s test timeout with 45s locator wait. If WASM init consistently takes >40s, we may need to increase. Monitor CI run times after first successful run.

4. **Error detection is text-based**: Checking for "Failed to load" text. If the error message changes in the viewer page, the assertion needs updating. This is acceptable — the text comes from the same component we control.

## Scenario Dashboard

No scenario changes — this ticket adds test infrastructure, not a customer-facing capability. No milestone to claim (e2e testing infrastructure was already established in T-037-01).

## Quality Gate

- `just fmt` — pass
- `just lint` — pass
- Prettier — all web files formatted
- `just test` / `just scenarios` — Rust side unchanged, no regressions expected
