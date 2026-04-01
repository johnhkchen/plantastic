---
id: T-038-03
story: S-038
title: db-scenario-recipe
type: task
status: open
priority: high
phase: ready
depends_on: [T-038-01, T-038-02]
---

## Context

Running scenarios with DATABASE_URL requires remembering the Docker Compose port and connection string. Add a `just` recipe that starts the DB and runs scenarios in one command.

## Acceptance Criteria

- `just scenarios-db` recipe:
  1. Starts Docker Compose DB (if not running)
  2. Waits for health check
  3. Runs `DATABASE_URL=postgres://plantastic:plantastic@localhost:5433/plantastic just scenarios`
  4. Prints summary
- `just scenarios` (without DB) still works and shows BLOCKED for API scenarios (no regression)
- Update .env.example with the Docker Compose DATABASE_URL on port 5433
- Document in CLAUDE.md: `just scenarios-db` for full DB-backed scenario run

## Implementation Notes

- Use `docker compose up -d db` (idempotent if already running)
- Health check wait: `docker compose exec db pg_isready` in a loop
- The recipe should not fail if Docker isn't available — just print a helpful message
- Consider adding `just scenarios-ci` for CI with Neon branching (T-021-02 pattern)
