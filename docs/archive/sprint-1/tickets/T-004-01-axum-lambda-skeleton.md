---
id: T-004-01
story: S-004
title: axum-lambda-skeleton
type: task
status: open
priority: high
phase: done
depends_on: [T-003-02]
---

## Context

The Axum API is the single backend entry point. It needs to run both locally (for development) and on Lambda (for production) from the same binary. SST manages the Lambda deployment. This ticket sets up the skeleton — routing, middleware, Lambda adapter, SST config, health endpoint — without business logic routes.

## Acceptance Criteria

- Axum router with Lambda runtime auto-detection (check AWS_LAMBDA_RUNTIME_API env var)
- Local mode: `cargo run` starts HTTP server on configurable port
- Lambda mode: uses lambda_http adapter for API Gateway / Function URL
- SST config (infra/sst.config.ts): Lambda function with provided.al2023, arm64, Function URL, RESPONSE_STREAM invoke mode
- Health endpoint: GET /health returns 200 with version info
- Middleware: request logging, CORS (permissive for dev, locked down via worker in prod)
- Database pool initialization on startup (connection string from env/SSM)
- Error handling: consistent JSON error responses with status codes
- Build script for cross-compilation (cargo build --target aarch64-unknown-linux-gnu)
- Deploys to Lambda via `npx sst deploy` and responds to health check
