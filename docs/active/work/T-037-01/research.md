# T-037-01 Research: Playwright Setup

## Current State

### Web Project Structure

The SvelteKit app lives in `web/` with the following key files:

- **`web/package.json`** — Scripts: `dev`, `build`, `preview`, `check`, `lint`, `format`. No test-related scripts. Uses `pnpm` as package manager. DevDependencies include SvelteKit 2.50.2, Svelte 5.54.0, Vite 7.3.1, Tailwind CSS 4.2.2, TypeScript 5.9.3. No test frameworks installed (no vitest, no Playwright).
- **`web/svelte.config.js`** — Cloudflare adapter, Svelte 5 rune mode.
- **`web/vite.config.ts`** — Tailwind + SvelteKit plugins. Dev server proxies `/api` to `process.env.API_URL` (default `http://localhost:3000`).
- **`web/.gitignore`** — Ignores `node_modules/`, `.svelte-kit/`, `/build`, env files, vite timestamps. Does NOT yet ignore Playwright artifacts.

### Routes Relevant to Smoke Test

- **`/` (`web/src/routes/+page.svelte`)** — Landing page. Contains `<h1>` with text "Plantastic" and `<p>` with "Landscaping design platform."
- **`/dashboard` (`web/src/routes/(app)/dashboard/+page.svelte`)** — Project list. Calls `apiFetch` on mount. Has loading/error states.
- **`/catalog` (`web/src/routes/(app)/catalog/+page.svelte`)** — Material catalog. Also calls `apiFetch` on mount.

### API Dependencies

Both `/dashboard` and `/catalog` call the real API on mount via `apiFetch()`. In dev mode, the Vite proxy forwards `/api` to `localhost:3000`. There is a `mock.ts` with fixture data but it's not auto-wired as a dev interceptor. This means:

- The root `/` page works without any API — pure static content. Best candidate for smoke test.
- `/dashboard` and `/catalog` will fail to load data without a running API, but the pages themselves render (with error/loading states).

### Justfile

- `justfile` at project root. Has Rust-focused recipes. No web test recipes.
- `dev-web` recipe: `cd web && npm run dev`.
- No `test-e2e` recipe exists.

### .gitignore (root)

- Root `.gitignore` handles Rust, Node, env, and scan assets. Does not mention Playwright directories (`test-results/`, `playwright-report/`).

### Playwright Compatibility

- SvelteKit + Playwright is well-supported. `@playwright/test` ships its own test runner.
- `webServer` config in `playwright.config.ts` can auto-start `pnpm run dev` and wait for port 5173.
- Chromium-only install keeps CI fast (~100MB vs ~400MB for all browsers).

## Constraints

1. The acceptance criteria say "do NOT add Playwright to `just check`" — it's a separate gate.
2. Only Chromium needed. Firefox/WebKit deferred.
3. Timeout should be 30s per test (WASM init slowness, though smoke test won't need WASM).
4. Retries: 1 in CI, 0 locally. Detect via `process.env.CI`.
5. Mock API handles dev-mode API calls — but for the smoke test on `/`, no API is needed.
6. Must add `web/test-results/` and `web/playwright-report/` to `.gitignore`.

## Key Files to Modify/Create

| File | Action |
|------|--------|
| `web/package.json` | Add `@playwright/test` devDependency, `test:e2e` script |
| `web/playwright.config.ts` | Create — config with webServer, Chromium project, timeouts |
| `web/e2e/smoke.spec.ts` | Create — smoke test for `/` and `/catalog` |
| `web/.gitignore` | Add `test-results/`, `playwright-report/` |
| `justfile` | Add `test-e2e` recipe |

## Observations

- The root page `/` is the best smoke target — no API calls, deterministic content.
- `/catalog` calls `apiFetch` which will fail without a backend. The smoke test can either: (a) assert the page renders at all (loading state), or (b) intercept the API route with Playwright's `page.route()`. Option (b) is more robust and aligns with S-037's mention of `page.route()`.
- The `(app)` route group layout includes Sidebar and Header components — these also call APIs for session/tenant info. The smoke test should handle this gracefully.
