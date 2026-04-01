# T-037-01 Review: Playwright Setup

## Summary

Installed Playwright E2E testing infrastructure for the SvelteKit web app. Two smoke tests verify the setup works: one tests the static landing page, the other tests the catalog page with API route interception via `page.route()`.

## Files Created

| File | Purpose |
|------|---------|
| `web/playwright.config.ts` | Playwright config: webServer auto-start on port 4173, Chromium-only, 30s test timeout, CI-aware retries (1 in CI, 0 locally), trace on first retry |
| `web/e2e/smoke.spec.ts` | Two smoke tests: landing page h1 assertion, catalog page with mocked `/api/materials` response |

## Files Modified

| File | Change |
|------|--------|
| `web/package.json` | Added `@playwright/test@^1.59.0` to devDependencies, added `test:e2e` script |
| `web/.gitignore` | Added `/test-results/`, `/playwright-report/`, `/playwright/.cache/` |
| `justfile` | Added `test-e2e` recipe (not part of `just check` per acceptance criteria) |

## Test Coverage

- **Landing page**: Asserts `<h1>` contains "Plantastic" and paragraph text is visible
- **Catalog page**: Intercepts `GET /api/materials` with mock fixture, asserts material name renders — proves `page.route()` API mocking pattern works for T-037-02

The smoke test itself IS the verification that the infrastructure works. No additional unit tests are needed.

## Acceptance Criteria Checklist

- [x] `@playwright/test` installed as devDependency
- [x] `npx playwright install chromium` run (Chromium only)
- [x] `web/playwright.config.ts` created with baseURL, webServer, Chromium project, 30s timeout, CI retries
- [x] `web/e2e/` directory created for test specs
- [x] `test:e2e` script in `web/package.json`
- [x] `just test-e2e` recipe in justfile
- [x] Smoke test navigates to `/` and asserts heading
- [x] Smoke test navigates to `/catalog` and asserts page loads
- [x] `just test-e2e` passes locally (2 passed, 3.5s)
- [x] NOT added to `just check` gate
- [x] `web/test-results/` and `web/playwright-report/` in `.gitignore`

## Design Decisions

1. **Port 4173 instead of 5173**: Avoids conflicts with any running dev server. Uses `npx vite dev --port 4173` directly because pnpm doesn't forward `--port` correctly through `pnpm run dev --`.
2. **`page.route()` for API mocking**: Chose this over mock servers or SvelteKit middleware. Aligns with S-037 story guidance and establishes the pattern for T-037-02 viewer tests.
3. **60s webServer timeout**: SvelteKit cold compilation can be slow. 60s gives headroom without being wasteful.

## Open Concerns

1. **Port conflict risk in CI**: If CI also uses port 4173 for something else, tests will fail. Mitigated by `reuseExistingServer: false` in CI (via `!process.env.CI`).
2. **The `(app)` layout makes API calls** (session, tenant) that are NOT intercepted. These fail silently because the Vite proxy has no backend. Currently cosmetic — pages still render. If this causes flaky tests later, broader route interception may be needed.
3. **Not in `just check`**: By design per the ticket, but means E2E regressions won't be caught by the standard quality gate until a future ticket adds it.

## Scenario Impact

No scenarios directly depend on Playwright setup — this is foundational infrastructure for T-037-02 (viewer load test) which will exercise the scan-to-viewer pipeline scenarios.
