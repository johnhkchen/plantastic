# T-025-01 Progress: Tenant Isolation Scenario

## Completed

### Step 1: Implement S.INFRA.2 scenario test
- Modified `tests/scenarios/src/suites/infrastructure.rs`
- Replaced `NotImplemented` stub with full 7-step API-level test
- Sync wrapper checks DATABASE_URL → builds tokio runtime → runs async test
- Async test creates two tenants and verifies cross-tenant access returns 404 for:
  - GET project, POST zone, PUT tier assignment (all via verify_project_tenant)
  - GET materials list (via list_by_tenant scoping)

### Step 2: Claim pt-tenant milestone
- Updated `tests/scenarios/src/progress.rs` line 310-314
- Set `delivered_by: Some("T-025-01")`
- Added note explaining TenantRepo + X-Tenant-Id + verify_project_tenant deliver isolation

### Step 3: Run quality gate
- `just check` passes — all gates green
- S.INFRA.2 shows as BLOCKED (no DATABASE_URL in local env) — correct behavior
- S.INFRA.2 prereqs now 4/4 met (pt-tenant milestone claimed)
- No regressions: 69.5 min / 240.0 min (29.0%) — unchanged from baseline
- Milestones: 16/24 delivered (pt-tenant now included)
