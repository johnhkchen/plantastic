# T-018-02 Research — Deploy Pipeline & Domain

## What Exists

### CI Workflow (T-018-01, done)
`.github/workflows/ci.yml` runs on PR and push to main. Two parallel jobs:
- **Rust**: fmt-check → lint → test → scenarios (ubuntu-latest, stable toolchain, rust-cache)
- **Web**: pnpm install → svelte-check → lint (Node 22, pnpm 9)

No deploy workflow exists yet. The CI workflow does not build Lambda or deploy anything.

### Infrastructure (T-017-01/T-017-02, done)
`infra/sst.config.ts` — SST v3 config:
- App name: `plantastic`, region: `us-west-2`
- `sst.Secret("DatabaseUrl")` reads from SSM Parameter Store
- `sst.aws.Bucket("Uploads")` with CORS
- `sst.aws.Function("Api")` — `provided.al2023`, arm64, 256 MB, 30s timeout
  - Bundle: `target/lambda/plantastic-api`
  - Function URL with `RESPONSE_STREAM` invoke mode, no auth
  - Env: `DATABASE_URL`, `S3_BUCKET`, `RUST_LOG`
- Outputs: `apiUrl`, `bucketName`

`infra/package.json` depends on `sst` ^3.

### Build Script
`scripts/build-lambda.sh` — cross-compiles Rust for Lambda:
- Target: `aarch64-unknown-linux-gnu` via `cargo-zigbuild`
- Output: `target/lambda/plantastic-api/bootstrap`
- Requires: `cargo-zigbuild`, `zig`, rustup target `aarch64-unknown-linux-gnu`

### Smoke Test Script
`scripts/verify-deploy.sh` — takes API URL argument:
1. `GET /health` → 200
2. `POST /projects` → 201/200, extracts project ID
3. `GET /projects/{id}` → 200, verifies name match
4. `DELETE /projects/{id}` → cleanup
- Reports pass/fail counts, exits with fail count

Currently hits the Lambda URL directly. Ticket requires testing through CF Worker URL.

### Cloudflare Worker
`worker/wrangler.toml`:
- Name: `plantastic-api-proxy`
- Vars: `ALLOWED_ORIGIN=*`, rate limits (60/min IP, 200/session)
- Secret: `BACKEND_URL` (Lambda Function URL, set via `wrangler secret put`)

`worker/src/index.ts` proxies `/api/*` and `/health` to Lambda, handles CORS and rate limiting.

### SvelteKit Frontend
`web/` — SvelteKit with `@sveltejs/adapter-cloudflare`:
- Build output: `.svelte-kit/cloudflare/`
- Deploy target: CF Pages project `plantastic`
- Uses pnpm, Node 22

### Justfile Deploy Recipe (current)
```
deploy stage="dev":
    npx sst deploy --stage {{stage}}
    cd web && npm run build && npx wrangler pages deploy .svelte-kit/cloudflare --project-name plantastic
    cd worker && npx wrangler deploy
```
Missing: Lambda build step, BACKEND_URL wiring, smoke tests.

### Secrets & Auth Needed for CI
- **AWS**: SST deploy needs AWS credentials (IAM user or OIDC)
- **Cloudflare**: Wrangler needs `CLOUDFLARE_API_TOKEN` (or `CLOUDFLARE_ACCOUNT_ID` + token)
- **SST secrets**: `DatabaseUrl` already set in SSM per stage (T-017-02)
- **Doppler**: Not needed in CI — SST reads from SSM directly

### Domain
`get-plantastic.com` — referenced in spec as the domain. No DNS config exists yet.
Cloudflare manages DNS. CF Pages custom domains are set via dashboard or wrangler.

## Constraints & Risks

1. **Cross-compilation in CI**: `cargo-zigbuild` + `zig` must be installed in the GitHub Actions runner. This adds ~30s setup time.
2. **SST deploy duration**: First deploy provisions CloudFormation stack (~2-3 min). Subsequent deploys update Lambda code (~30-60s).
3. **Secret wiring order**: Lambda must deploy first to get Function URL, then CF Worker secret must be set before Worker deploy.
4. **CF Pages project**: Must exist before `wrangler pages deploy`. Created manually or via `wrangler pages project create`.
5. **Smoke test target**: Ticket says test through CF Worker URL, not direct Lambda. The Worker URL is `plantastic-api-proxy.<account>.workers.dev` or a custom domain.
6. **No rollback**: Ticket explicitly says smoke failure notifies but doesn't rollback. Manual investigation.
7. **Domain setup is manual**: DNS records and CF Pages custom domain binding are Cloudflare dashboard operations, not automatable in the workflow. The workflow documents what to do; the actual DNS setup is a one-time manual step.

## Patterns from T-018-01

The CI workflow uses:
- `actions/checkout@v4`
- `actions-rust-lang/setup-rust-toolchain@v1` (stable, clippy, rustfmt)
- `Swatinem/rust-cache@v2`
- `taiki-e/install-action@just`
- `pnpm/action-setup@v4` (version 9)
- `actions/setup-node@v4` (node 22, pnpm cache)

Deploy workflow should reuse the same action versions for consistency.

## What the Worker Secret Is Actually Called

In `worker/wrangler.toml`, the env interface expects `BACKEND_URL`. In `worker/src/index.ts`, the code reads `env.BACKEND_URL`. This is the secret name to set via `wrangler secret put BACKEND_URL`.

## SST Output Extraction

`npx sst deploy --stage dev` prints outputs to stdout. The `apiUrl` output contains the Lambda Function URL. Can be captured via:
```bash
API_URL=$(cd infra && npx sst deploy --stage dev 2>&1 | grep -o 'https://[^ ]*lambda-url[^ ]*')
```
Or more reliably via SST's output command after deploy:
```bash
API_URL=$(cd infra && npx sst output --stage dev apiUrl 2>/dev/null)
```
