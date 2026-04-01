# Plan — T-008-02: Quote Scenario Two-Star

## Steps

### Step 1: Add dependencies to pt-scenarios Cargo.toml

Add: tokio, plantastic-api, pt-repo, sqlx, axum, tower, http-body-util, serde_json, uuid.

**Verify:** `cargo check -p pt-scenarios` compiles.

### Step 2: Create api_helpers.rs

Write the async helper module with:
- `scenario_pool()` — creates PgPool from DATABASE_URL
- `setup_db()` — runs migrations
- `create_tenant()` — inserts test tenant
- `router()` — builds API Router
- `api_call()` — sends request via tower::oneshot, returns (StatusCode, Value)

**Verify:** `cargo check -p pt-scenarios` still compiles.

### Step 3: Add mod declaration in main.rs

Add `mod api_helpers;` to make the module visible.

**Verify:** `cargo check -p pt-scenarios`

### Step 4: Rewrite S.3.1 scenario function

1. Rename old `s_3_1_quantity_from_geometry` to `s_3_1_unit_regression`
2. Write new `s_3_1_quantity_from_geometry`:
   - Check DATABASE_URL, return Blocked if absent
   - Create tokio runtime (current_thread)
   - POST /projects
   - POST 3 zones with GeoJSON polygons (12x15 patio, 8x20 bed, 10x10 edging)
   - POST 3 materials (Travertine $8.50/sq_ft, Premium Mulch $45/cu_yd, Steel Edge $3.25/lin_ft)
   - PUT /projects/{id}/tiers/better with 3 assignments
   - GET /projects/{id}/quote/better
   - Assert: 3 line items
   - Assert: patio line_total == "1530.00"
   - Assert: mulch line_total == "88.89"
   - Assert: edging line_total == "130.00"
   - Assert: subtotal == "1748.89"
   - Assert: total == "1748.89"
   - Return TwoStar

**Verify:** `cargo check -p pt-scenarios`

### Step 5: Rewrite S.3.2 scenario function

1. Rename old `s_3_2_three_tier_quotes` to `s_3_2_unit_regression`
2. Write new `s_3_2_three_tier_quotes`:
   - Check DATABASE_URL, return Blocked if absent
   - Create tokio runtime
   - POST /projects
   - POST 3 zones (same geometry as S.3.1)
   - POST 9 materials (3 per tier with escalating prices)
   - PUT /tiers/good, PUT /tiers/better, PUT /tiers/best
   - GET /quote/good, GET /quote/better, GET /quote/best
   - Assert: Good.total < Better.total < Best.total
   - Assert: subtotal == sum(line_totals) for each tier
   - Assert: exact totals: Good == "839.26", Best == "3148.40"
   - Assert: 3 line items per tier
   - Return TwoStar

**Verify:** `cargo check -p pt-scenarios`

### Step 6: Add unit regression test wrappers

Add `#[cfg(test)] mod tests` at bottom of quoting.rs:
- `#[test] fn s_3_1_regression()` — calls `s_3_1_unit_regression()`, asserts Pass
- `#[test] fn s_3_2_regression()` — calls `s_3_2_unit_regression()`, asserts Pass

**Verify:** `cargo test -p pt-scenarios` — regression tests pass.

### Step 7: Run quality gate

1. `just fmt` — format everything
2. `just lint` — clippy strict
3. `just test` — all workspace tests pass
4. `just scenarios` — run with DATABASE_URL set, verify S.3.1 and S.3.2 show TwoStar
5. `just scenarios` — run without DATABASE_URL, verify S.3.1 and S.3.2 show Blocked

## Testing Strategy

- **Unit regression:** Old computation tests run as `#[test]` in pt-scenarios
- **API integration:** New scenario functions exercise full HTTP path
- **Scenario dashboard:** S.3.1 and S.3.2 should show TwoStar (or Blocked if no DB)
- **No mocking:** Real Postgres, real API router, real pt-quote engine
- **Independent arithmetic:** Expected values computed in test, not by system

## Expected Dashboard Change

Before: S.3.1 = OneStar (5.0 eff min), S.3.2 = OneStar (3.0 eff min) = 8.0 eff min
After:  S.3.1 = TwoStar (10.0 eff min), S.3.2 = TwoStar (6.0 eff min) = 16.0 eff min
Delta:  +8.0 effective minutes
