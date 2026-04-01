# T-018-02 Progress — Deploy Pipeline & Domain

## Completed

1. **Created `.github/workflows/deploy.yml`** — Full deploy workflow triggered on push to main:
   - Builds Lambda via cargo-zigbuild (same as local script)
   - Deploys infrastructure via SST (captures Lambda Function URL)
   - Builds and deploys SvelteKit to CF Pages
   - Wires Lambda URL to CF Worker BACKEND_URL secret
   - Deploys CF Worker (captures Worker URL)
   - Runs smoke tests through Worker URL (continue-on-error, warns on failure)
   - Concurrency group prevents overlapping deploys

2. **Updated `justfile` deploy recipe** — Now includes Lambda build step, uses pnpm instead of npm, runs from correct directories. Added `just smoke <url>` recipe.

3. **Created `setup-domain.md`** — Manual setup guide for GitHub secrets, SST secrets, CF Pages project, DNS, SSL, and database migrations.

## Deviations from Plan

- **Smoke test URL**: Instead of requiring a separate `CLOUDFLARE_ACCOUNT_SUBDOMAIN` secret, the workflow captures the Worker URL directly from `wrangler deploy` output. Falls back to direct Lambda URL if capture fails.

## Remaining

- Run `just check` to verify no regressions
- Write review.md
