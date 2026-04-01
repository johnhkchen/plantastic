# T-018-01 Review — GitHub Actions CI

## Summary

Created a GitHub Actions CI workflow that runs the full quality gate (`just check` + web lint/check) on every PR to main. Also fixed 34+ pre-existing lint and type errors in the web frontend that would have caused immediate CI failure.

## Files Created

- `.github/workflows/ci.yml` — CI workflow with two parallel jobs (rust, web)
- `docs/active/work/T-018-01/research.md`
- `docs/active/work/T-018-01/design.md`
- `docs/active/work/T-018-01/structure.md`
- `docs/active/work/T-018-01/plan.md`
- `docs/active/work/T-018-01/progress.md`

## Files Modified

- `web/.prettierignore` — added `static/viewer` (generated WASM output)
- `web/eslint.config.js` — added `static/viewer/` to ignores
- `web/src/lib/api/mock.ts` — added missing `baseline: null` to mock Projects, `let` → `const`
- `web/src/lib/components/Sidebar.svelte` — added resolve(), each keys
- `web/src/lib/components/TabNav.svelte` — added resolve(), each key
- `web/src/lib/components/assignment/MaterialPicker.svelte` — each key
- `web/src/lib/components/assignment/TierTabs.svelte` — each key
- `web/src/lib/components/catalog/CatalogFilter.svelte` — each key
- `web/src/lib/components/quote/QuoteComparison.svelte` — SvelteSet, each keys, eslint-disable for relative link
- `web/src/lib/components/zone-editor/ZoneEditor.svelte` — each key, unused param
- `web/src/routes/(app)/catalog/+page.svelte` — each key, unused var
- `web/src/routes/(app)/dashboard/+page.svelte` — resolve(), each key
- `web/src/routes/(app)/project/[id]/+page.svelte` — each keys, unused var
- `web/src/routes/(app)/project/[id]/materials/+page.svelte` — removed unused import
- `web/src/routes/(app)/project/[id]/viewer/+page.svelte` — each key

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| `.github/workflows/ci.yml` runs on every PR to main | ✅ Configured |
| Steps: install Rust, just, fmt-check, lint, test, scenarios | ✅ All four steps in `rust` job |
| Rust toolchain cached | ✅ `actions-rust-lang/setup-rust-toolchain@v1` + `Swatinem/rust-cache@v2` |
| Cargo build artifacts cached between runs | ✅ `Swatinem/rust-cache@v2` caches registry + target |
| SvelteKit lint/check also runs | ✅ `web` job: `pnpm run check` + `pnpm run lint` |
| `just scenarios` output visible in CI logs | ✅ Runs as dedicated step, stdout captured in logs |
| PR blocked if any step fails | ⚠️ Requires manual branch protection rule (see below) |
| Runs in < 10 minutes | ⏳ Expected yes with caching; verified on first real run |

## Branch Protection (Manual Step Required)

The workflow file defines the checks. **Branch protection must be configured in GitHub UI:**
1. Settings → Branches → Add rule for `main`
2. Require status checks: enable `rust` and `web` jobs
3. Require branches to be up to date (recommended)

This cannot be automated via the workflow file.

## Scenario Dashboard

Before and after are identical — this ticket adds CI infrastructure, not application capabilities:
- **Before:** 58.0 / 240.0 min (24.2%), 8 pass, 0 fail, 9 not implemented
- **After:** 58.0 / 240.0 min (24.2%), 8 pass, 0 fail, 9 not implemented

No regressions. No scenario impact expected — CI protects existing value, it doesn't add new customer-facing capability.

## Test Coverage

This ticket's deliverable is a YAML configuration file. Coverage comes from:
- **Local validation:** `just check` passes, `pnpm run check` passes, `pnpm run lint` passes
- **Runtime validation:** First PR opened with this workflow will be the integration test

No new Rust tests or scenario registrations needed.

## Open Concerns

1. **First-run timing.** Cold cache CI run will be slow (3-5 min for Rust compilation). Subsequent runs with warm cache should be well under 10 minutes. Monitor the first few runs.

2. **pnpm version pinning.** The workflow pins pnpm v9. If the project upgrades pnpm, the CI config needs updating. Consider adding `packageManager` field to `web/package.json` for automatic version detection.

3. **sqlx compile-time checking.** Currently all queries use runtime string-based `query_scalar`/`query_as`. If any crate switches to `sqlx::query!()` macros, CI will need either `SQLX_OFFLINE=true` with cached `.sqlx/` data or a CI database connection.

4. **Database-dependent tests.** Currently no tests require a database. When integration tests land (per CLAUDE.md rule 8: "real infrastructure"), the CI will need a Postgres service container. This is a future concern, not a blocker.
