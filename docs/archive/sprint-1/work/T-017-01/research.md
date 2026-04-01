# T-017-01 Research: Rust Lambda Deploy

## Current State

### API Binary (crates/plantastic-api/)

The Axum API already supports Lambda deployment via dual-mode detection in `main.rs`:
- Checks `AWS_LAMBDA_RUNTIME_API` env var to decide Lambda vs local mode
- Lambda mode: calls `lambda_http::run(router)` — the adapter is already a dependency (`lambda_http = "0.14"`)
- Local mode: binds to `PORT` (default 3000) with `tokio::net::TcpListener`
- Tracing: JSON format for CloudWatch in Lambda, pretty format for local
- Health endpoint at `GET /health` returns `{"status":"ok","version":"0.1.0"}`

Dependencies relevant to Lambda:
- `lambda_http = "0.14"` (AWS Lambda HTTP adapter)
- `aws-sdk-s3 = "1"`, `aws-config = "1"` (S3 client, auto-configured in Lambda)
- `sqlx = "0.8"` with `rustls` TLS (no native OpenSSL dependency — good for cross-compile)
- `tokio` with `rt-multi-thread` and `macros`

The binary name is `plantastic-api` (defined in Cargo.toml `[[bin]]`). Lambda custom runtime expects the binary to be named `bootstrap`.

### SST Config (infra/sst.config.ts)

Already fully configured:
- `runtime: "provided.al2023"` — Amazon Linux 2023 custom runtime
- `architecture: "arm64"` — matches aarch64 cross-compile target
- `handler: "bootstrap"` — expects binary named `bootstrap`
- `bundle: "target/lambda/plantastic-api"` — SST looks for the bootstrap binary here
- `memory: "256 MB"`, `timeout: "30 seconds"`
- `url.invokeMode: "RESPONSE_STREAM"` — enables SSE via Function URL
- `environment.DATABASE_URL` wired from `sst.Secret("DatabaseUrl")`
- `environment.RUST_LOG` set to `plantastic_api=info,pt_repo=info,warn`
- Region: `us-west-2`

### Justfile Recipes

- `build-lambda`: exists but incomplete — runs bare `cargo build --target aarch64-unknown-linux-gnu`
  - No cross-compilation toolchain specified (will fail on macOS without linker)
  - Doesn't rename binary to `bootstrap` or place it in `target/lambda/plantastic-api/`
- `deploy stage="dev"`: calls `npx sst deploy --stage {{stage}}` then deploys web + worker

### Missing Infrastructure

- No `scripts/` directory exists
- No `package.json` in `infra/` for SST dependencies
- No `.cargo/config.toml` for cross-compile linker configuration
- No `target/lambda/` staging directory setup

## Cross-Compilation Options

### Option A: cargo-zigbuild (preferred per ticket)

Uses Zig's C compiler as a drop-in cross-linker. No Docker required.
- Install: `cargo install cargo-zigbuild` + `brew install zig`
- Usage: `cargo zigbuild -p plantastic-api --release --target aarch64-unknown-linux-gnu`
- Pros: fast, no Docker overhead, works natively on macOS (both Intel and Apple Silicon)
- Cons: requires zig installed, occasional edge cases with system libraries
- This project uses `rustls` (not OpenSSL), so zig-based cross-compile should work cleanly

### Option B: cross (Docker-based)

Uses Docker containers with pre-configured toolchains.
- Install: `cargo install cross`
- Usage: `cross build -p plantastic-api --release --target aarch64-unknown-linux-gnu`
- Pros: hermetic build environment, handles system libs automatically
- Cons: requires Docker running, slower, heavier

### Option C: Native cargo with system linker

Requires manually installing `aarch64-linux-gnu-gcc` and configuring `.cargo/config.toml`.
- On macOS: `brew install aarch64-elf-gcc` or similar — fragile, not well-supported
- Not recommended

## Key Constraints

1. **Binary must be named `bootstrap`** — Lambda custom runtime convention
2. **Binary must be at `target/lambda/plantastic-api/bootstrap`** — SST bundle path
3. **No OpenSSL dependency** — sqlx uses rustls, so no native lib cross-compile issues
4. **SST needs npm/npx** — `infra/` likely needs a `package.json` with `sst` as a dependency
5. **Rustup target** — `rustup target add aarch64-unknown-linux-gnu` needed regardless of approach
6. **Cold start budget** — ticket flags 5s threshold; Rust Lambda cold starts are typically 50-200ms for simple binaries, but DB pool init could add latency

## Relevant Patterns from HMW Workshop

The ticket references prior art at `/Volumes/ext1/swe/repos/how-might-we/`. Key patterns:
- Go binary cross-compiled with `zig cc` as the C compiler
- SST v3 config with Function URL and RESPONSE_STREAM
- Same `provided.al2023` + `arm64` pattern
- The Rust equivalent is `cargo-zigbuild` which uses the same zig toolchain

## Environment Variables for Lambda

From `main.rs` and SST config, the Lambda function needs:
- `DATABASE_URL` — from SSM via `sst.Secret("DatabaseUrl")`
- `RUST_LOG` — hardcoded in SST config
- `S3_BUCKET` — currently NOT in SST config (defaults to "plantastic-dev" in code)
- `AWS_LAMBDA_RUNTIME_API` — set automatically by Lambda runtime
- `AWS_REGION` — set automatically by Lambda

## File Inventory

Files to create:
- `scripts/build-lambda.sh` — build script
- `infra/package.json` — SST dependency

Files to modify:
- `justfile` — update `build-lambda` recipe
- `infra/sst.config.ts` — possibly add S3 bucket resource + env var

Files that need no changes:
- `crates/plantastic-api/src/main.rs` — already Lambda-ready
- `crates/plantastic-api/Cargo.toml` — dependencies correct
- `Cargo.toml` (workspace) — workspace deps correct
