# T-004-01 Plan: Axum Lambda Skeleton

## Step 1: Add workspace dependencies

- Add axum, tower-http, lambda_http, tracing, tracing-subscriber, dotenvy to workspace deps
- **Verify**: `cargo check` (workspace resolves)

## Step 2: Create crate skeleton + state + error

- `crates/plantastic-api/Cargo.toml` as binary crate
- `src/state.rs` — AppState with PgPool
- `src/error.rs` — AppError enum with IntoResponse + From<RepoError>
- `src/main.rs` — minimal stub (empty main)
- **Verify**: `cargo check -p plantastic-api`

## Step 3: Health route + router assembly

- `src/routes/health.rs` — GET /health handler
- `src/routes/mod.rs` — assembles Router with health routes + middleware (TraceLayer, CorsLayer)
- **Verify**: `cargo check -p plantastic-api`

## Step 4: Main function — Lambda/local dual mode

- Load .env via dotenvy
- Init tracing subscriber (JSON format for Lambda, pretty for local)
- Read DATABASE_URL, create PgPool, build AppState
- Build router
- Detect AWS_LAMBDA_RUNTIME_API: Lambda → lambda_http::run(), Local → axum serve on PORT
- **Verify**: `cargo build -p plantastic-api`

## Step 5: SST configuration

- Create `infra/sst.config.ts` with Lambda function definition
- provided.al2023, arm64, Function URL, RESPONSE_STREAM invoke mode
- DATABASE_URL environment variable from SSM
- **Verify**: File exists and is syntactically valid TypeScript

## Step 6: Create `.env.example`

- Template with DATABASE_URL and PORT
- **Verify**: File exists

## Step 7: Tests

- `tests/health_test.rs` — use axum::test helpers to send GET /health and verify response
- No database needed for health endpoint test
- **Verify**: `cargo test -p plantastic-api`

## Step 8: Claim milestone + quality gate

- Update progress.rs: claim "Axum API: routes + Lambda deployment" milestone with T-004-01
- Run `just check`
- **Verify**: all gates pass

## Testing Strategy

| Layer | What | DB required? |
|-------|------|-------------|
| Unit (health_test) | GET /health returns 200 + JSON | No |
| Manual | `cargo run -p plantastic-api` starts on PORT | Yes (pool init) |
| Future (T-004-02) | Full CRUD integration tests | Yes |
