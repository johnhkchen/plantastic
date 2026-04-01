# T-018-02 Plan — Deploy Pipeline & Domain

## Step 1: Create deploy workflow

**File:** `.github/workflows/deploy.yml`

Create the GitHub Actions workflow with:
- Trigger: `push` to `main`
- Concurrency: `deploy-dev`, cancel-in-progress false
- Single job: `deploy` on `ubuntu-latest`, 30 min timeout
- Steps in order:
  1. Checkout
  2. Setup Rust (stable + aarch64-unknown-linux-gnu target)
  3. Rust cache
  4. Install zig (`sudo snap install zig --classic --beta` or apt)
  5. Install cargo-zigbuild (`cargo install cargo-zigbuild`)
  6. Run `./scripts/build-lambda.sh`
  7. Setup Node 22 + pnpm 9 (same actions as ci.yml)
  8. `cd infra && npm install`
  9. `cd infra && npx sst deploy --stage dev` — capture output, extract apiUrl
  10. `cd web && pnpm install --frozen-lockfile`
  11. `cd web && pnpm run build`
  12. `cd web && npx wrangler pages deploy .svelte-kit/cloudflare --project-name plantastic`
  13. `echo "$API_URL" | cd worker && npx wrangler secret put BACKEND_URL`
  14. `cd worker && npm install && npx wrangler deploy`
  15. Smoke test: `./scripts/verify-deploy.sh $WORKER_URL` with `continue-on-error: true`
  16. Check smoke result, fail job if smoke failed (but deploy already happened)

- Environment variables from GitHub secrets:
  - `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` (for SST)
  - `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` (for Wrangler)

**Verify:** Workflow YAML is valid (check syntax). Workflow will be tested on first push to main after merge.

## Step 2: Update justfile deploy recipe

Update the existing `deploy` recipe to include Lambda build:
```
deploy stage="dev":
    ./scripts/build-lambda.sh
    cd infra && npx sst deploy --stage {{stage}}
    cd web && pnpm run build
    cd web && npx wrangler pages deploy .svelte-kit/cloudflare --project-name plantastic
    cd worker && npx wrangler deploy
```

Add new `smoke` recipe:
```
smoke url:
    ./scripts/verify-deploy.sh {{url}}
```

**Verify:** `just --list` shows both recipes with descriptions.

## Step 3: Create domain setup guide

**File:** `docs/active/work/T-018-02/setup-domain.md`

Document the manual steps:
1. Cloudflare DNS setup for get-plantastic.com
2. CF Pages project creation and custom domain binding
3. CF Worker route configuration
4. GitHub repo secrets setup (AWS + Cloudflare credentials)
5. SST secret setup (DatabaseUrl per stage)
6. Initial deploy verification

**Verify:** Guide is complete and references correct secret names.

## Step 4: Run quality gate

Run `just check` to verify no regressions. The deploy workflow is YAML-only — no Rust code changes, so fmt/lint/test/scenarios should all pass unchanged.

**Verify:** `just check` passes.

## Testing Strategy

This ticket is primarily infrastructure automation (YAML workflow + shell recipes). Testing approach:

1. **Syntax validation**: GitHub Actions YAML is valid (actionlint if available, or manual review)
2. **Script compatibility**: `verify-deploy.sh` already works with any URL — no changes needed
3. **Local recipe test**: `just --list` confirms new recipes are registered
4. **Integration test**: First push to main after merge exercises the full pipeline. Smoke tests verify end-to-end.
5. **No Rust code changes**: No new unit tests needed. Scenario dashboard should be unchanged.

## Commit Plan

1. Single commit: deploy workflow + justfile updates + domain setup guide
   - `.github/workflows/deploy.yml` (new)
   - `justfile` (modified)
   - `docs/active/work/T-018-02/setup-domain.md` (new)
