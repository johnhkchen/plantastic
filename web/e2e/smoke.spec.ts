import { expect, test } from '@playwright/test';

test.describe('smoke', () => {
	test('landing page loads', async ({ page }) => {
		await page.goto('/');
		await expect(page.locator('h1')).toContainText('Plantastic');
		await expect(page.getByText('Landscaping design platform')).toBeVisible();
	});

	test('catalog page loads', async ({ page }) => {
		// Intercept API calls so the page renders without a backend
		await page.route('**/api/materials', (route) =>
			route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify([
					{
						id: '00000000-0000-0000-0000-000000000001',
						name: 'Flagstone Pavers',
						category: 'hardscape',
						unit: 'sq_ft',
						price_per_unit: '12.50',
						depth_inches: 2.0,
						supplier_sku: 'FSP-001'
					}
				])
			})
		);

		await page.goto('/catalog');
		await expect(page.getByText('Flagstone Pavers')).toBeVisible();
	});
});
