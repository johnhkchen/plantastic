# T-018-01 Progress — GitHub Actions CI

## Completed

### Step 1: Created `.github/workflows/ci.yml`
- Two parallel jobs: `rust` (fmt, lint, test, scenarios) and `web` (svelte-check, eslint+prettier)
- Rust caching via `Swatinem/rust-cache@v2`, pnpm caching via `actions/setup-node@v4`
- Triggers on PR to main and push to main
- 15-minute timeout per job

### Step 2: Fixed pre-existing web lint/type errors
During local validation, discovered 34 ESLint errors and 4 TypeScript errors across the web frontend — all pre-existing on main.

**Fixes applied:**
- Added `baseline: null` to 4 mock Project objects in `mock.ts` (missing required field)
- Added `static/viewer/` to `.prettierignore` and ESLint ignores (generated WASM output)
- Changed `let` to `const` for `mockTierAssignments` in `mock.ts`
- Added `{#each ... (key)}` to 14 each blocks across 11 Svelte components
- Added `resolve()` wrapping for navigation hrefs in Sidebar, TabNav, dashboard
- Replaced `new Set` with `SvelteSet` in QuoteComparison
- Removed unused imports/variables in 4 files
- Ran `pnpm prettier --write .` to fix formatting across 16 source files

### Step 3: Local verification
- `just check` passes (fmt-check, lint, test, scenarios)
- `pnpm run check` passes (0 errors)
- `pnpm run lint` passes (0 errors)

## Deviations from Plan

**Added: Pre-existing lint/type error fixes.** The plan didn't anticipate that the web frontend had 34+ lint errors and type errors. Per CLAUDE.md rule 6 ("Own what you find"), these were fixed as part of this ticket since CI would fail immediately otherwise. This is directly necessary for the CI acceptance criteria.

## Remaining

- Branch protection rule configuration (manual GitHub UI step — documented in review)
- Runtime verification happens when the workflow is first triggered on a real PR
