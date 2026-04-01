---
id: S-037
epic: E-014
title: Playwright E2E Testing — Viewer & Scan Pipeline
status: open
priority: high
tickets: [T-037-01, T-037-02, T-037-03]
---

## Goal

Verify the Bevy WASM viewer actually loads and renders after scan processing, and that the scan-to-viewer pipeline works end-to-end in a browser. Playwright runs headless Chromium against the SvelteKit dev server with the mock API.

## Key Challenges

1. **WASM init is slow** — Bevy takes several seconds to initialize WebGL. Tests need generous timeouts for the `ready` postMessage.
2. **WebGL in CI** — headless Chromium supports WebGL via SwiftShader (software renderer). It's slow but functional.
3. **API mocking** — the viewer page fetches scene URLs from the API. Use Playwright's `page.route()` to intercept and return fixture GLB URLs.
4. **The iframe boundary** — the Bevy viewer runs in an iframe with postMessage. Playwright can access iframe content via `page.frame()`.

## Acceptance Criteria

- Playwright installed and configured for the web/ project
- Tests verify: viewer page loads, iframe mounts, WASM initializes, scene loads
- Tests verify: scan-produced GLB loads in the viewer (from fixture, not live API)
- `just test-e2e` recipe runs the suite
- CI-compatible (headless Chromium, no GPU required)
