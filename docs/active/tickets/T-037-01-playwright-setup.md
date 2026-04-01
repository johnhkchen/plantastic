---
id: T-037-01
story: S-037
title: playwright-setup
type: task
status: open
priority: high
phase: done
depends_on: []
---

## Context

No web testing infrastructure exists. This ticket sets up Playwright with SvelteKit, configures the dev server auto-start, and writes a smoke test proving the setup works.

## Acceptance Criteria

- Install `@playwright/test` as devDependency in web/package.json
- Run `npx playwright install chromium` (only Chromium needed, not Firefox/WebKit)
- Create `web/playwright.config.ts`:
  - baseURL: `http://localhost:5173`
  - webServer: `pnpm run dev` with auto-start and port wait
  - projects: Chromium only (add Firefox/WebKit later if needed)
  - timeout: 30s per test (WASM init is slow)
  - retries: 1 in CI, 0 locally
- Create `web/e2e/` directory for test specs
- Add `test:e2e` script to web/package.json: `playwright test`
- Add `just test-e2e` recipe to justfile: `cd web && npx playwright test`
- Smoke test `web/e2e/smoke.spec.ts`:
  - Navigate to `/` (dashboard)
  - Assert page title or heading is present
  - Navigate to `/catalog`
  - Assert catalog page loads
- `just test-e2e` passes locally

## Implementation Notes

- Do NOT add Playwright to the `just check` gate yet — it's slow and needs WASM build
- The SvelteKit mock API (mock.ts) handles all API calls in dev mode, no backend needed
- Playwright config should set `CI` env detection for retry/reporter switching
- Add `web/test-results/` and `web/playwright-report/` to .gitignore
