# T-018-02 Structure — Deploy Pipeline & Domain

## Files Created

### `.github/workflows/deploy.yml`
Deploy workflow triggered on push to main. Single job with sequential steps:

```
name: Deploy
on: push to main
concurrency: deploy-$stage (cancel-in-progress: false — don't cancel running deploys)

env:
  STAGE: dev

jobs:
  deploy:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      1. Checkout
      2. Setup Rust toolchain (stable, aarch64-unknown-linux-gnu target)
      3. Rust cache
      4. Install cargo-zigbuild + zig
      5. Build Lambda (./scripts/build-lambda.sh)
      6. Setup Node 22 + pnpm 9
      7. Install SST dependencies (cd infra && npm install)
      8. SST deploy → capture API_URL output
      9. Install web dependencies (cd web && pnpm install --frozen-lockfile)
      10. Build SvelteKit (cd web && pnpm run build)
      11. Deploy CF Pages (cd web && npx wrangler pages deploy)
      12. Set Worker BACKEND_URL secret
      13. Install worker dependencies (cd worker && npm install)
      14. Deploy CF Worker
      15. Smoke test (continue-on-error: true)
      16. Report smoke result
```

Secrets required (GitHub repo secrets):
- `AWS_ACCESS_KEY_ID` — IAM user for SST/Lambda
- `AWS_SECRET_ACCESS_KEY` — IAM user for SST/Lambda
- `CLOUDFLARE_API_TOKEN` — Wrangler deploy + secret management
- `CLOUDFLARE_ACCOUNT_ID` — Wrangler account context

### `docs/active/work/T-018-02/setup-domain.md`
Manual domain setup guide covering:
- Cloudflare DNS for get-plantastic.com
- CF Pages custom domain binding (staging.get-plantastic.com)
- CF Worker custom route (optional, api.get-plantastic.com)
- SSL/TLS configuration (Full strict via Cloudflare)
- GitHub repo secrets configuration

## Files Modified

### `justfile`
Update the `deploy` recipe to include Lambda build and BACKEND_URL wiring:

```
deploy stage="dev":
    @echo "Building Lambda..."
    ./scripts/build-lambda.sh
    @echo "Deploying infrastructure..."
    cd infra && npx sst deploy --stage {{stage}}
    @echo "Building web..."
    cd web && pnpm run build
    @echo "Deploying web to CF Pages..."
    cd web && npx wrangler pages deploy .svelte-kit/cloudflare --project-name plantastic
    @echo "Deploying CF Worker..."
    cd worker && npx wrangler deploy
    @echo "Deployed to {{stage}}."
```

Add new `smoke` recipe:
```
smoke url:
    ./scripts/verify-deploy.sh {{url}}
```

## Files Unchanged

- `scripts/build-lambda.sh` — already correct
- `scripts/verify-deploy.sh` — already accepts any URL, works for Worker URL
- `scripts/migrate.sh` — not part of deploy pipeline (migrations are manual)
- `infra/sst.config.ts` — no changes needed
- `worker/wrangler.toml` — no changes needed
- `web/` — no changes needed
- `.github/workflows/ci.yml` — remains independent

## Module Boundaries

- **CI workflow** (`ci.yml`): validates code quality on PR and push. Does not deploy.
- **Deploy workflow** (`deploy.yml`): deploys on push to main. Does not run tests (CI handles that).
- **Build script** (`build-lambda.sh`): cross-compiles Rust. Used by both `just deploy` and deploy workflow.
- **Smoke script** (`verify-deploy.sh`): verifies deployment. Used by both `just smoke` and deploy workflow.
- **Justfile**: local developer interface. `deploy` for manual deploys, `smoke` for manual verification.

## Ordering Constraints

1. Lambda build must complete before SST deploy (SST bundles the built binary)
2. SST deploy must complete before CF Worker secret set (need the Function URL)
3. CF Worker secret must be set before Worker deploy (otherwise Worker proxies to wrong/no URL)
4. All deploys must complete before smoke test
5. Web build/deploy and SST deploy are independent but run sequentially for simplicity

## GitHub Actions Concurrency

Use `concurrency` group `deploy-dev` with `cancel-in-progress: false`. If two pushes happen in quick succession, the second waits for the first to finish rather than canceling it. Canceling a deploy mid-SST-deploy could leave CloudFormation in a bad state.
