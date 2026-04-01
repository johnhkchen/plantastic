---
id: T-021-02
story: S-021
title: ci-neon-branching
type: task
status: open
priority: high
phase: done
depends_on: [T-021-01]
---

## Context

Neon's copy-on-write branching creates an ephemeral database from the dev branch in <1 second. Use this in CI so integration tests run against real Postgres+PostGIS without Docker in GitHub Actions.

## Acceptance Criteria

### CI workflow
- GitHub Actions workflow step: create Neon branch from `dev` branch
- Branch name includes run ID for uniqueness (e.g., `ci-run-{run_id}`)
- `TEST_DATABASE_URL` set to the ephemeral branch's pooled connection string
- Integration tests run against the branch
- Branch deleted in a `post` or `always` cleanup step (even on test failure)

### Neon CLI / API
- Use `neonctl` CLI or Neon API for branch management
- Neon API token stored as GitHub Actions secret
- Branch creation timeout: fail CI if branch isn't ready in 30 seconds

### Verification
- CI integration tests pass against Neon branch
- Branch is cleaned up after every run (no orphaned branches accumulating)
- Total CI overhead from branching: < 10 seconds (creation + deletion)
- Integration tests + branching fits within the 10-minute CI budget (from T-018-01)
