---
id: T-021-03
story: S-021
title: lambda-connection-validation
type: task
status: open
priority: high
phase: done
depends_on: [T-021-01, T-017-01]
---

## Context

Validate that Lambda → Neon connections work reliably under real conditions (cold starts, concurrent invocations, idle periods). Update the E-008 deployment epic and related tickets to reference Neon instead of Railway.

## Acceptance Criteria

### Connection validation
- Deploy Lambda to us-west-2 (from T-017-01)
- Connect to Neon using pooled connection string
- Measure cold start time breakdown:
  - Lambda init (Rust binary startup)
  - Neon compute wake (if suspended)
  - Connection establishment (TLS handshake)
  - First query
- Test after 10-minute idle (Neon compute may suspend on free/launch tier)
- Test 10 concurrent cold starts (Lambda scaling)

### Resilience
- Verify retry logic from T-020-02 handles Neon cold starts
- Verify no hangs (the documented sqlx/tokio-postgres issue)
- Health endpoint returns 200 even on cold start within acceptable time (< 5 seconds total)

### Documentation updates
- Update E-008 epic: Railway → Neon in infrastructure table and description
- Update T-017-02: Railway-specific acceptance criteria → Neon equivalents
- Document connection timing results in work artifacts
- Note any tuning applied (suspend timeout, pool size, etc.)
