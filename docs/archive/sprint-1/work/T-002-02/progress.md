# Progress — T-002-02: pt-quote Engine

## Completed

### Step 1: Cargo.toml dependencies
Added `pt-geo`, `pt-project`, `pt-materials`, `geo` to pt-quote deps. Added `serde_json`, `pt-test-utils` as dev-deps.

### Step 2: error.rs
Created `QuoteError` enum with `MaterialNotFound` and `MissingDepth` variants.

### Step 3: types.rs
Created `Quote` and `LineItem` structs with all fields per design.

### Step 4: engine.rs — compute_quote
Implemented the core function with helpers: `find_material`, `compute_quantity`, `effective_price`, `round_currency`, `round_quantity`.

### Step 5: Unit tests (12 passing)
All 12 planned tests implemented and passing:
- sqft_10x10_patio_at_5_per_sqft
- cuyd_material_with_depth
- linearft_edging
- each_material
- zone_with_no_assignment_skipped
- multiple_materials_per_zone
- override_price
- override_depth
- missing_material_returns_error
- missing_depth_for_cuyd_returns_error
- empty_tier_produces_empty_quote
- tax_included_in_total

### Step 6: lib.rs re-exports
Wired up modules. Public API: `compute_quote`, `Quote`, `LineItem`, `QuoteError`.

### Step 7: Scenario tests S.3.1 and S.3.2
Updated `tests/scenarios/Cargo.toml` with domain crate dependencies. Rewrote S.3.1 and S.3.2 from stubs to real tests. Both pass.

### Step 8: Quality gate
- `just fmt-check` — pass
- `just lint` — pass
- `cargo test --workspace` — 72 tests pass
- `cargo run -p pt-scenarios` — S.3.1 and S.3.2 PASS, 40 min verified savings

## Deviations from Plan

- **Pre-existing lint fixes:** Fixed `missing_debug_implementations` on `MaterialBuilder` (pt-materials) and 6 `missing_errors_doc` warnings on pt-project public functions. Per CLAUDE.md rule 6 ("own what you find").
- **`just test` timeout binary:** The `timeout` command isn't installed on this Mac, so `just test` fails at the shell level. This is a pre-existing infrastructure issue — all tests pass when run via `cargo test --workspace` directly. Not caused by this ticket.

## Remaining
Nothing. All planned work complete.
