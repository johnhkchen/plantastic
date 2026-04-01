---
id: S-017
epic: E-008
title: Infrastructure Provisioning
status: open
priority: high
dependencies:
  - S-004
---

# S-017: Infrastructure Provisioning

## Purpose

Get the Rust API running on Lambda, connected to Railway PostGIS, with S3 for object storage. This is the production backbone — everything else (frontend, worker, CI) connects to it.

## Scope

- Rust cross-compilation to aarch64-unknown-linux-gnu (Lambda arm64)
- SST config: Lambda function (provided.al2023, arm64, RESPONSE_STREAM), S3 bucket
- Railway: enable PostGIS extension, apply migrations, extract DATABASE_URL
- Secrets pipeline: DATABASE_URL + any API keys into Doppler → SSM → Lambda env
- Verify: deploy, hit health endpoint, create+fetch a project through Railway PostGIS

## Tickets

- T-017-01: Rust cross-compilation + SST Lambda deployment
- T-017-02: Railway PostGIS setup + S3 bucket + secrets wiring
