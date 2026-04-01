---
id: T-025-01
story: S-025
title: tenant-isolation-scenario
type: task
status: open
priority: high
phase: done
depends_on: [T-022-01]
---

## Context

S.INFRA.2 (Tenant isolation) is NotImplemented but 3/4 prereqs are delivered. The Axum routes already enforce `X-Tenant-Id` scoping (T-004-02), TenantRepo exists in pt-repo (T-003-02), and connection hardening is in place (T-020-02). The "pt-tenant" milestone may already be satisfied by existing code.

## Acceptance Criteria

- S.INFRA.2 scenario test implemented in `tests/scenarios/src/suites/infrastructure.rs`
- Test flow (requires DATABASE_URL):
  1. Create Tenant A (via TenantRepo or API)
  2. Create project as Tenant A → 201
  3. Fetch project as Tenant B → 404 (not 403, no existence leak)
  4. Create material as Tenant A → 201
  5. List materials as Tenant B → Tenant A's material not in list
  6. Create zone on Tenant A's project as Tenant B → 404
  7. Attempt tier assignment on Tenant A's project as Tenant B → 404
- Fallback: if no DATABASE_URL, return `Blocked("no DATABASE_URL")` or computation fallback
- Pass at ★★☆☆☆ integration (API-level isolation verified)
- Claim "pt-tenant" milestone in `progress.rs` with note explaining TenantRepo + X-Tenant-Id already deliver this
- `just check` passes

## Implementation Notes

- Reuse `api_helpers` module (scenario_pool, create_tenant, router, api_call) from quoting suite
- Each tenant gets a unique UUID-based name for isolation
- The test must NOT use the same tenant ID for cross-tenant checks
- 404 vs 403: the routes should return 404 to avoid leaking resource existence — verify this is the current behavior
