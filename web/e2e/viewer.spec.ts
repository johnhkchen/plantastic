import { expect, test } from '@playwright/test';

test.describe('viewer', () => {
	test('Bevy viewer initializes and loads scene', async ({ page }, testInfo) => {
		// Mock the scene API so the page renders the Viewer instead of an error
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

		await page.goto('/project/test-project/viewer');

		// Verify the viewer iframe is present in the DOM
		const iframe = page.locator('iframe[src="/viewer/index.html"]');
		await expect(iframe).toBeVisible();

		// Wait for Bevy WASM to initialize — the ready postMessage sets data-viewer-ready
		// 45s budget: ~15s WASM download + ~20s SwiftShader WebGL init + headroom
		await expect(page.locator('[data-viewer-ready]')).toBeAttached({ timeout: 45_000 });

		// Confirm no error postMessage was received (ErrorBanner would appear)
		await expect(page.getByText('Failed to load')).not.toBeVisible();

		// Screenshot for CI artifact review
		const screenshot = await page.screenshot();
		await testInfo.attach('viewer-loaded', { body: screenshot, contentType: 'image/png' });
	});
});
