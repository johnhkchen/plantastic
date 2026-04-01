# T-018-02 Review — Deploy Pipeline & Domain

## Summary

This ticket delivers the automated deployment pipeline for Plantastic. On every push to main, the system builds the Rust Lambda binary, deploys infrastructure via SST, ships the SvelteKit frontend to Cloudflare Pages, wires the Lambda Function URL into the CF Worker, deploys the Worker, and runs smoke tests through the Worker URL.

## Files Created

| File | Purpose |
|------|---------|
| `.github/workflows/deploy.yml` | GitHub Actions deploy workflow — triggered on push to main |
| `docs/active/work/T-018-02/setup-domain.md` | Manual setup guide for DNS, secrets, and CF projects |

## Files Modified

| File | Change |
|------|--------|
| `justfile` | Updated `deploy` recipe (added Lambda build, pnpm, correct working dirs); added `smoke` recipe |

## Acceptance Criteria Verification

### Deploy pipeline
- [x] `.github/workflows/deploy.yml` runs on push to main
- [x] Builds Rust for Lambda via `scripts/build-lambda.sh`
- [x] Deploys Lambda + S3 via `npx sst deploy --stage dev`
- [x] Deploys SvelteKit to CF Pages via `npx wrangler pages deploy`
- [x] Deploys CF Worker via `npx wrangler deploy`
- [x] Extracts Lambda Function URL and wires it to CF Worker's BACKEND_URL secret

### Domain
- [x] DNS setup documented in `setup-domain.md` (manual Cloudflare operation)
- [x] CF Pages custom domain binding documented (staging.get-plantastic.com)
- [x] SSL/TLS via Cloudflare documented (Full strict mode)

### Smoke tests
- [x] `scripts/verify-deploy.sh` runs after deploy in workflow
- [x] Tests health endpoint → 200
- [x] Tests create project → 201
- [x] Tests fetch project → 200 with matching data
- [x] Tests delete project → cleanup
- [x] Tests through CF Worker URL (captured from wrangler deploy output)
- [x] Smoke test failure warns but doesn't prevent deploy (continue-on-error + warning annotation)

### Updated recipes
- [x] `just deploy dev` runs full pipeline (build Lambda → SST → web → CF Pages → CF Worker)
- [x] `just smoke <url>` runs smoke tests against a deployed environment

## Quality Gate

```
just check → All gates passed
  fmt-check: pass
  lint: pass (0 warnings)
  test: 129 passed, 0 failed, 29 ignored
  scenarios: 58.0 / 240.0 min (24.2%) — unchanged from baseline
```

No regressions. This ticket is infrastructure-only (YAML workflow + shell recipes) — no Rust code was modified.

## Scenario Impact

This ticket does not directly advance any scenario. It is a deployment infrastructure ticket that unblocks live testing of all scenarios. Once the pipeline runs and smoke tests pass against a live environment, S.INFRA.1 (full stack round-trip) will be closer to green — the deploy pipeline is a prerequisite for validating the API end-to-end in a production-like environment.

No milestone claim is appropriate here — the deploy pipeline is operational tooling, not a capability crate.

## Test Coverage

No new Rust tests. The deploy workflow is tested by:
1. GitHub Actions workflow execution on first push to main
2. Smoke tests (`verify-deploy.sh`) validating the deployed stack end-to-end
3. `just deploy dev` for local manual verification

## Open Concerns

1. **GitHub secrets must be configured before first deploy.** Without `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `CLOUDFLARE_API_TOKEN`, and `CLOUDFLARE_ACCOUNT_ID`, the workflow will fail. See `setup-domain.md`.

2. **SST output parsing.** The `grep -oP 'apiUrl:\s*\K\S+'` pattern depends on SST v3's output format. If SST changes its output format, the extraction step will fail and the workflow will halt before wiring the wrong URL.

3. **Worker URL capture.** The `grep -oP 'https://\S+\.workers\.dev'` pattern captures the Worker URL from `wrangler deploy` output. If Wrangler changes its output format, the smoke test falls back to testing the direct Lambda URL (still useful but doesn't match the "test through Worker" requirement).

4. **Zig installation in CI.** Using `sudo snap install zig --classic --beta` — snap is available on ubuntu-latest but the `--beta` channel may lag behind or change. If this breaks, switch to downloading zig directly from ziglang.org/download.

5. **Concurrency group prevents parallel deploys.** If two pushes happen in quick succession, the second waits. This is intentional (protecting CloudFormation state) but means rapid-fire merges queue up.

6. **No database migration in the pipeline.** Migrations are run manually (`just migrate`). This is intentional per the project's approach — schema changes are deliberate, not automatic. But it means a deploy can succeed with code that expects a schema the database doesn't have yet.

7. **Domain DNS is a manual step.** The workflow deploys to CF Pages/Worker project names, not to custom domains. Custom domain binding is a one-time Cloudflare dashboard operation documented in `setup-domain.md`.
