# T-025-02 Structure: Full-Stack Round-Trip Scenario

## Files Modified

### 1. `tests/scenarios/src/suites/infrastructure.rs`

**Change**: Replace `s_infra_1_full_stack()` stub (lines 26-44) with full
implementation.

Add two functions:

```
fn s_infra_1_full_stack() -> ScenarioOutcome
  - Check DATABASE_URL env var
  - Build tokio runtime
  - block_on(s_infra_1_api())

async fn s_infra_1_api() -> ScenarioOutcome
  - Setup: scenario_pool, setup_db, create_tenant, router
  - Steps 1-9 as sequential api_call invocations
  - Return Pass(TwoStar, OneStar) on success
```

**No changes** to S.INFRA.2 or the SCENARIOS array.

### 2. `tests/scenarios/src/progress.rs`

**Change**: Claim three milestones by updating `delivered_by` and `note` fields.

- Line 172: `pt-project: Project/Zone/Tier model + GeoJSON serde`
  - `delivered_by: None` → `Some("T-025-02")`
  - Add descriptive note

- Line 178: `pt-quote: quantity takeoff engine`
  - `delivered_by: None` → `Some("T-025-02")`
  - Add descriptive note

- Line 305: `SvelteKit frontend + CF Worker proxy`
  - `delivered_by: None` → `Some("T-025-02")`
  - Add descriptive note

## Files NOT Modified

- `api_helpers.rs` — reuse as-is, no new helpers needed
- `registry.rs` — no new types needed
- `suites/mod.rs` — infrastructure already registered
- `Cargo.toml` — all dependencies already present
- No new files created

## Module Boundaries

The test function uses only:
- `crate::api_helpers` — pool, setup_db, create_tenant, router, api_call
- `crate::registry` — ScenarioOutcome, Integration, Polish
- `axum::http` — Method, StatusCode
- `serde_json` — json! macro, Value

No cross-crate calls. The test exercises the stack through HTTP only.

## Ordering

1. Implement `s_infra_1_api()` in infrastructure.rs
2. Implement `s_infra_1_full_stack()` wrapper
3. Claim milestones in progress.rs
4. Run `just check` to verify
