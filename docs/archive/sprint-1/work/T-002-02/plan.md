# Plan — T-002-02: pt-quote Engine

## Step 1: Add dependencies to pt-quote Cargo.toml

Add `pt-geo`, `pt-project`, `pt-materials` as path deps. Add `geo` workspace dep. Add `serde_json` as dev-dependency for test serialization. Add `pt-test-utils` as dev-dependency.

Verify: `cargo check -p pt-quote` compiles.

## Step 2: Create error.rs

Define `QuoteError` enum with `MaterialNotFound` and `MissingDepth` variants. Implement `Display` and `Error`.

Verify: `cargo check -p pt-quote` compiles.

## Step 3: Create types.rs

Define `Quote` and `LineItem` structs with all fields per design. Derive standard traits.

Verify: `cargo check -p pt-quote` compiles.

## Step 4: Implement engine.rs with compute_quote

Write the core function:
1. For each assignment in the tier, look up the zone and material.
2. Skip assignments where zone_id doesn't match any zone (defensive).
3. Compute quantity based on `material.unit`.
4. Apply overrides for price and depth.
5. Compute `line_total = quantity * unit_price`, round to 2 dp.
6. Collect line items, compute subtotal as sum, total as subtotal + tax.

Private helpers: `find_material`, `compute_quantity`, `round_currency`.

Verify: `cargo check -p pt-quote` compiles.

## Step 5: Write unit tests in engine.rs

Tests (all wrapped in `pt_test_utils::timed()`):

1. **10x10 sq_ft patio at $5/sq_ft = $500.00** — the acceptance criteria example.
2. **cu_yd material with depth** — area 100 sq_ft, depth 4 inches → `100 * (4/12) / 27 = 1.2345... cu_yd * $40 = $49.38`.
3. **linear_ft edging** — 10x10 square perimeter = 40 ft × $2.50 = $100.00.
4. **each material** — quantity 1, price $150 = $150.00.
5. **zone with no assignment skipped** — two zones, only one assigned, quote has 1 line item.
6. **multiple materials per zone** — gravel base + paver surface = 2 line items for same zone.
7. **override price** — material at $5/sq_ft, override to $7/sq_ft, line total uses $7.
8. **override depth** — cu_yd material, catalog depth 3", override to 6", quantity doubles.
9. **missing material returns error** — assignment references nonexistent MaterialId.
10. **missing depth for cu_yd returns error** — cu_yd material with no depth, no override.
11. **empty tier produces empty quote** — no assignments, no line items, subtotal = $0.
12. **tax included in total** — subtotal $500, tax $42.50, total $542.50.

All expected values computed as hand arithmetic, not via pt-geo calls.

Verify: `cargo test -p pt-quote` passes.

## Step 6: Update lib.rs re-exports

Wire up modules and re-export `compute_quote`, `Quote`, `LineItem`, `QuoteError`.

Verify: `cargo test -p pt-quote` still passes.

## Step 7: Implement S.3.1 and S.3.2 scenario tests

Update `tests/scenarios/Cargo.toml` to add domain crate dependencies.

Rewrite `s_3_1_quantity_from_geometry`:
- Build 3 zones: 12x15 patio (180 sq_ft), 8x20 bed (160 sq_ft, 4" depth), path with 40 ft perimeter.
- Assign materials at known prices.
- Call `compute_quote`, assert exact line totals and subtotal.
- Return `ScenarioOutcome::Pass` or `ScenarioOutcome::Fail`.

Rewrite `s_3_2_three_tier_quotes`:
- Same geometry, 3 tiers with increasing prices.
- Assert Good < Better < Best totals.
- Assert subtotal == sum(line_totals) for each tier.
- Assert no duplicate zone assignments.
- Return `ScenarioOutcome::Pass` or `ScenarioOutcome::Fail`.

Verify: `cargo run -p pt-scenarios` shows S.3.1 and S.3.2 as PASS.

## Step 8: Run full quality gate

`just check` — fmt, lint, test, scenarios all pass.

## Testing Strategy Summary

| Layer | What | Count |
|-------|------|-------|
| Unit tests | engine.rs inline #[cfg(test)] | ~12 tests |
| Scenario tests | S.3.1, S.3.2 in quoting.rs | 2 scenarios |
| Integration tests | None needed — pt-quote is pure computation | 0 |

No mocking. All tests use real pt-geo, pt-project, pt-materials. Expected values are hand-computed arithmetic.
