# T-037-02 Progress: Viewer Load Test

## Completed Steps

### Step 1: Add `data-viewer-ready` to Viewer.svelte
- Added `data-viewer-ready={ready || undefined}` to container div
- Prettier reformatted the element across multiple lines (no functional change)
- Attribute is present when WASM `ready` postMessage received, absent otherwise

### Step 2: Increase Playwright test timeout
- Changed `timeout` from `30_000` to `60_000` in `web/playwright.config.ts`
- Provides headroom for WASM download + SwiftShader init + test assertions

### Step 3: Create viewer.spec.ts
- Created `web/e2e/viewer.spec.ts` with single test
- Mocks `/api/projects/*/scene/*` → returns test GLB URL
- Navigates to `/project/test-project/viewer`
- Asserts iframe visible, waits for `[data-viewer-ready]` (45s timeout)
- Checks no error text visible
- Captures screenshot and attaches to test report

### Step 4: Quality checks
- `just fmt` — clean
- `just lint` — clean (no clippy warnings)
- Prettier — all modified files formatted

## Deviations from Plan

None. Implementation followed plan exactly.

## Remaining

- Review phase (next)
