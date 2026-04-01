import { expect, test } from '@playwright/test';

test.describe('scan viewer', () => {
	/** Mock the scene API to serve the real scan-produced terrain GLB. */
	async function mockScanScene(page: import('@playwright/test').Page) {
		await page.route('**/api/projects/*/scene/*', (route) =>
			route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					url: '/viewer/assets/models/powell-market.glb',
					metadata: { zone_count: 0, triangle_count: 49998, tier: 'good' }
				})
			})
		);
	}

	test('real scan terrain GLB loads and renders', async ({ page }, testInfo) => {
		await mockScanScene(page);
		await page.goto('/project/test-project/viewer');

		// Verify the viewer iframe is present
		const iframe = page.locator('iframe[src="/viewer/index.html"]');
		await expect(iframe).toBeVisible();

		// Wait for Bevy WASM to initialize (ready postMessage sets data-viewer-ready)
		// 45s budget: ~15s WASM download + ~20s SwiftShader WebGL init + headroom
		await expect(page.locator('[data-viewer-ready]')).toBeAttached({ timeout: 45_000 });

		// Allow time for the 1.3 MB GLB to fetch and render after ready
		await page.waitForTimeout(2000);

		// Confirm no error postMessage was received
		await expect(page.getByText('Failed to load')).not.toBeVisible();

		// Screenshot — this is the end-to-end proof: PLY → pt-scan → GLB → browser
		// Should show Powell & Market brick-path terrain with vertex colors
		const screenshot = await page.screenshot();
		await testInfo.attach('scan-terrain-loaded', {
			body: screenshot,
			contentType: 'image/png'
		});
	});

	test('orbit interaction moves camera', async ({ page }, testInfo) => {
		await mockScanScene(page);
		await page.goto('/project/test-project/viewer');

		await expect(page.locator('[data-viewer-ready]')).toBeAttached({ timeout: 45_000 });
		await page.waitForTimeout(2000);

		// Screenshot before orbit
		const before = await page.screenshot();
		await testInfo.attach('orbit-before', { body: before, contentType: 'image/png' });

		// Simulate mouse drag on the viewer to orbit the camera
		// Target the center of the iframe bounding box
		const iframe = page.locator('iframe[src="/viewer/index.html"]');
		const box = await iframe.boundingBox();
		if (!box) throw new Error('iframe not found for orbit test');

		const cx = box.x + box.width / 2;
		const cy = box.y + box.height / 2;

		await page.mouse.move(cx, cy);
		await page.mouse.down();
		// Drag 200px to the right to orbit
		for (let x = cx; x <= cx + 200; x += 20) {
			await page.mouse.move(x, cy);
		}
		await page.mouse.up();

		// Wait for camera animation to settle
		await page.waitForTimeout(500);

		// Screenshot after orbit
		const after = await page.screenshot();
		await testInfo.attach('orbit-after', { body: after, contentType: 'image/png' });

		// The two screenshots must differ — proves the camera actually moved
		expect(before.equals(after)).toBe(false);
	});
});
