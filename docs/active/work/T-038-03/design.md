# T-038-03 Design: db-scenario-recipe

## Problem
Running DB-backed scenarios requires manually remembering the Docker Compose port (5433) and connection string. The existing port references in the justfile are stale (5432 vs 5433).

## Options considered

### Option A: Simple `just scenarios-db` with `docker compose up -d db --wait`
Use `docker compose up -d db --wait` which blocks until the healthcheck passes, then run scenarios with DATABASE_URL.

**Pros:** Minimal code, leverages existing Docker Compose healthcheck, one-liner.
**Cons:** `--wait` with a single service may behave inconsistently across Docker Compose versions.

### Option B: `just scenarios-db` with manual pg_isready loop
Start the DB with `docker compose up -d db`, then loop on `docker compose exec db pg_isready` until it succeeds, then run scenarios.

**Pros:** Explicit, works on all Docker Compose versions, matches the ticket's implementation note.
**Cons:** More code, but trivial (~5 lines of bash).

### Option C: Single recipe that detects Docker and auto-injects DATABASE_URL
Modify `just scenarios` to auto-detect a running Docker DB and inject DATABASE_URL if found.

**Pros:** Single command.
**Cons:** Breaks AC requirement that `just scenarios` without DB still shows BLOCKED. Magic behavior. Rejected.

## Decision: Option B (manual pg_isready loop)

Rationale:
1. Matches the ticket's implementation notes exactly
2. More portable across Docker Compose versions
3. Explicit health check with a retry loop is more debuggable
4. `docker compose up -d db` (no `--wait`) is simpler and well-tested

## Design details

### `just scenarios-db` recipe
```
scenarios-db:
    #!/usr/bin/env bash
    set -euo pipefail
    # Check Docker is available
    if ! command -v docker &>/dev/null; then
        echo "Docker not found. Install Docker to run DB-backed scenarios."
        echo "Alternatively, set DATABASE_URL and run: just scenarios"
        exit 1
    fi
    # Start DB (idempotent)
    echo "Starting Postgres..."
    docker compose up -d db
    # Wait for health
    echo "Waiting for Postgres to be ready..."
    for i in $(seq 1 30); do
        if docker compose exec db pg_isready -U plantastic -d plantastic &>/dev/null; then
            break
        fi
        if [ "$i" -eq 30 ]; then
            echo "Postgres did not become ready in 30 attempts."
            exit 1
        fi
        sleep 1
    done
    echo "Postgres ready."
    # Run scenarios with DATABASE_URL
    DATABASE_URL=postgres://plantastic:plantastic@localhost:5433/plantastic \
        cargo run -p pt-scenarios 2>/dev/null
```

### Port fix in existing recipes
Update `dev-db`, `dev-reset`, and `test-integration` to reference port 5433 (matching docker-compose.yml).

### `.env.example` update
Change `DATABASE_URL` and `TEST_DATABASE_URL` from port 5432 to 5433. Add a comment noting this matches Docker Compose.

### CLAUDE.md update
Add `just scenarios-db` to the Key Commands section.

### `scenarios-ci` — deferred
The ticket says "consider adding `just scenarios-ci`" but it's not in the AC. Skipping for now; can be a follow-up if needed.

## What won't change
- `just scenarios` — unchanged, infra scenarios still show BLOCKED without DATABASE_URL
- `docker-compose.yml` — no changes needed
- Scenario test code — no changes needed
