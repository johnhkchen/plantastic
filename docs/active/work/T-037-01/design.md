# T-037-01 Design: Playwright Setup

## Decision: Smoke Test Strategy

### Option A: Test only `/` (static landing page)

- Pros: Zero API dependency, deterministic, fast.
- Cons: Only proves Playwright works, not that app routes load. Doesn't exercise the `(app)` layout group.

### Option B: Test `/` and `/catalog` with `page.route()` API interception

- Pros: Proves Playwright can intercept API calls (pattern needed for T-037-02). Tests that app routes render. Aligns with S-037's mention of `page.route()`.
- Cons: Slightly more setup.

### Option C: Test `/` and `/catalog` without interception (assert loading/error states)

- Pros: Simple. No mock data needed.
- Cons: Testing error states isn't a meaningful smoke test. Fragile if error rendering changes.

**Decision: Option B.** The smoke test should prove two things: (1) Playwright + SvelteKit webServer works, (2) API route interception works. This sets up the pattern for T-037-02. The root `/` test needs no interception. The `/catalog` test uses `page.route()` to return mock materials.

## Playwright Config Design

```
webServer:
  command: pnpm run dev
  url: http://localhost:5173
  reuseExistingServer: !CI   # reuse if dev server is already running locally
  timeout: 30000              # 30s for dev server startup

projects:
  - name: chromium
    use: { ...devices['Desktop Chrome'] }

timeout: 30000                # 30s per test (WASM init headroom)
retries: CI ? 1 : 0
reporter: CI ? [html, list] : list
outputDir: test-results
```

## Smoke Test Design

### Test 1: Landing page loads

```
Navigate to /
Assert h1 contains "Plantastic"
Assert page has text "Landscaping design platform"
```

### Test 2: Catalog page loads with mocked API

```
Intercept GET /api/materials → return mock material array
Navigate to /catalog
Assert page contains a material name from the mock data
```

The catalog test needs to intercept the API call that `CatalogFilter` triggers on mount. Looking at the catalog page, it calls `apiFetch('/materials')` which the Vite proxy sends to `/api/materials`. Playwright's `page.route('**/api/materials', ...)` will intercept before the proxy.

## Rejected Alternatives

- **Vitest for component tests**: Out of scope. This ticket is E2E infrastructure only.
- **All three browsers**: Ticket says Chromium only, add others later.
- **Adding to `just check`**: Explicitly excluded by acceptance criteria.
- **Complex mock server**: Overkill. `page.route()` is the right tool for E2E API mocking.

## Risk

- The `(app)` layout may make additional API calls (session, tenant) that could cause console errors. This is cosmetic — the pages will still render. If it becomes a problem, we can add broader route interception later.
