# T-037-03 Structure: scan-glb-viewer-test

## Files Created

### `web/e2e/scan-viewer.spec.ts`

New Playwright test file for scan-specific viewer testing.

```
web/e2e/scan-viewer.spec.ts
├── describe('scan viewer')
│   ├── test('real scan terrain GLB loads and renders')
│   │   ├── Mock API route → powell-market.glb
│   │   ├── Navigate to /project/test-project/viewer
│   │   ├── Wait for [data-viewer-ready] (45s)
│   │   ├── Wait 2s for scene render
│   │   ├── Assert no error visible
│   │   └── Attach screenshot 'scan-terrain-loaded'
│   └── test('orbit interaction moves camera')
│       ├── Mock API route → powell-market.glb
│       ├── Navigate + wait for ready + wait for render
│       ├── Screenshot before drag
│       ├── Mouse drag on iframe bounding box
│       ├── Wait 500ms
│       ├── Screenshot after drag
│       └── Assert before ≠ after (Buffer comparison)
```

### `web/static/viewer/assets/models/powell-market.glb`

Binary fixture: terrain GLB generated from Powell & Market scan. Already placed (1.3 MB).

## Files Modified

None. The test is additive — no changes to existing code.

## Module Boundaries

- `scan-viewer.spec.ts` is fully independent of `viewer.spec.ts`
- Both test files share the same Playwright config and mock API pattern
- No shared test utilities needed — the pattern is simple enough to duplicate
- Fixture GLB is a static asset served by SvelteKit's vite dev server

## Interface Contracts

### Mock API Contract

Test mocks the same endpoint as T-037-02 (`/api/projects/*/scene/*`) but with scan-specific metadata:

```typescript
{
  url: '/viewer/assets/models/powell-market.glb',
  metadata: { zone_count: 0, triangle_count: 49998, tier: 'good' }
}
```

### Viewer Ready Contract

Same as T-037-02 — `data-viewer-ready` attribute appears when Bevy sends `ready` postMessage. No new contracts introduced.

### Scene Render Timing

No explicit "scene loaded" signal exists. Design decision: fixed 2-second wait after `ready`. This is documented as a known limitation in the test comments.

## Ordering

1. Fixture already placed (`powell-market.glb`)
2. Write `scan-viewer.spec.ts` with both tests
3. Run tests to verify
4. If orbit test is flaky in headless, consider wrapping with `test.skip` and a clear reason

## Test Artifacts

Screenshots are attached to the Playwright HTML report:
- `scan-terrain-loaded` — primary demo screenshot
- `orbit-before` / `orbit-after` — orbit interaction proof (if orbit test included)

These appear in `web/test-results/` (gitignored) and the HTML report.
