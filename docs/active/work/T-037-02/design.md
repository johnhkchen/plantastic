# T-037-02 Design: Viewer Load Test

## Problem

Verify the full Bevy WASM viewer initialization chain in a headless browser:
SvelteKit page → iframe mount → WASM download → WebGL init → `ready` postMessage → scene load.

## Options Evaluated

### Option A: Listen for postMessage via page.evaluate()

Set up a `window.addEventListener('message')` in the page context before navigation, store the `ready` event in a window variable, then poll for it.

```ts
await page.evaluate(() => {
  window.__viewerReady = false;
  window.addEventListener('message', (e) => {
    if (e.data?.type === 'ready') window.__viewerReady = true;
  });
});
// ... navigate ...
await page.waitForFunction(() => window.__viewerReady, { timeout: 30000 });
```

**Pros**: No source changes, tests only.
**Cons**: Race condition — if navigation triggers page reload, the evaluate runs too late. Must set up listener before `goto()` but `goto()` may reset the page context. Fragile ordering.

### Option B: Add `data-viewer-ready` attribute to Viewer.svelte

Add a data attribute to the Viewer container div that reflects the `ready` state. Test waits for the attribute with standard Playwright locator.

```svelte
<div class="..." data-viewer-ready={ready || undefined}>
```

```ts
await page.locator('[data-viewer-ready]').waitFor({ timeout: 30000 });
```

**Pros**: Clean, idiomatic Playwright. No race conditions. Reusable for any future test. Trivial source change (one attribute).
**Cons**: Modifies production component (minimal — one data attribute).

### Option C: Wait for loading overlay to disappear

The Viewer shows "Loading viewer..." until `ready`. Test could wait for that overlay to disappear.

```ts
await expect(page.getByText('Loading viewer...')).not.toBeVisible({ timeout: 30000 });
```

**Pros**: No source changes. Tests observable behavior.
**Cons**: Indirect — overlay could disappear for reasons other than `ready` (error, timeout). Doesn't positively confirm the viewer initialized.

## Decision: Option B — `data-viewer-ready` attribute

**Rationale**:
1. Most reliable signal — directly tied to the `ready` state variable
2. No race conditions — Playwright's `waitFor()` handles timing automatically
3. Minimal source change — one `data-viewer-ready` attribute on the container div
4. Reusable — any future e2e test can check viewer readiness the same way
5. The ticket explicitly suggests this approach

## Test Strategy

### API Mocking
- Intercept `/api/projects/*/scene/*` with `page.route()` before navigation
- Return `{ url: '/viewer/assets/models/test_scene.glb', metadata: { zone_count: 1, triangle_count: 100, tier: 'good' } }`
- This lets the page render the Viewer component instead of showing an error

### Viewer Ready Detection
- Navigate to `/project/test-project/viewer`
- Wait for `[data-viewer-ready]` with extended timeout (30s for WASM cold start)
- This confirms: iframe loaded → WASM initialized → `ready` postMessage received → state updated

### Error Detection
- After scene load, evaluate `window` for any error postMessages
- Alternative: add `data-viewer-error` attribute to Viewer.svelte for error state, check it's absent
- Simpler: just check that no ErrorBanner is visible on the page

### Screenshot
- Take screenshot after viewer is ready (or after a short delay for rendering)
- Save to `web/test-results/viewer-loaded.png`
- Attach to test report via `testInfo.attach()`

### Timeout Handling
- The 30s default test timeout in playwright.config.ts matches the WASM cold start budget
- The `waitFor` on `[data-viewer-ready]` will use the test timeout
- If WebGL/WASM fails entirely in CI, the test times out — that's correct behavior (a real failure)
- We do NOT want to skip gracefully on WebGL failure — that would hide real problems

### Viewer.svelte Changes
Add two data attributes:
- `data-viewer-ready` — present when `ready` is true (signals successful WASM init)
- `data-viewer-error` — present when there's an error message (makes error detection clean)

Both are passive data attributes with no visual or behavioral impact.

## What Was Rejected

- **Option A** rejected because of race conditions between `page.evaluate()` and navigation
- **Option C** rejected because it's an indirect signal that could give false positives
- **Graceful skip on WebGL failure** rejected — CI must support WebGL (SwiftShader), and silent skips hide real problems
- **Separate test file per concern** rejected — one `viewer.spec.ts` file with a single focused test is sufficient for this ticket
