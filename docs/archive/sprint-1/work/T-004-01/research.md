# T-004-01 Research: Axum Lambda Skeleton

## Current State

### No API binary exists yet
- No `apps/` directory with Rust binaries
- No axum, lambda_http, lambda_runtime, tower, or tower-http in Cargo.lock
- The justfile references `plantastic-api` (`cargo run -p plantastic-api`, `cargo build -p plantastic-api --release --target aarch64-unknown-linux-gnu`) but the crate doesn't exist
- `just build-lambda` expects to find `plantastic-api` crate

### Repository layer ready (T-003-02)
- `crates/pt-repo` provides all CRUD operations with `PgPool`-accepting functions
- `pt_repo::create_pool(database_url)` returns Lambda-tuned PgPool
- All repos: tenant, project, zone, material, tier_assignment
- `RepoError` maps to HTTP-friendly errors (NotFound, Conflict, Database, Conversion)

### Worker proxy exists (T-005-02 done)
- `worker/src/index.ts` — Cloudflare Worker proxies to `BACKEND_URL`
- Routes: `/api/*` and `/health` are proxied
- CORS handled at worker level (permissive `*` in dev, locked to CF Pages URL in prod)
- Methods allowed: GET and POST (no PUT, PATCH, DELETE)
  - **Issue**: T-004-02 requires PUT, PATCH, DELETE routes but worker only allows GET/POST. This is a worker concern, not this ticket.

### Infrastructure directory
- `infra/` exists with only `.gitkeep` — SST config needs to be created here

### Spec references
- Infrastructure table: "Rust (Axum) on AWS Lambda, provided.al2023, arm64, RESPONSE_STREAM"
- Lambda binary includes Axum router + all domain crates + BAML runtime + Lambda streaming adapter
- IaC: SST (AWS CDK abstraction)
- Secrets: Doppler (dev) + AWS SSM (prod)

### HMW Workshop pattern (reference)
- Go + BAML on Lambda (provided.al2023, arm64, RESPONSE_STREAM)
- SST for IaC
- Stateless backend, streaming SSE
- Plantastic replaces Go with Rust but follows same deployment pattern

## Key Dependencies

### Rust crates needed
- `axum` — HTTP framework, router, middleware
- `tower-http` — CORS layer, trace layer (request logging)
- `lambda_http` — Lambda HTTP adapter (converts API Gateway events to http::Request)
- `tokio` — already in workspace
- `tracing` + `tracing-subscriber` — structured logging
- `serde` + `serde_json` — already in workspace

### SST / TypeScript
- `sst` — IaC framework (infra/sst.config.ts)
- Defines Lambda function, Function URL, environment variables

## Architecture Constraints

1. **Single binary, dual mode**: Same `main()` detects Lambda vs local via `AWS_LAMBDA_RUNTIME_API` env var. Lambda mode → lambda_http adapter. Local mode → `tokio::net::TcpListener`.

2. **Workspace member**: Lives in `crates/plantastic-api/` (matching justfile convention) or `apps/api/`. Justfile already references `plantastic-api` as package name.

3. **Shared PgPool**: Created once in `main()`, passed to router as Axum `State`. All handlers receive it via `State(pool)`.

4. **No business routes yet**: This ticket is skeleton only. T-004-02 adds CRUD routes. This ticket delivers: health endpoint, middleware, error handling, Lambda adapter, SST config.

5. **CORS at worker level**: The CF Worker handles CORS. The API itself needs permissive CORS for local dev only (when hitting the API directly without the worker). tower-http CorsLayer for dev mode.

## Existing Patterns in Codebase

- Error types follow `enum + Display + Error` pattern (see ProjectError, RepoError)
- Testing: `pt-test-utils` for timeouts, all integration tests against real Postgres
- All crates use workspace lints configuration
- snake_case serde throughout
