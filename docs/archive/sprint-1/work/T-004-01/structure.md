# T-004-01 Structure: Axum Lambda Skeleton

## New Crate: `crates/plantastic-api/`

### `crates/plantastic-api/Cargo.toml`
Binary crate (has main.rs). Dependencies: axum, tower-http (cors, trace), lambda_http, tracing, tracing-subscriber, tokio, serde, serde_json, pt-repo, dotenvy.

### Module Layout

```
crates/plantastic-api/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point: env detection, pool init, router build, Lambda/local dispatch
    ├── error.rs        # AppError enum, IntoResponse impl, From<RepoError>
    ├── state.rs        # AppState struct (PgPool)
    └── routes/
        ├── mod.rs      # Route tree assembly (merge all route groups)
        └── health.rs   # GET /health handler
```

### Integration Tests

```
crates/plantastic-api/tests/
└── health_test.rs      # Test health endpoint against real router (no DB needed)
```

## File Details

### `main.rs`
```
1. dotenv loading (dotenvy::dotenv().ok())
2. tracing subscriber init
3. DATABASE_URL from env
4. PgPool creation via pt_repo::create_pool()
5. AppState construction
6. Router building (with_state + middleware)
7. Lambda vs local detection and dispatch
```

### `error.rs`
- `AppError { NotFound, BadRequest(String), Conflict(String), Internal(String) }`
- `impl IntoResponse for AppError` → JSON `{"error": "..."}` with appropriate status code
- `impl From<pt_repo::RepoError> for AppError` — NotFound→404, Conflict→409, Database/Conversion→500

### `state.rs`
- `AppState { pool: PgPool }` — `#[derive(Clone)]`

### `routes/mod.rs`
- `pub fn router(state: AppState) -> Router` — assembles route tree
- Currently just health routes, but T-004-02 will add project/zone/material/tier groups

### `routes/health.rs`
- `pub fn routes() -> Router<AppState>` — returns Router with health endpoint
- `async fn health() -> impl IntoResponse` — returns `{"status":"ok","version":"0.1.0"}`

## Infrastructure: `infra/sst.config.ts`

SST v3 config defining:
- Lambda function (provided.al2023, arm64, RESPONSE_STREAM)
- Function URL (public, no auth — worker handles auth)
- DATABASE_URL from SSM parameter
- Bundle pointing to cross-compiled Rust binary

## Workspace Changes

### `Cargo.toml` (workspace root)
Add to `[workspace.dependencies]`:
```toml
axum = "0.8"
tower-http = { version = "0.6", features = ["cors", "trace"] }
lambda_http = "0.14"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
dotenvy = "0.15"
```

### `.env.example` (new)
```
DATABASE_URL=postgres://localhost:5432/plantastic_dev
PORT=3000
```

## Ordering Constraints
1. Workspace deps first
2. error.rs and state.rs (no internal deps)
3. routes/health.rs and routes/mod.rs
4. main.rs (ties everything together)
5. SST config (independent of Rust code)
6. Tests last
