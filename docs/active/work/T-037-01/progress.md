# T-037-01 Progress: Playwright Setup

## Completed

- [x] Step 1: Installed `@playwright/test@1.59.0` as devDependency, installed Chromium browser
- [x] Step 2: Created `web/playwright.config.ts` with webServer auto-start (port 4173), Chromium-only project, 30s test timeout, CI-aware retries
- [x] Step 3: Updated `web/.gitignore` with Playwright artifact directories
- [x] Step 4: Wrote `web/e2e/smoke.spec.ts` with landing page + catalog page (API-mocked) smoke tests
- [x] Step 5: Added `test:e2e` script to `web/package.json`, added `test-e2e` recipe to `justfile`
- [x] Step 6: Verified `just test-e2e` passes — 2 tests green in 3.5s

## Deviations from Plan

- **Port changed from 5173 to 4173**: Port 5173 was occupied by another dev server during development. Using 4173 avoids conflicts. The `npx vite dev --port 4173` command is used directly instead of `pnpm run dev -- --port 4173` because pnpm was not forwarding the `--port` flag correctly to Vite.
- **webServer timeout set to 60s**: Increased from 30s to 60s to handle slower cold starts with SvelteKit compilation.

## Remaining

Nothing — all acceptance criteria met.
