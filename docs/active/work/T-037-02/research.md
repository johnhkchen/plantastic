# T-037-02 Research: Viewer Load Test

## Existing E2E Infrastructure (from T-037-01)

Playwright is fully configured in `web/`:
- **Config**: `web/playwright.config.ts` — Chromium only, 30s test timeout, 60s server startup, port 4173
- **Smoke tests**: `web/e2e/smoke.spec.ts` — landing page + catalog page with `page.route()` API mocking
- **Scripts**: `pnpm test:e2e` runs `playwright test`
- **Justfile**: `test-e2e` recipe exists (not in `just check` gate)
- **Artifacts**: `web/test-results/` and `web/playwright-report/` gitignored

## Viewer Architecture

### Route Structure
- **Page**: `web/src/routes/(app)/project/[id]/viewer/+page.svelte`
- **Layout**: `web/src/routes/(app)/project/[id]/+layout.ts` — passes `{ id: params.id }` as data
- **Component**: `web/src/lib/components/viewer/Viewer.svelte`
- **Types**: `web/src/lib/components/viewer/types.ts`

### Communication Protocol
The viewer uses a host ↔ iframe postMessage bridge:

**Viewer.svelte** (host side):
- Renders `<iframe src="/viewer/index.html">` inside a container div
- Listens for `message` events via `window.addEventListener('message', handleMessage)`
- On `ready` message: sets `ready = true`, sends `loadScene` command, calls `onReady?.()`
- Shows "Loading viewer..." overlay until `ready` is true
- Type-checks messages with `isViewerMessage()` guard

**viewer/index.html** (iframe side):
- Loads Bevy WASM: `plantastic-viewer-265851ff64b87b5.js` + `.wasm` (15.3 MB)
- Canvas: `<canvas id="bevy-canvas">`
- Loading indicator hidden via MutationObserver on canvas attributes or 10s fallback
- WASM bindings exported to `window.wasmBindings`
- Dispatches `TrunkApplicationStarted` custom event

### Message Types (types.ts)
- **Outbound** (viewer → host): `ready`, `error`, `zoneTapped`, `lightAngleChanged`, `tierChanged`
- **Inbound** (host → viewer): `loadScene`, `setTier`, `setLightAngle`

### Page Data Flow
1. Layout provides `data.id` from URL params
2. Page calls `apiFetch<SceneResponse>('/projects/${projectId}/scene/${tier}')` on mount
3. `apiFetch` prepends `/api` baseUrl, so actual URL is `/api/projects/{id}/scene/{tier}`
4. Response shape: `{ url: string, metadata: { zone_count, triangle_count, tier } }`
5. `sceneUrl` passed to `<Viewer>`, which sends `loadScene` to iframe on `ready`

## Test Assets
- **Test GLB**: `web/static/viewer/assets/models/test_scene.glb` — 1,516 bytes, exists
- Static files served at `/viewer/assets/models/test_scene.glb` by SvelteKit dev server

## Key Constraints

### WASM Loading in Headless CI
- WASM binary is 15.3 MB — cold download + compile takes time
- SwiftShader (software WebGL) adds 10-20s initialization
- The iframe's WASM module loads eagerly via `<script type="module">` with `await init()`
- If WASM init fails (no WebGL), there's no explicit error postMessage from the iframe side — the viewer just never sends `ready`
- Current Playwright timeout is 30s per test — may be tight for WASM + SwiftShader

### API Mocking Needed
- The page fetches `/api/projects/{id}/scene/{tier}` on mount via `$effect`
- Without mocking, this 404s and sets `viewerError`, showing ErrorBanner instead of Viewer
- Need to intercept this route and return `{ url, metadata }` pointing to test GLB

### iframe Challenges
- Playwright's `page.frameLocator()` or `page.frame()` needed to interact with iframe content
- postMessage events fire on the parent window, not inside the iframe
- The `data-viewer-ready` approach (ticket suggestion) would require modifying Viewer.svelte — adding `data-viewer-ready` attribute to the container div when `ready` is true. This is the cleanest Playwright integration.

### Screenshot Capture
- Playwright's `page.screenshot()` saves to a path
- Config already has `outputDir: './test-results'`
- Screenshots can be attached to test results via `testInfo.attach()`

## Patterns from Existing Tests
- `smoke.spec.ts` uses `test.describe()` grouping
- API mocking via `page.route()` with `route.fulfill()` — JSON body as stringified object
- Assertions via `expect(locator).toBeVisible()` / `.toContainText()`
- No fixture files — inline mock data
