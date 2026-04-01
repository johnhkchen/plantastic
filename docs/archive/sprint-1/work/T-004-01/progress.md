# T-004-01 Progress: Axum Lambda Skeleton

## Completed Steps

### Step 1: Workspace dependencies
- Added axum (0.8), tower-http (0.6, cors+trace), lambda_http (0.14), tracing (0.1), tracing-subscriber (0.3, env-filter+json), dotenvy (0.15) to workspace deps
- **Verified**: `cargo check` resolves all dependencies

### Step 2: Crate skeleton + state + error
- Created `crates/plantastic-api/` as binary crate
- `src/state.rs`: AppState with PgPool
- `src/error.rs`: AppError enum (NotFound/BadRequest/Conflict/Internal) with IntoResponse + From<RepoError>
- Added sqlx as direct dependency (needed for PgPool type in state.rs)
- **Verified**: `cargo check -p plantastic-api` compiles

### Step 3: Health route + router assembly
- `src/routes/health.rs`: GET /health → `{"status":"ok","version":"0.1.0"}`
- `src/routes/mod.rs`: Router assembly with TraceLayer + CorsLayer (permissive)
- **Verified**: compiles

### Step 4: Main function — Lambda/local dual mode
- dotenv loading (dotenvy::dotenv().ok())
- Tracing init: JSON format for Lambda, pretty format for local
- DATABASE_URL from env with localhost fallback + warning
- PgPool creation via pt_repo::create_pool()
- AWS_LAMBDA_RUNTIME_API detection: Lambda → lambda_http::run(), Local → axum::serve on PORT
- **Verified**: `cargo build -p plantastic-api`

### Step 5: SST configuration
- Created `infra/sst.config.ts`: Lambda function (provided.al2023, arm64, RESPONSE_STREAM)
- DatabaseUrl as SST Secret, RUST_LOG env var
- **Verified**: file exists

### Step 6: .env.example
- Created with DATABASE_URL and PORT templates
- **Verified**: file exists

### Step 7: Tests
- `tests/health_test.rs`: 2 tests (health_returns_200, unknown_route_returns_404)
- Uses axum test helpers + tower::ServiceExt::oneshot — no database needed
- **Verified**: `cargo test -p plantastic-api` — 2 tests pass

### Step 8: Milestone + quality gate
- Claimed "Axum API: routes + Lambda deployment" milestone (T-004-01)
- `just check` green: fmt ✓, lint ✓, test ✓ (84 pass, 20 ignored), scenarios ✓
- Dashboard: 3/18 milestones, S.INFRA.1 2/4 prereqs, S.INFRA.2 2/3 prereqs

## Deviations from Plan

1. **Added sqlx to direct deps**: state.rs needs `PgPool` type. Could have re-exported from pt-repo but direct dependency is cleaner.
2. **`#[allow(dead_code)]` on error + state modules**: These are designed for T-004-02's route handlers. Clippy strict (warnings=errors) would fail without the allow. Will be removed when T-004-02 uses them.
3. **Health test duplicates handler**: Rather than importing from the binary crate (complex with main.rs binaries), the test defines its own health handler. Functionally equivalent and avoids binary/library crate complexity.

## Remaining
None — all steps complete.
