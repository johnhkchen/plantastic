---
id: T-017-01
story: S-017
title: rust-lambda-deploy
type: task
status: open
priority: high
phase: done
depends_on: [T-004-02]
---

## Context

Get the Rust API binary running on AWS Lambda. This requires cross-compiling for aarch64-linux-gnu (Lambda's arm64 runtime) and configuring SST to deploy it. The HMW Workshop used zig cc for Go cross-compilation — we need the Rust equivalent.

Prior art: HMW Workshop's build.sh and sst.config.ts at /Volumes/ext1/swe/repos/how-might-we/

## Acceptance Criteria

- Cross-compilation target: aarch64-unknown-linux-gnu
- Build approach: either `cross` (Docker-based) or `cargo-zigbuild` (zig cc, no Docker — preferred)
- Build script: `scripts/build-lambda.sh` that produces a `bootstrap` binary
- SST config (infra/sst.config.ts):
  - Lambda function: provided.al2023, arm64, 256MB memory (start conservative)
  - Function URL with RESPONSE_STREAM invoke mode (for SSE)
  - Environment: DATABASE_URL from SSM, S3 bucket name
- `just build-lambda` recipe updated to call the build script
- Deploy: `npx sst deploy --stage dev` succeeds
- Health endpoint responds at the Lambda Function URL
- Measure and document: cold start time, binary size, memory usage
- If cold start > 5s, investigate and document (connection pool init, binary size)
