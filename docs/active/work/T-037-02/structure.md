# T-037-02 Structure: Viewer Load Test

## Files Modified

### `web/src/lib/components/viewer/Viewer.svelte`
- Add `data-viewer-ready` attribute to the container `<div>` — bound to `ready` state
- The attribute is present (empty string) when `ready === true`, absent when false
- No other changes to the component logic or structure

### `web/playwright.config.ts`
- Increase test timeout from 30s to 60s — WASM cold start + SwiftShader rendering needs headroom
- The 30s budget is for the WASM init alone; the full test (navigate + mock setup + wait + screenshot) needs more

## Files Created

### `web/e2e/viewer.spec.ts`
Single test file with one `test.describe('viewer')` block containing one test.

**Test: "Bevy viewer initializes and loads scene"**

```
Setup:
  1. page.route('**/api/projects/*/scene/*') → mock scene response
  2. page.goto('/project/test-project/viewer')

Assertions:
  3. iframe[src="/viewer/index.html"] is present in DOM
  4. [data-viewer-ready] appears (up to 45s for WASM cold start)
  5. No ErrorBanner visible on page
  6. Screenshot captured and attached to test report
```

### Structure of the test

```ts
import { expect, test } from '@playwright/test';

test.describe('viewer', () => {
  test('Bevy viewer initializes and loads scene', async ({ page }, testInfo) => {
    // 1. Mock the scene API
    await page.route('**/api/projects/*/scene/*', (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          url: '/viewer/assets/models/test_scene.glb',
          metadata: { zone_count: 1, triangle_count: 100, tier: 'good' }
        })
      })
    );

    // 2. Navigate
    await page.goto('/project/test-project/viewer');

    // 3. Assert iframe present
    const iframe = page.locator('iframe[src="/viewer/index.html"]');
    await expect(iframe).toBeVisible();

    // 4. Wait for viewer ready (WASM cold start)
    await expect(page.locator('[data-viewer-ready]')).toBeAttached({ timeout: 45_000 });

    // 5. No error
    await expect(page.locator('[class*="ErrorBanner"]')).not.toBeVisible();
    // Alternative: check text content for error indicators

    // 6. Screenshot
    const screenshot = await page.screenshot();
    await testInfo.attach('viewer-loaded', { body: screenshot, contentType: 'image/png' });
  });
});
```

## Module Boundaries

- **Viewer.svelte**: Only change is adding a data attribute. No new props, no new logic.
- **viewer.spec.ts**: Self-contained test. No shared fixtures or helpers needed.
- **playwright.config.ts**: Timeout increase affects all tests, but 60s is still reasonable for the smoke tests (they'll finish in <5s regardless).

## No Files Deleted

No files are removed.

## Ordering

1. Modify `Viewer.svelte` first (add data attribute)
2. Update `playwright.config.ts` (timeout)
3. Create `viewer.spec.ts` (test depends on both changes above)

## Interface Notes

The `data-viewer-ready` attribute is the contract between the component and the test:
- Component sets it when `ready` transitions to `true`
- Test waits for it with `toBeAttached()`
- Using `toBeAttached` rather than `toBeVisible` because the div is always visible (it's the container), but the attribute presence is what we're checking
