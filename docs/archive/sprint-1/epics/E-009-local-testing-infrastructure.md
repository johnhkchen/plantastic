---
id: E-009
title: Local Testing Infrastructure
status: open
sprint: 3
---

# E-009: Local Testing Infrastructure

## Goal

Stand up a portable, Docker-Compose-based local development environment that mirrors production closely enough that passing local integration tests gives high confidence in deployed behavior. Migrate from Railway to Neon for managed PostgreSQL. Un-ignore all integration tests and make them part of the normal quality gate.

The local stack must be transferable — `docker compose up && just test-integration` works on any machine with Docker and Rust installed.

## Production → Local Mapping

| Layer | Production | Local | Parity |
|-------|-----------|-------|--------|
| Database | Neon PG 17 + PostGIS 3.5 | Docker: PG 18 + PostGIS 3.6 | Same engine, same extensions, same migrations |
| Object storage | Cloudflare R2 / S3 | R2 local via `wrangler dev` | Same API (S3-compatible) |
| Cache | Valkey (when needed) | Docker: Valkey | Identical |
| API | Lambda arm64 (Axum) | Native Axum (`just dev-api`) | Same binary, auto-detects mode |
| Edge proxy | CF Worker | `wrangler dev` | Same code |
| Frontend | CF Pages | `npm run dev` | Same SvelteKit app |
| CI database | Neon branch (ephemeral) | — | Real Postgres, instant setup |

## What changes from the current state

1. **Railway → Neon** for managed Postgres. Gains: PostGIS as first-class extension, database branching for CI, built-in connection pooling, Lambda co-location (us-west-2).
2. **Docker Compose** for local infrastructure (Postgres + PostGIS, Valkey). Portable to any dev machine.
3. **`#[sqlx::test]`** replaces manual test DB setup. Each integration test gets an ephemeral database with migrations auto-applied.
4. **All `#[ignore]` tests un-ignored** — integration tests become part of the normal test flow when a database is available.
5. **Neon branching** for CI — GitHub Actions creates an ephemeral branch per run, runs integration tests, deletes it.
6. **sqlx connection hardening** — explicit timeouts, retry logic, pooled connection strings for Neon cold-start resilience.

## Stories

- **S-019**: Docker Compose local stack — Postgres 18 + PostGIS 3.6, Valkey, justfile recipes
- **S-020**: Integration test framework — `#[sqlx::test]`, un-ignore tests, connection hardening
- **S-021**: Neon migration & CI — provision Neon, branching for CI, update deployment epic

## Success Criteria

- `docker compose up -d && just test-integration` passes on a clean machine
- All previously `#[ignore]`'d Postgres tests run and pass
- `just dev-stack` starts the full local environment (Postgres, Valkey, API, Worker, Frontend)
- CI integration tests run against an ephemeral Neon branch
- No SQLite, no mocks, no in-memory fakes for any database test
- Neon connection from Lambda handles cold starts gracefully (timeout + retry)
