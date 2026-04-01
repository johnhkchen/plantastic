# T-037-03 Plan: scan-glb-viewer-test

## Step 1: Create scan-viewer.spec.ts

Write `web/e2e/scan-viewer.spec.ts` with two tests:

### Test 1: Real scan terrain GLB loads and renders

```
1. Mock API: page.route('**/api/projects/*/scene/*') → powell-market.glb response
2. Navigate: page.goto('/project/test-project/viewer')
3. Assert iframe visible
4. Wait: page.locator('[data-viewer-ready]').toBeAttached({ timeout: 45_000 })
5. Wait: page.waitForTimeout(2000) — allow scene to fetch + render
6. Assert: page.getByText('Failed to load').not.toBeVisible()
7. Screenshot: testInfo.attach('scan-terrain-loaded', ...)
```

### Test 2: Orbit interaction moves camera

```
1. Same mock + navigate + wait as Test 1
2. Screenshot "before": page.screenshot()
3. Get iframe bounding box
4. Mouse drag: page.mouse.move → mousedown → mousemove (drag 200px right) → mouseup
5. Wait 500ms for camera animation
6. Screenshot "after": page.screenshot()
7. Assert: !before.equals(after) — screenshots differ
8. Attach both screenshots to report
```

## Step 2: Run tests locally

```bash
cd web && npx playwright test scan-viewer
```

Verify:
- Test 1 passes and screenshot shows colored terrain (not gray)
- Test 2 passes if orbit works; if flaky, wrap with descriptive skip

## Step 3: Handle orbit flakiness (if needed)

If Test 2 fails because mouse events don't propagate through the iframe to Bevy's WebGL canvas in headless mode:

- Keep the test but wrap with `test.fixme('Orbit events unreliable in headless SwiftShader — T-037-03')`
- Document in review.md

## Verification Criteria

- `npx playwright test scan-viewer` — both tests pass (or orbit is explicitly skipped with reason)
- Screenshot artifact shows vertex-colored terrain, not a blank/gray canvas
- `just test-e2e` — all e2e tests pass (smoke + viewer + scan-viewer)
- GLB fixture is < 2 MB (confirmed: 1.3 MB)

## Testing Strategy

- **Unit tests**: None needed — this is an e2e test ticket
- **Integration tests**: The Playwright tests ARE the integration tests
- **Scenario impact**: No new scenarios — this validates existing S-037 pipeline
