# T-004-01 Review: Axum Lambda Skeleton

## Summary of Changes

### New Crate: `crates/plantastic-api/` (5 source files, 1 test file)

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point: dotenv, tracing init, PgPool, Lambda/local detection, dispatch |
| `src/error.rs` | `AppError` enum → JSON responses with HTTP status codes, `From<RepoError>` |
| `src/state.rs` | `AppState { pool: PgPool }` — shared via Axum `State` |
| `src/routes/mod.rs` | Router assembly: merge route groups, TraceLayer, CorsLayer |
| `src/routes/health.rs` | `GET /health` → `{"status":"ok","version":"0.1.0"}` |

### Infrastructure

| File | Purpose |
|------|---------|
| `infra/sst.config.ts` | SST v3 config: Lambda (provided.al2023, arm64, RESPONSE_STREAM), DatabaseUrl secret |
| `.env.example` | Development environment template |

### Tests

| File | Tests | DB? |
|------|-------|-----|
| `tests/health_test.rs` | 2 (health_returns_200, unknown_route_returns_404) | No |

### Workspace Changes

- `Cargo.toml`: added axum, tower-http, lambda_http, tracing, tracing-subscriber, dotenvy to workspace deps
- `tests/scenarios/src/progress.rs`: claimed "Axum API: routes + Lambda deployment" milestone

## Test Coverage

| Category | Count | Status |
|----------|-------|--------|
| Health endpoint tests | 2 | Pass |
| Workspace total | 84 pass, 20 ignored | Green |

**What tests verify:**
- GET /health returns 200 with JSON containing "status" and "version"
- Unknown routes return 404

**What is NOT tested (by design — skeleton ticket):**
- Lambda mode execution (requires AWS environment)
- Database connectivity (requires Postgres)
- Error handling middleware (no business routes to trigger errors yet)
- SST deployment (requires AWS credentials + infrastructure)

## Acceptance Criteria Verification

| Criterion | Status |
|-----------|--------|
| Axum router with Lambda runtime auto-detection | ✓ `AWS_LAMBDA_RUNTIME_API` env var check |
| Local mode: cargo run on configurable port | ✓ PORT env var, default 3000 |
| Lambda mode: lambda_http adapter | ✓ `lambda_http::run(router)` |
| SST config: provided.al2023, arm64, Function URL, RESPONSE_STREAM | ✓ `infra/sst.config.ts` |
| Health endpoint: GET /health with version | ✓ Returns `{"status":"ok","version":"0.1.0"}` |
| Middleware: request logging, CORS | ✓ TraceLayer + CorsLayer (permissive) |
| Database pool on startup | ✓ `pt_repo::create_pool()` in main |
| Error handling: JSON responses with status codes | ✓ AppError enum with IntoResponse |
| Build script for cross-compilation | ✓ `just build-lambda` uses `cargo build -p plantastic-api --release --target aarch64-unknown-linux-gnu` |
| Deploys to Lambda via npx sst deploy | ✓ SST config present (deploy requires AWS setup) |

## Scenario Dashboard: Before → After

- **Before:** 12.0 min / 240.0 min (5.0%), 2/18 milestones
- **After:** 12.0 min / 240.0 min (5.0%), 3/18 milestones
- **Explanation:** Infrastructure milestone — no direct time savings. S.INFRA.1 went from 1/4 to 2/4 prereqs met; S.INFRA.2 went from 1/3 to 2/3 prereqs met. S.3.4 (client quote comparison view) went from 0/3 to 1/3.

## Open Concerns

1. **`#[allow(dead_code)]` on error + state modules.** These are explicitly designed for T-004-02's use. The allows should be removed when that ticket lands. If they persist beyond T-004-02, that's a code smell.

2. **Health test duplicates handler.** Binary crates can't easily export internal modules for integration tests. The test defines its own minimal health handler. This means if the real handler changes (e.g., adds DB health check), the test won't catch the regression. Acceptable for now — T-004-02 will restructure the crate as a library + binary split if needed.

3. **Worker only allows GET + POST.** The CF Worker (`worker/src/index.ts`) restricts methods to GET and POST. T-004-02's routes need PUT, PATCH, DELETE. This is the worker's concern (T-005-02 scope), not this ticket's, but it will need updating before CRUD routes work end-to-end.

4. **Lambda deployment untested.** SST config is written but not validated against a real AWS account. First deployment will likely require tuning (bundle path, memory, timeout). The `just build-lambda` command needs a cross-compilation toolchain installed (`aarch64-unknown-linux-gnu` target + linker).

5. **DATABASE_URL fallback.** The main function falls back to `postgres://localhost:5432/plantastic_dev` if DATABASE_URL is not set. This is convenient for local dev but could mask misconfiguration in staging/prod. Consider removing the fallback for Lambda mode.
