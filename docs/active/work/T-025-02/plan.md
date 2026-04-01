# T-025-02 Plan: Full-Stack Round-Trip Scenario

## Step 1: Implement s_infra_1_full_stack() and s_infra_1_api()

Replace the NotImplemented stub in `infrastructure.rs` with:

**Wrapper function** (`s_infra_1_full_stack`):
- Check `DATABASE_URL` env var → Blocked if missing
- Create single-threaded tokio runtime
- block_on(s_infra_1_api())

**Async function** (`s_infra_1_api`):
- Setup: scenario_pool → setup_db → create_tenant("S.INFRA.1 Round-trip") → router
- Step 1: POST /projects with `{"client_name": "Round-Trip Test"}` → expect 201, capture id
- Step 2: GET /projects/:id → expect 200, verify `body["client_name"] == "Round-Trip Test"`
- Step 3: POST /projects/:id/zones with 12×15 patio geometry → expect 201, capture zone_id
- Step 4: GET /projects/:id/zones → expect 200, verify array contains zone with matching id
- Step 5: POST /materials with Travertine Pavers ($8.50, SqFt) → expect 201, capture material_id
- Step 6: PUT /projects/:id/tiers/good with assignment (zone_id → material_id) → expect 200
- Step 7: GET /projects/:id/quote/good → expect 200, verify:
  - 1 line item
  - line_total == "1530.00" (12 × 15 × 8.50 = 1530.00, computed independently)
  - subtotal == "1530.00"
- Step 8: DELETE /projects/:id → expect 200
- Step 9: GET /projects/:id → expect 404
- Return Pass(TwoStar, OneStar)

**Verification**: `cargo build -p pt-scenarios` compiles.

## Step 2: Claim milestones in progress.rs

Update three milestones:

1. `pt-project` milestone: `delivered_by: Some("T-025-02")`, note describing
   Zone/Tier/Project types in pt-project, GeoJSON serde, used by API routes.

2. `pt-quote` milestone: `delivered_by: Some("T-025-02")`, note describing
   compute_quote() function, line item generation, verified by API round-trip.

3. `SvelteKit frontend + CF Worker proxy` milestone: `delivered_by: Some("T-025-02")`,
   note that frontend exists on CF Pages, Worker proxy deployed.

**Verification**: `cargo build -p pt-scenarios` compiles.

## Step 3: Run quality gate

Run `just check` (fmt + lint + test + scenarios).

Expected outcomes:
- fmt/lint/test: pass (no logic changes to production code)
- scenarios: S.INFRA.1 either passes (with DATABASE_URL) or shows Blocked
- Milestones: 19/24 delivered (was 16/24)
- No regressions in other scenarios

## Testing Strategy

- **No unit tests**: This IS a scenario test. It tests the full stack.
- **Database required**: Test only runs with DATABASE_URL set. Without it, returns Blocked.
- **Independent verification**: Quote total $1,530.00 = 12 × 15 × $8.50, computed in the test.
- **Regression check**: All existing scenarios must maintain their current status.
