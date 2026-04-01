# T-004-01 Design: Axum Lambda Skeleton

## Decision 1: Crate Location and Name

### Options
**A) `crates/plantastic-api/`**: Matches justfile convention (`cargo run -p plantastic-api`). Consistent with `crates/*` workspace member pattern.
**B) `apps/api/`**: Separates apps from library crates. The spec mentions apps/ for binaries. But justfile already references `plantastic-api` as the package name.

### Decision: Option A — `crates/plantastic-api/`
The justfile already uses `cargo run -p plantastic-api` and `cargo build -p plantastic-api`. Keeping the package name consistent avoids breaking existing commands. The workspace `members = ["crates/*", "tests/scenarios"]` already includes it.

## Decision 2: Lambda Runtime Detection

### Options
**A) Check `AWS_LAMBDA_RUNTIME_API` env var**: Simple, standard. If set → Lambda mode, if not → local server.
**B) Feature flags**: `--features lambda` at build time. Forces two build paths.
**C) CLI arg**: `--lambda` flag. Lambda doesn't pass CLI args cleanly.

### Decision: Option A — env var detection at runtime
```rust
if std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
    run_lambda(router).await
} else {
    run_local(router, port).await
}
```
Single binary. Same build for local dev and Lambda. No feature-flag complexity. This is the proven pattern from HMW Workshop (Go equivalent).

## Decision 3: Application State

### Decision: Struct with PgPool
```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}
```
Axum extracts via `State(state): State<AppState>`. Created once in main(), shared across all handlers. Additional fields (e.g., S3 client, BAML runtime) can be added later without changing handler signatures.

## Decision 4: Error Handling

### Decision: AppError newtype wrapping RepoError + custom variants
```rust
pub enum AppError {
    NotFound,
    BadRequest(String),
    Conflict(String),
    Internal(String),
}
impl IntoResponse for AppError { ... }
```
Maps to HTTP status codes: NotFound→404, BadRequest→400, Conflict→409, Internal→500. Returns consistent JSON: `{"error": "message"}`.

`From<RepoError>` converts: RepoError::NotFound→404, Conflict→409, Database→500, Conversion→500.

## Decision 5: Middleware Stack

### Decision: Minimal for skeleton
1. **TraceLayer** (tower-http): Request/response logging with tracing
2. **CorsLayer** (tower-http): Permissive for local dev. In production the CF Worker handles CORS, so this is a safety net only.
3. **No auth middleware yet**: T-004-02 mentions tenant_id from header as placeholder. Auth is a separate concern.

## Decision 6: Health Endpoint

### Decision: GET /health returns JSON with version
```json
{"status": "ok", "version": "0.1.0"}
```
Version from `env!("CARGO_PKG_VERSION")`. No DB check in health — keep it fast. The worker proxies `/health` directly.

## Decision 7: SST Configuration

### Decision: TypeScript SST config in `infra/`
```typescript
// infra/sst.config.ts
export default $config({
  app(input) { ... },
  async run() {
    const api = new sst.aws.Function("Api", {
      runtime: "provided.al2023",
      architecture: "arm64",
      handler: "bootstrap",
      bundle: "target/lambda/plantastic-api",
      url: { invokeMode: "RESPONSE_STREAM" },
      environment: { DATABASE_URL: "..." },
    });
  }
});
```
Following HMW pattern. Function URL with RESPONSE_STREAM for future SSE support.

## Decision 8: Database URL Configuration

### Decision: `DATABASE_URL` env var with dotenv fallback
- Lambda: `DATABASE_URL` set via SST environment config (from SSM parameter)
- Local: Read from `.env` file via `dotenvy::dotenv().ok()` (fail silently if no .env)
- No hardcoded defaults — explicit config required

## What Was Rejected

- **Separate Lambda handler crate**: Over-engineering for a single binary.
- **Feature flags for Lambda vs local**: Adds build complexity. Env var detection is simpler and proven.
- **Auth middleware in skeleton**: Premature. T-004-02 will add tenant_id header extraction.
- **Database health check in /health**: Adds latency. Lambda health checks should be fast. DB connectivity is tested implicitly by every business route.
