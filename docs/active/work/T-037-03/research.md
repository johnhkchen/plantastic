# T-037-03 Research: scan-glb-viewer-test

## Objective

Verify that a real scan-produced terrain GLB loads in the Bevy viewer via Playwright e2e test. This is the end-to-end proof: PLY → pt-scan → GLB → browser.

## Existing E2E Infrastructure

### Playwright Setup (T-037-01, done)

- Config: `web/playwright.config.ts` — 60s timeout, Chromium only, vite dev on :4173
- Smoke tests: `web/e2e/smoke.spec.ts` — landing page + catalog with API mocking
- Runner: `pnpm test:e2e` or `just test-e2e`

### Viewer Load Test (T-037-02, done)

- `web/e2e/viewer.spec.ts` — tests Bevy WASM init with a generic test_scene.glb (1.5KB)
- Pattern: mock `/api/projects/*/scene/*` → navigate to `/project/test-project/viewer` → wait for `[data-viewer-ready]` (45s) → assert no error → screenshot
- Fixture: `web/static/viewer/assets/models/test_scene.glb`

## Viewer Architecture

### Component: `web/src/lib/components/viewer/Viewer.svelte`

- Mounts iframe at `/viewer/index.html` (Bevy WASM app)
- Listens for `ready` postMessage → sets `ready = true` → sends `loadScene` with sceneUrl
- Container div: `data-viewer-ready={ready || undefined}` — used as Playwright locator
- Error postMessage → calls `onError` callback → ErrorBanner renders "Failed to load"

### Viewer Page: `web/src/routes/(app)/project/[id]/viewer/+page.svelte`

- Route: `/project/[id]/viewer`
- Fetches scene URL from `/api/projects/${projectId}/scene/${tier}`
- Response shape: `{ url: string, metadata: { zone_count, triangle_count, tier } }`

### Bevy WASM App

- Bridge plugin handles postMessage I/O (inbound: `loadScene`, `setTier`, `setLightAngle`; outbound: `ready`, `error`, `zoneTapped`)
- Scene plugin loads GLB via AssetServer, spawns scene entity
- WASM cold start: ~15s download + ~20s SwiftShader WebGL init in headless Chromium

## Scan GLB Generation

### pt-scan Pipeline

- `process_scan_timed()`: PLY → voxel downsample → outlier removal → RANSAC ground fit → classify
- `generate_terrain()`: ground points → Delaunay mesh → GLB export + plan view PNG
- GLB format: POSITION (vec3 f32), NORMAL (vec3 f32), COLOR_0 (vec4 u8 RGBA), indices (u32)
- Coordinate transform: Z-up scan → Y-up glTF `[x, z, -y]`

### Powell & Market Terrain GLB

- Generated from `assets/scans/samples/Scan at 09.23.ply` (308 MB, 20.5M points)
- Output: 1.3 MB terrain GLB — 49,998 triangles, vertex-colored from scan RGB
- Well under 2 MB fixture target, well under 5 MB CI limit
- Copied to `web/static/viewer/assets/models/powell-market.glb`

## Key Differences from T-037-02

| Aspect | T-037-02 (viewer.spec.ts) | T-037-03 (scan-viewer.spec.ts) |
|--------|--------------------------|-------------------------------|
| GLB | test_scene.glb (1.5KB, synthetic) | powell-market.glb (1.3MB, real scan) |
| Purpose | Viewer init + WASM boot | End-to-end scan → viewer proof |
| Screenshot | Generic viewer loaded | Real terrain with vertex colors |
| Orbit test | No | Optional (AC item 7) |
| Metadata | zone_count: 1, triangles: 100 | zone_count: 0, triangles: ~50K |

## Constraints

- Test timeout: 60s (config), with 45s budget for WASM init
- GLB is larger (1.3MB vs 1.5KB) — may add a few seconds to asset fetch
- Orbit interaction requires mouse events on the iframe canvas — Playwright can dispatch these but WebGL picking may be unreliable in headless mode
- Screenshot is demo material — should show colored terrain, not gray blob

## Open Questions

1. Does the 1.3MB GLB load within the 45s WASM init window, or does scene loading add noticeable time after `ready`?
2. Can we reliably detect orbit camera movement in a screenshot diff, or is the orbit test inherently flaky?
3. Should we add a `sceneLoaded` postMessage for more precise readiness detection?
