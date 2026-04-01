# Structure — T-002-02: pt-quote Engine

## Files Modified

### `crates/pt-quote/Cargo.toml`
Add dependencies: `pt-geo`, `pt-project`, `pt-materials` (path deps), `geo` (workspace).

### `crates/pt-quote/src/lib.rs`
Module root. Re-exports public API: `compute_quote`, `Quote`, `LineItem`, `QuoteError`.

Declares modules:
```
pub mod types;
pub mod engine;
pub mod error;
```

### `tests/scenarios/Cargo.toml`
Add dependencies: `pt-geo`, `pt-project`, `pt-materials`, `pt-quote`, `geo`, `rust_decimal`.

## Files Created

### `crates/pt-quote/src/types.rs`
Output types — no logic, just data:
- `Quote { tier: TierLevel, line_items: Vec<LineItem>, subtotal: Decimal, tax: Option<Decimal>, total: Decimal }`
- `LineItem { zone_id: ZoneId, zone_label: Option<String>, material_id: MaterialId, material_name: String, quantity: Decimal, unit: Unit, unit_price: Decimal, line_total: Decimal }`

Derives: `Debug, Clone, PartialEq, Serialize, Deserialize`.

### `crates/pt-quote/src/error.rs`
Error enum:
- `QuoteError::MaterialNotFound { material_id: MaterialId, zone_id: ZoneId }`
- `QuoteError::MissingDepth { material_id: MaterialId, material_name: String }`

Implements `Display`, `Error`.

### `crates/pt-quote/src/engine.rs`
The computation module. Contains:
- `pub fn compute_quote(zones: &[Zone], tier: &Tier, materials: &[Material], tax: Option<Decimal>) -> Result<Quote, QuoteError>`
- `fn find_material<'a>(id: MaterialId, materials: &'a [Material]) -> Option<&'a Material>` (private helper)
- `fn compute_quantity(zone: &Zone, material: &Material, overrides: Option<&AssignmentOverrides>) -> Result<Decimal, QuoteError>` (private helper)
- `fn round_currency(d: Decimal) -> Decimal` (private, rounds to 2 dp)

Inline `#[cfg(test)]` module with unit tests.

## Module Boundaries

```
pt-quote
├── lib.rs          (re-exports)
├── types.rs        (Quote, LineItem — data only)
├── error.rs        (QuoteError — Display + Error)
└── engine.rs       (compute_quote — all logic + tests)
```

Public interface is three items: `compute_quote`, `Quote`, `LineItem`, `QuoteError`. Everything else is internal.

## Dependency Direction

```
pt-geo ←── pt-quote
pt-project ←── pt-quote (reads Zone, Tier, MaterialAssignment, etc.)
pt-materials ←── pt-quote (reads Material, Unit, MaterialId)
```

pt-quote depends on all three. No reverse dependency. This matches the spec's crate dependency graph.

## What Is NOT Changed

- pt-geo, pt-project, pt-materials — no modifications needed. They provide everything pt-quote needs.
- apps/api — not touched in this ticket. API routes that call pt-quote come later.
- Scenario test functions in `tests/scenarios/src/suites/quoting.rs` — rewritten from stubs to real tests.

## Ordering

1. Types and error first (no logic deps).
2. Engine depends on types + error.
3. Scenario tests depend on all of the above compiling.
