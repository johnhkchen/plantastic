# Review — T-002-02: pt-quote Engine

## Summary

Implemented the pt-quote crate — a pure-computation quote engine that takes zone geometries, material assignments, and a material catalog, and produces a `Quote` with line items, subtotal, and total. Turned scenario tests S.3.1 and S.3.2 from stubs to passing.

## Files Created

| File | Purpose |
|------|---------|
| `crates/pt-quote/src/types.rs` | `Quote` and `LineItem` output structs |
| `crates/pt-quote/src/error.rs` | `QuoteError` enum (`MaterialNotFound`, `MissingDepth`) |
| `crates/pt-quote/src/engine.rs` | `compute_quote()` function + 12 unit tests |

## Files Modified

| File | Change |
|------|--------|
| `crates/pt-quote/Cargo.toml` | Added pt-geo, pt-project, pt-materials deps |
| `crates/pt-quote/src/lib.rs` | Module declarations and re-exports |
| `tests/scenarios/Cargo.toml` | Added domain crate deps for real scenario tests |
| `tests/scenarios/src/suites/quoting.rs` | Rewrote S.3.1 and S.3.2 from stubs to passing tests |
| `crates/pt-materials/src/builder.rs` | Added `#[derive(Debug)]` to `MaterialBuilder` (pre-existing lint fix) |
| `crates/pt-project/src/project.rs` | Added `# Errors` doc sections (pre-existing lint fix) |
| `crates/pt-project/src/geojson_conv.rs` | Added `# Errors` doc section (pre-existing lint fix) |
| `crates/pt-project/src/serde_helpers.rs` | Added `# Errors` doc sections (pre-existing lint fix) |

## Test Coverage

### Unit tests: 12 in pt-quote
All verify hand-computed expected values, not system-derived values.

| Test | Capability |
|------|-----------|
| sqft_10x10_patio_at_5_per_sqft | Area-based pricing (acceptance criteria example) |
| cuyd_material_with_depth | Volume-based pricing with depth conversion |
| linearft_edging | Perimeter-based pricing |
| each_material | Per-unit pricing |
| zone_with_no_assignment_skipped | Unassigned zones produce no line items |
| multiple_materials_per_zone | Multiple materials on one zone |
| override_price | Price override takes precedence over catalog |
| override_depth | Depth override takes precedence over catalog |
| missing_material_returns_error | Error on unknown MaterialId |
| missing_depth_for_cuyd_returns_error | Error on cu_yd material without depth |
| empty_tier_produces_empty_quote | Empty tier edge case |
| tax_included_in_total | Tax flows through to total |

### Scenario tests: 2 passing
- **S.3.1** (25 min) — Quantity computation from geometry. Three zone types (sq_ft, cu_yd, linear_ft) with independently computed expected values.
- **S.3.2** (15 min) — Three-tier quote generation. Good < Better < Best ordering, subtotal arithmetic identity, zone label matching, no duplicate assignments.

### Workspace total: 72 tests passing
pt-geo: 21, pt-materials: 11, pt-project: 28, pt-quote: 12.

## Value Dashboard — Before and After

**Before:** 0.0 min / 240.0 min (0.0%). All 17 scenarios NotImplemented.

**After:** 40.0 min / 240.0 min (16.7%). S.3.1 and S.3.2 PASS. Quoting area at 40/60 min (67%).

## Quality Gate Status

- `just fmt-check` — PASS
- `just lint` — PASS
- `cargo test --workspace` — PASS (72 tests)
- `cargo run -p pt-scenarios` — PASS (2 pass, 0 fail, 15 not implemented)

Note: `just test` fails because `timeout` binary is not installed on this Mac. This is a pre-existing infra issue unrelated to this ticket. All tests pass via `cargo test --workspace`.

## Open Concerns

1. **`just test` timeout binary.** The justfile uses `timeout` (GNU coreutils) which isn't in the default macOS PATH. Recommend `brew install coreutils` or switching to a Rust-based timeout. Not blocking — tests work directly.

2. **f64 → Decimal conversion.** Quantities computed from geometry are f64 (from the `geo` crate). Conversion to Decimal uses `Decimal::from_f64()` which can introduce float representation artifacts. For the geometries tested (small integer coordinates), this is exact. For complex real-world polygons with many decimal places, the 4-decimal rounding on display quantity handles it. Line totals are always rounded to 2dp.

3. **No "each" zone area check.** An "each" material always produces quantity 1 regardless of zone size. This is correct per the spec, but there's no validation that "each" materials should only be used on small zones. Not a pt-quote concern — business logic validation belongs in the API layer.

## Acceptance Criteria Verification

- [x] Quote struct: tier name, line_items vec, subtotal, optional tax, total
- [x] LineItem struct: zone_label, material_name, quantity, unit, unit_price, line_total
- [x] sq_ft → area, cu_yd → area × depth / 12 / 27, linear_ft → perimeter, each → 1
- [x] Zones with no assignment skipped
- [x] Multiple materials per zone handled
- [x] Subtotal sums all line totals; total = subtotal + tax
- [x] Test: 10×10 patio at $5/sq_ft = $500
- [x] Test: three tiers produce different totals for same zones
