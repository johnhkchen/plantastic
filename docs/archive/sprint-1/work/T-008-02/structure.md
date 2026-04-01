# Structure — T-008-02: Quote Scenario Two-Star

## Files Modified

### 1. `tests/scenarios/Cargo.toml`

Add dependencies needed for API-level scenario tests:

```
tokio        — async runtime (current_thread + io + time)
plantastic-api — router() + AppState for in-process HTTP
pt-repo      — create_pool(), tenant::create(), setup helpers
sqlx         — PgPool type
axum         — Router, Request, Body, StatusCode
tower        — ServiceExt::oneshot()
http-body-util — BodyExt for response body collection
serde_json   — JSON request/response construction
uuid         — Uuid type for IDs
```

### 2. `tests/scenarios/src/api_helpers.rs` (NEW)

Module providing async helpers for API-level scenario tests. ~80 lines.

**Public interface:**
```rust
pub async fn scenario_pool() -> Result<PgPool, String>
pub async fn setup_db(pool: &PgPool) -> Result<(), String>
pub async fn create_tenant(pool: &PgPool, name: &str) -> Result<Uuid, String>
pub fn router(pool: PgPool) -> Router
pub async fn api_call(
    app: &Router, method: Method, uri: &str,
    tenant_id: Uuid, body: Option<Value>,
) -> Result<(StatusCode, Value), String>
```

All functions return `Result` rather than panicking, so callers can convert errors
to `ScenarioOutcome::Fail` or `ScenarioOutcome::Blocked`.

### 3. `tests/scenarios/src/main.rs`

Add `mod api_helpers;` declaration.

### 4. `tests/scenarios/src/suites/quoting.rs`

**Structural changes:**

1. Rename `s_3_1_quantity_from_geometry` -> `s_3_1_unit_regression` (private)
2. Rename `s_3_2_three_tier_quotes` -> `s_3_2_unit_regression` (private)
3. New `s_3_1_quantity_from_geometry` — API-based, returns TwoStar or Blocked
4. New `s_3_2_three_tier_quotes` — API-based, returns TwoStar or Blocked
5. Add `#[cfg(test)] mod tests` with unit test wrappers for the regression functions

**Scenario entries unchanged** — still point to `s_3_1_quantity_from_geometry` and
`s_3_2_three_tier_quotes`, but now those names resolve to the new API-based implementations.

**New function flow (S.3.1):**
```
check DATABASE_URL -> create tokio runtime -> block_on async {
  create pool -> setup db -> create tenant -> build router ->
  POST /projects -> POST 3 zones (GeoJSON) -> POST 3 materials ->
  PUT /projects/{id}/tiers/better (3 assignments) ->
  GET /projects/{id}/quote/better ->
  assert line items, totals, subtotal
} -> TwoStar | Blocked | Fail
```

**New function flow (S.3.2):**
```
same setup, but 9 materials and 3 PUT tier assignments ->
GET /quote/good, GET /quote/better, GET /quote/best ->
assert Good < Better < Best, subtotal integrity, exact totals
```

## Files NOT Modified

- `crates/pt-quote/src/engine.rs` — unit tests remain untouched
- `crates/plantastic-api/tests/common/mod.rs` — API test helpers unchanged
- `crates/plantastic-api/tests/crud_test.rs` — existing integration test unchanged
- `tests/scenarios/src/progress.rs` — milestones already claimed
- `tests/scenarios/src/registry.rs` — no type changes needed

## Module Boundaries

```
tests/scenarios/
  src/
    main.rs          ← adds `mod api_helpers`
    api_helpers.rs   ← NEW: async helpers for API scenarios
    suites/
      quoting.rs     ← MODIFIED: new API-based scenarios + old as regression tests
```

The api_helpers module is used only by quoting.rs (and future TwoStar+ scenarios
in other suites). It depends on plantastic-api, pt-repo, axum, tower, and tokio.

## Ordering

1. Cargo.toml deps first (everything else depends on compilation)
2. api_helpers.rs (quoting.rs imports from it)
3. main.rs mod declaration (makes api_helpers visible)
4. quoting.rs rewrite (the main work)
