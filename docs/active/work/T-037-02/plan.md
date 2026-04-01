# T-037-02 Plan: Viewer Load Test

## Step 1: Add `data-viewer-ready` to Viewer.svelte

**File**: `web/src/lib/components/viewer/Viewer.svelte`

**Change**: On the container `<div>` (line 71), add `data-viewer-ready={ready || undefined}`. When `ready` is `true`, the attribute renders as `data-viewer-ready=""`. When `false`, `undefined` omits the attribute entirely.

**Verify**: `pnpm run check` in `web/` to confirm no type errors or warnings.

## Step 2: Increase Playwright test timeout

**File**: `web/playwright.config.ts`

**Change**: Bump `timeout` from `30_000` to `60_000`. The WASM binary is 15.3 MB, SwiftShader WebGL init is slow, and we need headroom above the 45s locator timeout used in the test.

**Verify**: Config parses correctly (Playwright will error on invalid config when tests run).

## Step 3: Create viewer.spec.ts

**File**: `web/e2e/viewer.spec.ts`

**Test structure**:
```
test.describe('viewer', () => {
  test('Bevy viewer initializes and loads scene', async ({ page }, testInfo) => {
    // Mock /api/projects/*/scene/* → test GLB
    // Navigate to /project/test-project/viewer
    // Assert iframe visible
    // Wait for [data-viewer-ready] (45s timeout)
    // Assert no error postMessage (check ErrorBanner not visible)
    // Screenshot → attach to test report
  });
});
```

**Key details**:
- `page.route()` before `page.goto()` to catch the API call during page load
- Scene mock returns `{ url: '/viewer/assets/models/test_scene.glb', metadata: {...} }`
- The `test_scene.glb` is served from `web/static/viewer/assets/models/` by the dev server
- 45s locator timeout for WASM init (within 60s test timeout)
- Screenshot attached via `testInfo.attach()` for CI artifact inspection

**Verify**: Run `cd web && pnpm test:e2e` — test should pass locally if WebGL is available, or timeout cleanly if not.

## Step 4: Run quality checks

- `cd web && pnpm test:e2e` — viewer test passes (or provides clear timeout/WebGL error)
- `just fmt` — formatting
- `just lint` — no clippy warnings (Rust side unchanged, but verify)

## Testing Strategy

**This is an e2e integration test, not a unit test.**
- Real SvelteKit dev server (started by Playwright webServer config)
- Real browser (Chromium via Playwright)
- Real WASM binary loading (the actual Bevy viewer)
- Mocked: only the scene API endpoint (no backend needed)

**What the test proves**:
1. SvelteKit route `/project/[id]/viewer` renders correctly
2. Viewer component mounts and creates the iframe
3. The iframe loads `/viewer/index.html` from static assets
4. WASM module downloads and initializes (or we learn it can't in this env)
5. Bevy sends `ready` postMessage to host
6. Host receives it and sets `ready = true` (reflected via data attribute)
7. No error messages during the process
8. Visual proof via screenshot

**What the test doesn't prove** (deferred to T-037-03):
- Actual 3D rendering of scan data
- Real GLB from the scan pipeline
- Zone interaction after load
