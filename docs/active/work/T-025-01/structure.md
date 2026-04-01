# T-025-01 Structure: Tenant Isolation Scenario

## Files Modified

### 1. `tests/scenarios/src/suites/infrastructure.rs`

**Current**: 55 lines, two stub scenarios returning `NotImplemented`.

**Changes**:
- Add imports: `Integration`, `Polish` from registry; `serde_json::json`; `axum::http::{Method, StatusCode}`.
- Replace `s_infra_2_tenant_isolation()` body:
  - Check `DATABASE_URL` env var → return `Blocked` if absent.
  - Build tokio current-thread runtime.
  - `block_on(s_infra_2_api())`.
- Add new `async fn s_infra_2_api() -> ScenarioOutcome`:
  - Setup: `scenario_pool()` → `setup_db()` → `create_tenant()` x2 → `router()`.
  - Step 1: POST /projects as Tenant A → assert 201, capture project_id.
  - Step 2: GET /projects/:id as Tenant B → assert 404.
  - Step 3: POST /materials as Tenant A → assert 201, capture material_id.
  - Step 4: GET /materials as Tenant B → assert Tenant A's material not in list.
  - Step 5: POST /projects/:id/zones as Tenant A → assert 201 (setup for step 6).
  - Step 6: POST /projects/:id/zones as Tenant B → assert 404.
  - Step 7: PUT /projects/:id/tiers/good as Tenant B → assert 404.
  - Return `Pass(Integration::TwoStar, Polish::OneStar)`.

**Estimated size**: ~120 lines for the async function + ~15 lines for the sync wrapper.

### 2. `tests/scenarios/src/progress.rs`

**Current**: Line 310-314, pt-tenant milestone with `delivered_by: None`.

**Change**: Set `delivered_by: Some("T-025-01")` and add descriptive note explaining what's verified.

## Files NOT Modified

- `tests/scenarios/src/suites/mod.rs` — infrastructure module already registered.
- `tests/scenarios/src/registry.rs` — no new enums or types needed.
- `tests/scenarios/src/api_helpers.rs` — existing helpers sufficient.
- `tests/scenarios/Cargo.toml` — no new dependencies needed.
- No new files created.

## Module Boundaries

- `s_infra_2_api()` is a private async function within `infrastructure.rs`.
- All external dependencies flow through `api_helpers` (pool, setup, tenant, router, api_call).
- No new public interfaces introduced.

## Ordering

1. Implement `infrastructure.rs` changes first (the test).
2. Update `progress.rs` milestone (the claim).
3. Run `just check` to verify.
