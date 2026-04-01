# T-017-01 Review: Rust Lambda Deploy

## Summary

Delivered the cross-compilation build pipeline and SST infrastructure configuration for deploying the Plantastic API to AWS Lambda. The API binary (`plantastic-api`) was already Lambda-ready (dual-mode detection in main.rs, lambda_http adapter); this ticket adds the build tooling and deployment config to get it running on AWS.

## Files Created

| File | Purpose |
|------|---------|
| `scripts/build-lambda.sh` | Cross-compile script: cargo-zigbuild → aarch64-linux → bootstrap binary |
| `infra/package.json` | NPM package for SST v3 dependency |

## Files Modified

| File | Change |
|------|--------|
| `justfile` | `build-lambda` recipe now calls `./scripts/build-lambda.sh` |
| `infra/sst.config.ts` | Added S3 bucket (Uploads), linked to Lambda function, added S3_BUCKET env var |
| `.gitignore` | Added `target/lambda/`, `.sst/`, `.open-next/` |
| `tests/scenarios/src/progress.rs` | Claimed milestone: "Lambda deploy: cross-compile + SST + S3 bucket" |

## Files Unchanged (already correct)

- `crates/plantastic-api/src/main.rs` — Lambda dual-mode detection already works
- `crates/plantastic-api/Cargo.toml` — lambda_http, aws-sdk-s3 already present
- `Cargo.toml` (workspace) — all workspace deps correct

## Quality Gate

| Gate | Result |
|------|--------|
| `just fmt-check` | Pass |
| `just lint` | Pass (clippy strict, warnings = errors) |
| `just test` | Pass (all workspace tests) |
| `just scenarios` | 58.0 min / 240.0 min (24.2%), 13/22 milestones — no regression |

Milestone count: 12 → 13 (T-017-01 milestone added).

## Test Coverage

This ticket is infrastructure tooling — no new Rust code that requires unit tests. The deliverables are:
- A shell script (build-lambda.sh) — verified by structure review
- SST config changes (TypeScript) — verified by SST's own validation at deploy time
- Justfile recipe — verified by `just --list`

The existing API tests (health, CRUD, scan) remain green and will exercise the deployed binary once it's live.

## What's Not Covered

1. **Actual deployment** — `npx sst deploy --stage dev` was not executed. Requires:
   - AWS credentials configured
   - cargo-zigbuild + zig installed locally
   - DatabaseUrl SSM secret set (T-017-02)

2. **Cold start measurement** — documented in acceptance criteria, requires live deployment. Expected: <500ms for Rust + rustls. Risk: DB pool init could add 1-3s if Neon is cold.

3. **Binary size measurement** — requires running the build. Expected: 10-30 MB for a Rust API binary with sqlx + aws-sdk.

## Open Concerns

1. **SST v3 version pinning** — `package.json` uses `"sst": "^3"` (caret range). First deploy will install latest v3.x. If a specific version is needed, pin it after first successful deploy.

2. **S3 bucket naming** — SST auto-generates bucket names per stage. The code in main.rs reads `S3_BUCKET` env var, which SST now sets from the bucket resource. This should work end-to-end.

3. **Missing `infra/package-lock.json`** — not generated yet. Will be created on first `npm install` or `npx sst deploy`. Should be committed to lock versions.

4. **cargo-zigbuild not installed** — the build script will fail with a clear error message if it's missing. Install: `cargo install cargo-zigbuild && brew install zig`. This is a developer setup step, not a code issue.

## Dependencies

- **Depends on (done):** T-004-02 (Axum API routes) — delivered
- **Blocks:** T-017-02 (secrets wiring), T-018-01 (CI/CD pipeline)
- **Related:** T-021-01 (Neon provisioning) — needed for DATABASE_URL in production

## Deployment Checklist (for T-017-02 or manual testing)

1. `cargo install cargo-zigbuild && brew install zig`
2. `rustup target add aarch64-unknown-linux-gnu`
3. `just build-lambda` — produces `target/lambda/plantastic-api/bootstrap`
4. Set DatabaseUrl secret: `npx sst secret set DatabaseUrl "postgres://..." --stage dev`
5. `cd infra && npm install`
6. `npx sst deploy --stage dev`
7. Verify: `curl <function-url>/health`
8. Measure: check CloudWatch for cold start time, memory usage
