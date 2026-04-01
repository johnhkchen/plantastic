# T-012-01 Plan — Catalog CRUD Page

## Steps

### Step 1: Fix extrusion casing bug in catalog page

**File:** `web/src/routes/(app)/catalog/+page.svelte`
**Change:** Line 68 — `'Fills'` → `'fills'`
**Verify:** Visual inspection (the fix is a single character change). The serde contract is defined in `crates/pt-materials/src/types.rs` line 59: `#[serde(tag = "type", rename_all = "snake_case")]`.

### Step 2: Add pt-materials dependency to scenario crate

**File:** `tests/scenarios/Cargo.toml`
**Change:** Add `pt-materials` to `[dependencies]` if not present. Also ensure `rust_decimal` and `serde_json` are available for the scenario test.
**Verify:** `cargo check -p pt-scenarios`

### Step 3: Implement S.2.2 scenario test

**File:** `tests/scenarios/src/suites/design.rs`
**Change:** Replace `s_2_2_material_catalog()` body with:

1. Construct 5 materials spanning all 4 categories:
   - Travertine Pavers: Hardscape, SqFt, $8.50, SitsOnTop
   - Premium Mulch: Softscape, CuYd, $45.00, Fills
   - Steel Edging: Edging, LinearFt, $3.25, BuildsUp
   - Pea Gravel: Fill, CuYd, $38.00, Fills
   - Flagstone: Hardscape, SqFt, $12.00, SitsOnTop

2. Verify JSON serialization for API contract:
   - Each material serializes via serde_json::to_value()
   - category → "hardscape", "softscape", "edging", "fill"
   - unit → "sq_ft", "cu_yd", "linear_ft"
   - extrusion → has "type" key with snake_case value
   - price_per_unit → string (Decimal serializes as string)

3. Verify in-memory filtering by category:
   - hardscape count = 2
   - softscape count = 1
   - edging count = 1
   - fill count = 1

4. Return `ScenarioOutcome::Pass(Integration::OneStar)`

**Verify:** `cargo run -p pt-scenarios` — S.2.2 should show PASS ★☆☆☆☆

### Step 4: Claim pt-materials milestone

**File:** `tests/scenarios/src/progress.rs`
**Change:** Set `delivered_by: Some("T-012-01")` and write descriptive note.
**Verify:** `cargo run -p pt-scenarios` — milestone shows as delivered

### Step 5: Run quality gate

**Command:** `just check`
**Verify:** All four checks pass (fmt, lint, test, scenarios). Scenario dashboard shows S.2.2 passing and effective minutes increased.

## Testing strategy

- **Unit tests:** No new unit tests needed. pt-materials already has comprehensive serde tests. The frontend fix is too small for a unit test.
- **Scenario test:** S.2.2 moves from NotImplemented to Pass(OneStar). This is the primary deliverable.
- **Integration tests:** No new integration tests. The material CRUD API is already tested in `crates/plantastic-api/tests/crud_test.rs`.

## Expected scenario impact

Before: S.2.2 = NotImplemented (0 effective minutes)
After: S.2.2 = Pass(OneStar) → 10.0 × 0.2 = 2.0 effective minutes
Net gain: +2.0 effective minutes

Dashboard should go from 41.0 min → 43.0 min.
