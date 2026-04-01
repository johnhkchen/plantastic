# T-025-01 Plan: Tenant Isolation Scenario

## Steps

### Step 1: Implement S.INFRA.2 scenario test

**File**: `tests/scenarios/src/suites/infrastructure.rs`

1. Add imports for `Integration`, `Polish` from registry.
2. Replace `s_infra_2_tenant_isolation()` with sync wrapper:
   - Check `DATABASE_URL` env var, return `Blocked` if absent.
   - Build tokio `current_thread` runtime.
   - `block_on(s_infra_2_api())`.
3. Implement `s_infra_2_api()` async function with 7-step test flow:
   - Setup: pool → migrations → two tenants → router.
   - Project isolation: create as A (201), fetch as B (404).
   - Material isolation: create as A (201), list as B (absent).
   - Zone isolation: create zone as A (201 setup), create zone as B on A's project (404).
   - Tier isolation: PUT tier assignment on A's project as B (404).
   - Return `Pass(TwoStar, OneStar)`.

**Verification**: `cargo build -p pt-scenarios` compiles. If DATABASE_URL is set, `cargo run -p pt-scenarios` shows S.INFRA.2 passing.

### Step 2: Claim pt-tenant milestone

**File**: `tests/scenarios/src/progress.rs`

1. Set `delivered_by: Some("T-025-01")` on the pt-tenant milestone (line 311).
2. Add note explaining: TenantRepo (T-003-02) provides the model, X-Tenant-Id extractor (T-004-02) provides auth context, verify_project_tenant enforces isolation at route level. T-025-01 verifies end-to-end with API-level scenario test covering projects, materials, zones, and tiers.

**Verification**: `cargo build -p pt-scenarios` compiles.

### Step 3: Run quality gate

Run `just check` (fmt + lint + test + scenarios).

- Expect S.INFRA.2 to show `NotImplemented` or `Blocked` (if no DATABASE_URL in CI) — not `Fail`.
- All other scenarios unchanged.
- No lint warnings, no format issues.

## Testing Strategy

- **Primary**: S.INFRA.2 scenario test itself IS the test. It exercises 7 API calls across 4 resource types.
- **Fallback**: Without DATABASE_URL, returns `Blocked` — this is correct behavior, not a failure.
- **Regression**: All existing scenarios must remain at same status or better.
- **No new unit tests**: The scenario test is an integration test. No isolated unit behavior to test separately.
