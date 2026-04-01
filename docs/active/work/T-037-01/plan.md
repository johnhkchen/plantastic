# T-037-01 Plan: Playwright Setup

## Step 1: Install Playwright

1. `cd web && pnpm add -D @playwright/test`
2. `cd web && npx playwright install chromium`
3. Verify `@playwright/test` appears in `web/package.json` devDependencies

## Step 2: Create Playwright Config

Create `web/playwright.config.ts` with webServer auto-start, Chromium-only project, 30s timeout, CI-aware retries, and trace-on-retry.

Verify: `cd web && npx playwright test --list` should parse the config without errors (even if no tests exist yet).

## Step 3: Update .gitignore

Add Playwright artifact directories to `web/.gitignore`:
- `/test-results/`
- `/playwright-report/`
- `/playwright/.cache/`

## Step 4: Write Smoke Test

Create `web/e2e/smoke.spec.ts` with two tests:
1. Landing page: navigate to `/`, assert `h1` text
2. Catalog page: intercept `/api/materials`, navigate to `/catalog`, assert mock material visible

## Step 5: Add Scripts

1. Add `"test:e2e": "playwright test"` to `web/package.json` scripts
2. Add `test-e2e` recipe to `justfile`

## Step 6: Verify

Run `just test-e2e` and confirm both smoke tests pass.

## Testing Strategy

- The smoke test IS the test — this ticket's deliverable is working test infrastructure, verified by the smoke test passing.
- No unit tests needed (nothing to unit test — config files and a smoke test).
- Verification: `just test-e2e` exits 0 with both tests green.
