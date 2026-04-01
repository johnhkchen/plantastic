# T-037-03 Design: scan-glb-viewer-test

## Decision: Extend Existing Pattern with Scan-Specific Assertions

### Approach

Create `web/e2e/scan-viewer.spec.ts` that follows the T-037-02 pattern (mock API → navigate → wait for ready → screenshot) but:

1. Uses the real scan-produced `powell-market.glb` fixture (1.3 MB)
2. Returns scan-accurate metadata in the mock response (triangles: 49998, zone_count: 0)
3. Adds a wait after `data-viewer-ready` to allow the larger GLB to load and render
4. Takes a screenshot labeled for demo use
5. Includes an orbit interaction test (mouse drag on canvas)

### Alternatives Considered

**A. Single test file extending viewer.spec.ts**

Add the scan test as a second `test()` inside the existing `viewer.spec.ts`.

Rejected: T-037-03 is a separate ticket with different semantics (scan proof vs. viewer init). Separate files keep concerns clean and allow independent CI targeting.

**B. Parameterized test with multiple GLBs**

Use `test.describe.parallel()` with a fixture matrix.

Rejected: Over-engineering. There are only two fixtures. The tests verify fundamentally different things (boot vs. scan rendering). Shared parameterization would obscure the intent.

**C. Skip orbit test entirely**

The ticket marks orbit as optional. Skip it to reduce flakiness.

Rejected: The orbit test is low-cost (one mouse drag + screenshot comparison) and high-value (proves interactivity, not just a frozen image). Include it with a generous tolerance for screenshot comparison.

### Fixture Strategy

Use the terrain GLB generated from the real Powell & Market scan:

- Source: `assets/scans/samples/Scan at 09.23.ply` → `pt-scan` pipeline → terrain GLB
- File: `web/static/viewer/assets/models/powell-market.glb` (1.3 MB, 49,998 triangles)
- Vertex-colored from scan RGB (brick paths visible)
- Committed as a binary fixture — acceptable at 1.3 MB

No decimation needed — the pipeline already decimated from 20M points to ~50K triangles.

### Scene Load Timing

The `ready` postMessage fires when Bevy WASM initializes, BEFORE the GLB is fetched and rendered. After `ready`, the host sends `loadScene` and Bevy's AssetServer fetches the GLB. For a 1.3 MB file on localhost, this should take < 1 second. But we need a buffer for WebGL mesh setup.

Strategy: After `data-viewer-ready`, wait an additional fixed delay (2 seconds) before taking the screenshot. This is pragmatic — there's no `sceneLoaded` postMessage yet, and adding one is out of scope for this ticket.

### Orbit Test Design

1. Take initial screenshot after scene load
2. Simulate mouse drag on the viewer iframe (mousedown → mousemove → mouseup)
3. Wait 500ms for camera animation
4. Take second screenshot
5. Compare: the two screenshots should differ (camera moved)

Comparison method: pixel-level Buffer comparison. If screenshots are byte-identical, the camera didn't move. We don't need visual diff tooling — just `!Buffer.equals()`.

Risk: In headless Chromium with SwiftShader, WebGL rendering may be deterministic but orbit input events may not reach the Bevy camera controller through the iframe. If the orbit test is flaky, mark it with a clear skip reason referencing this ticket.

### Mock API Response

```json
{
  "url": "/viewer/assets/models/powell-market.glb",
  "metadata": {
    "zone_count": 0,
    "triangle_count": 49998,
    "tier": "good"
  }
}
```

Zone count is 0 because this is raw terrain — zone segmentation is a later pipeline stage.

### Test Structure

```
web/e2e/scan-viewer.spec.ts
  describe('scan viewer')
    test('real scan terrain GLB loads and renders')
      - mock API → powell-market.glb
      - navigate, wait for ready
      - wait 2s for scene render
      - assert no error
      - screenshot: 'scan-terrain-loaded'
    test('orbit interaction moves camera')
      - same setup as above
      - screenshot before drag
      - mouse drag on iframe
      - screenshot after drag
      - assert screenshots differ
```

### Acceptance Criteria Mapping

| AC | Design |
|----|--------|
| 1. Reference terrain GLB | powell-market.glb fixture |
| 2. Navigate to viewer | `/project/test-project/viewer` |
| 3. Intercept scene API | `page.route('**/api/projects/*/scene/*', ...)` |
| 4. Wait for ready | `[data-viewer-ready]` with 45s timeout |
| 5. Assert no error | `page.getByText('Failed to load').not.toBeVisible()` |
| 6. Screenshot | `testInfo.attach('scan-terrain-loaded', ...)` |
| 7. Orbit interaction | Mouse drag + screenshot diff |
