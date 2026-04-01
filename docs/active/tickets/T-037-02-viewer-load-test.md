---
id: T-037-02
story: S-037
title: viewer-load-test
type: task
status: open
priority: high
phase: done
depends_on: [T-037-01]
---

## Context

The critical e2e test: does the Bevy WASM viewer actually initialize in a browser? This verifies the full chain: SvelteKit page → iframe mount → WASM download → WebGL init → `ready` postMessage → scene load.

## Acceptance Criteria

- `web/e2e/viewer.spec.ts`:
  1. Navigate to `/project/test-project/viewer`
  2. Assert the viewer iframe is present in the DOM
  3. Wait for the Bevy `ready` postMessage (up to 30s for WASM cold start)
  4. Intercept the scene API call with `page.route()`, return the test GLB URL
  5. Assert no `error` postMessage received
  6. Take a screenshot after scene load (visual proof for CI artifacts)
- The test GLB is the existing `web/static/viewer/assets/models/test_scene.glb`
- Mock the API: `page.route('**/projects/*/scene/*', ...)` returns fixture URL
- Test passes in headless Chromium (SwiftShader WebGL)
- Screenshot saved to `web/test-results/` for review

## Implementation Notes

- The iframe is at `iframe[src="/viewer/index.html"]` — use `page.frameLocator()` or `page.frame()`
- The `ready` message comes via `window.addEventListener('message')` — Playwright can listen with `page.evaluate()` or `page.waitForEvent('console')` if the host logs it
- Alternative: add a `data-viewer-ready` attribute to the Viewer.svelte component when it receives `ready`, then `await page.locator('[data-viewer-ready]').waitFor()`
- SwiftShader is a software WebGL renderer — it's slow (10-20s init) but works in headless CI
- If WebGL fails entirely in CI, the test should detect the `error` postMessage and skip gracefully, not hang
