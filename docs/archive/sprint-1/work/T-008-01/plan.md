# Plan — T-008-01: Quote API Route

## Step 1: Add pt-quote dependency

**Files:** `crates/plantastic-api/Cargo.toml`
**Action:** Add `pt-quote = { path = "../pt-quote" }` to `[dependencies]`
**Verify:** `cargo check -p plantastic-api` succeeds

## Step 2: Make parse_tier pub(crate)

**Files:** `crates/plantastic-api/src/routes/tiers.rs`
**Action:** Change `fn parse_tier` to `pub(crate) fn parse_tier`
**Verify:** `cargo check -p plantastic-api` succeeds (no behavior change)

## Step 3: Implement the quote route handler

**Files:** `crates/plantastic-api/src/routes/quotes.rs` (new)
**Action:**
- Create route function: `GET /projects/{id}/quote/{tier}`
- Implement `get_quote` handler:
  1. Extract TenantId, project_id, tier string
  2. verify_project_tenant (reuse from zones.rs)
  3. parse_tier (reuse from tiers.rs, now pub(crate))
  4. Load project row (need tenant_id for material query)
  5. Load zones via zone::list_by_project
  6. Load tier assignments via tier_assignment::get_by_project_and_tier
  7. Load materials via material::list_by_tenant (using project's tenant_id)
  8. Convert repo types to domain types (3 conversion functions)
  9. Call compute_quote(zones, tier, materials, None)
  10. Map QuoteError to AppError::BadRequest
  11. Return Json<Quote>
- Implement private conversion functions:
  - `zone_rows_to_zones(Vec<ZoneRow>) -> Vec<Zone>`
  - `material_rows_to_materials(Vec<MaterialRow>) -> Vec<Material>`
  - `build_tier(TierLevel, Vec<TierAssignmentRow>) -> Result<Tier, AppError>`
**Verify:** `cargo check -p plantastic-api` succeeds

## Step 4: Register the route

**Files:**
- `crates/plantastic-api/src/routes/mod.rs`
**Action:**
- Add `pub mod quotes;`
- Add `.merge(quotes::routes())` to the router
**Verify:** `cargo check -p plantastic-api` succeeds

## Step 5: Run existing tests + lint

**Action:** `just fmt && just lint && just test`
**Verify:** All pass, no regressions. Existing scenario dashboard unchanged.

## Step 6: Add integration test

**Files:** `crates/plantastic-api/tests/crud_test.rs`
**Action:** Add `quote_route_integration` test:
- Setup: pool, migrations, tenant, router
- Create project via POST /projects
- Create zone: 12x15 patio via POST /projects/:id/zones
- Create material: Pavers at $8.50/sq_ft via POST /materials
- Set tier assignment: good tier, zone→material via PUT /projects/:id/tiers/good
- GET /projects/:id/quote/good → assert 200
  - Assert 1 line item
  - Assert line_total == "1530.00" (12 * 15 = 180 sq_ft * $8.50)
  - Assert subtotal == total == "1530.00"
  - Assert tier == "good"
- GET /projects/:id/quote/better → assert 200, empty line_items, $0 total
- GET /projects/:id/quote/invalid → assert 400
- GET /projects/{random-uuid}/quote/good → assert 404
**Verify:** Test compiles. (Actual execution requires Postgres; test is #[ignore])

## Step 7: Claim milestone in progress.rs

**Files:** `tests/scenarios/src/progress.rs`
**Action:** Add a new milestone entry for the quote API route:
- label: "Quote API route: GET /projects/:id/quote/:tier"
- delivered_by: Some("T-008-01")
- unlocks: &["S.3.1", "S.3.2"]
- note: describes what was delivered
**Verify:** `just scenarios` runs without regression

## Step 8: Final quality gate

**Action:** `just check` (fmt-check + lint + test + scenarios)
**Verify:** All four checks pass. No scenario regressions.

## Testing strategy

| Test type | What | Where |
|-----------|------|-------|
| Integration | Full HTTP round-trip: create data → fetch quote → verify totals | crud_test.rs |
| Integration | Empty tier returns $0 quote | crud_test.rs |
| Integration | 404 for missing project | crud_test.rs |
| Integration | 400 for invalid tier name | crud_test.rs |
| Unit (existing) | compute_quote arithmetic correctness | pt-quote engine tests |
| Scenario (existing) | S.3.1, S.3.2 at OneStar | scenarios/quoting.rs |

No new unit tests needed — the handler is glue code (load, convert, call engine, return).
The integration test verifies the glue works end-to-end with real Postgres. The unit tests
in pt-quote already verify all arithmetic edge cases.
