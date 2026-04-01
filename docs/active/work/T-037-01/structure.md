# T-037-01 Structure: Playwright Setup

## Files Created

### `web/playwright.config.ts`

Playwright configuration file. Exports a `PlaywrightTestConfig` with:

- `baseURL`: `http://localhost:5173`
- `webServer`: starts `pnpm run dev`, waits for port 5173, reuses existing server when not in CI
- `projects`: single Chromium project using `Desktop Chrome` device
- `timeout`: 30000ms (30s per test)
- `retries`: 1 in CI, 0 locally (detected via `process.env.CI`)
- `reporter`: `[['html', { open: 'never' }], ['list']]` in CI, `'list'` locally
- `outputDir`: `test-results`
- `use.trace`: `'on-first-retry'` for debugging failures

### `web/e2e/smoke.spec.ts`

Smoke test file with two test cases:

1. **`landing page loads`** — navigates to `/`, asserts `h1` contains "Plantastic"
2. **`catalog page loads`** — intercepts `**/api/materials` with mock response, navigates to `/catalog`, asserts a mock material name is visible

Both tests are in a `describe('smoke')` block.

## Files Modified

### `web/package.json`

- Add `@playwright/test` to `devDependencies`
- Add `"test:e2e": "playwright test"` to `scripts`

### `web/.gitignore`

- Add `/test-results/`
- Add `/playwright-report/`
- Add `/playwright/.cache/`

### `justfile`

- Add `test-e2e` recipe after the testing section:
  ```
  # Run Playwright E2E tests (not part of `just check` — slow, needs WASM build)
  test-e2e:
      cd web && npx playwright test
  ```

## Files NOT Modified

- `justfile` `check` recipe — explicitly excluded per acceptance criteria
- No changes to SvelteKit routes or components
- No changes to `vite.config.ts` or `svelte.config.js`

## Directory Structure After

```
web/
├── e2e/
│   └── smoke.spec.ts
├── playwright.config.ts
├── package.json          (modified)
├── .gitignore            (modified)
└── ...existing files...
```

## Module Boundaries

- Playwright tests are fully isolated from the SvelteKit source — they interact only via HTTP (browser automation).
- API mocking happens at the Playwright network layer (`page.route()`), not in SvelteKit middleware.
- The `test:e2e` script and `just test-e2e` recipe are the only entry points.
