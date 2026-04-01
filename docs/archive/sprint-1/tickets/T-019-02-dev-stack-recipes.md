---
id: T-019-02
story: S-019
title: dev-stack-recipes
type: task
status: open
priority: high
phase: done
depends_on: [T-019-01]
---

## Context

Wire Docker Compose into the justfile so developers have simple commands for the full local stack. Create `.env.example` documenting all environment variables.

## Acceptance Criteria

### Justfile recipes
- `just dev-db` — starts only the database (Compose `db` service), waits for health check, prints connection string
- `just dev-stack` — starts Compose services + API + worker + frontend (instructions for multi-terminal or background processes)
- `just dev-down` — stops all Compose services
- `just dev-reset` — `docker compose down -v && docker compose up -d` (clean slate)

### Environment config
- `.env.example` at project root with all variables:
  - `DATABASE_URL` (local Postgres)
  - `TEST_DATABASE_URL` (same, for tests)
  - `VALKEY_URL` (local Valkey)
  - `S3_ENDPOINT` (placeholder for R2 local, future)
  - `RUST_LOG` level
- `.env` added to `.gitignore` (if not already)
- Docker Compose reads from `.env` for any overrides

### Documentation
- Comment block at top of docker-compose.yml explaining the setup
- `just` with no arguments shows the new recipes in the list
