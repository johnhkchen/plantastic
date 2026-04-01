# Research — T-008-02: Quote Scenario Two-Star

## Objective

Upgrade S.3.1 and S.3.2 scenario tests from OneStar (pure computation) to TwoStar
(API-level integration). The scenarios must exercise the full HTTP path: create project,
add zones, create materials, set tier assignments, fetch quote via GET, and verify.

---

## Current State

### S.3.1 — Quantity computation from geometry (OneStar)

- **File:** `tests/scenarios/src/suites/quoting.rs:47-219`
- **What it does:** Constructs in-memory Zone/Material/Tier structs, calls
  `pt_quote::compute_quote()` directly, verifies line item totals against
  hand-computed arithmetic.
- **Zones:** 12x15 patio (180 sq_ft), 8x20 bed (160 sq_ft, 4" depth), 10x10 edging (40 lin_ft)
- **Materials:** Travertine Pavers $8.50/sq_ft, Premium Mulch $45/cu_yd, Steel Edge $3.25/lin_ft
- **Expected:** Patio $1,530.00 + Mulch $88.89 + Edging $130.00 = $1,748.89
- **Returns:** `ScenarioOutcome::Pass(Integration::OneStar)`

### S.3.2 — Three-tier quote generation (OneStar)

- **File:** `tests/scenarios/src/suites/quoting.rs:226-504`
- **What it does:** Same geometry, 3 tiers with 9 materials at escalating prices.
  Calls `compute_quote()` for each tier, verifies Good < Better < Best, subtotal
  integrity, zone label matching, no duplicate assignments, and exact Good/Best totals.
- **Expected totals:** Good $839.26, Best $3,148.40
- **Returns:** `ScenarioOutcome::Pass(Integration::OneStar)`

### Quote API Route (T-008-01, delivered)

- **File:** `crates/plantastic-api/src/routes/quotes.rs`
- **Route:** `GET /projects/{id}/quote/{tier}` where tier is good|better|best
- **Flow:** verify tenant -> parse tier -> load zones/assignments/materials from DB ->
  convert repo types to domain types -> call `pt_quote::compute_quote()` -> return JSON
- **Response shape:** `{ tier, line_items: [{zone_id, zone_label, material_id, material_name, quantity, unit, unit_price, line_total}], subtotal, tax, total }`
- **Decimal serialization:** `rust_decimal` with `serde-with-str` feature serializes
  Decimal values as strings in JSON (e.g., `"1530.00"`).

### API Test Infrastructure

- **File:** `crates/plantastic-api/tests/common/mod.rs`
- **Helpers:**
  - `test_pool()` — PgPool from DATABASE_URL env var
  - `setup_test_db(pool)` — runs all migrations from `migrations/`
  - `create_test_tenant(pool, name)` — inserts tenant, returns UUID
  - `test_router(pool)` — builds Axum Router with AppState
  - `send(app, method, uri, tenant_id, body)` — sends request, returns (StatusCode, Value)
- **Pattern:** These are in `tests/common/` (test module), not a library. Cannot be
  imported by the scenarios crate directly.

### Existing Integration Test (T-008-01)

- **File:** `crates/plantastic-api/tests/crud_test.rs:478-605`
- **Test:** `quote_route_integration` — creates project, zone (12x15 patio), material
  (Travertine $8.50/sq_ft), sets tier assignment, GETs quote, verifies line_total = "1530.00".
- **Demonstrates:** The full API path works with real Postgres. Exact pattern needed for scenarios.

### Scenario Harness Architecture

- **Entry:** `tests/scenarios/src/main.rs` — runs all scenarios, catches panics, prints dashboard.
- **Signature:** `test_fn: fn() -> ScenarioOutcome` — plain sync function, not async.
- **Implication:** TwoStar scenarios that hit the API need to create a tokio runtime
  inside the function to run async code, or the harness needs to change.
- **Current deps:** geo, rust_decimal, pt-geo, pt-project, pt-materials, pt-quote, pt-satellite,
  pt-solar, chrono. No async runtime, no axum, no sqlx, no tower.

### Milestone Status

Already claimed (no action needed for T-008-02):
- "Quote API route" — delivered by T-008-01, unlocks S.3.1/S.3.2
- "Axum API" — delivered by T-004-02
- "PostGIS schema" — delivered by T-003-02
- "pt-geo" — delivered by T-001-02

### API Endpoints Needed for Setup

1. `POST /projects` — `{address?, client_name?, client_email?}` -> `{id, ...}`
2. `POST /projects/{id}/zones` — `{geometry: GeoJSON, zone_type, label?, sort_order?}` -> `{id, ...}`
3. `POST /materials` — `{name, category, unit, price_per_unit, depth_inches?, extrusion, ...}` -> `{id, ...}`
4. `PUT /projects/{id}/tiers/{tier}` — `{assignments: [{zone_id, material_id, overrides?}]}` -> 204
5. `GET /projects/{id}/quote/{tier}` -> `{tier, line_items, subtotal, tax, total}`

All require `X-Tenant-Id` header (UUID).

---

## Constraints and Risks

1. **Sync scenario signature:** The `fn() -> ScenarioOutcome` signature forces us to
   create a tokio runtime inside each scenario function. This is safe but adds ~1ms overhead.

2. **DATABASE_URL dependency:** TwoStar scenarios require a live Postgres instance.
   If DATABASE_URL is not set, scenarios should return `Blocked` rather than panic.
   This keeps `just scenarios` runnable in environments without Postgres.

3. **Test isolation:** Each scenario should create its own tenant to avoid interference.
   The existing CRUD tests follow this pattern.

4. **Migration idempotency:** `setup_test_db()` runs all migrations. If tables already
   exist, this may fail. The existing tests handle this via CREATE TABLE IF NOT EXISTS
   (checking migration SQL would confirm). Alternatively, scenarios can share a pool
   and only run setup once.

5. **Decimal comparison in JSON:** API returns decimals as strings ("1530.00"). Scenario
   assertions need to compare strings, not floats.

6. **Old test preservation:** Acceptance criteria requires old OneStar tests remain as
   unit-level regression. The pt-quote engine tests (engine.rs:124-452, 13 tests) already
   cover the arithmetic. The scenario-specific combinations should be preserved as non-scenario
   functions or regular tests.

7. **GeoJSON format:** Zones are created via API with GeoJSON geometry. The polygons must
   be closed rings: `[[0,0],[12,0],[12,15],[0,15],[0,0]]`. The old tests use `geo::polygon!`
   macro which doesn't require closing, but GeoJSON does.

8. **Extrusion serialization:** Materials require an `extrusion` field as JSON. The API
   expects the tagged enum format: `{"type": "sits_on_top", "height_inches": 1.0}`.
