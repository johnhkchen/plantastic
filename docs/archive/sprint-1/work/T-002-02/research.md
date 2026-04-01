# Research — T-002-02: pt-quote Engine

## What Exists

### pt-geo (pure computation, no I/O)

The geometry crate provides the measurement functions pt-quote needs:

- **`area::area_sqft(&Polygon<f64>) -> f64`** — unsigned area via `geo::Area`. Used for sq_ft materials.
- **`perimeter::perimeter_ft(&Polygon<f64>) -> f64`** — exterior ring length via `geo::Length<Euclidean>`. Used for linear_ft materials.
- **`volume::volume_cuyd(area_sqft: f64, depth_ft: f64) -> f64`** — area * depth / 27. Used for cu_yd materials.

Also has `multi_*` variants for `MultiPolygon`, boolean ops, and simplification. The volume module takes pre-computed area and depth in feet (not inches), so pt-quote must convert `depth_inches / 12.0` before calling.

Re-exports `geo::{Polygon, Coord, LineString, MultiPolygon, polygon, coord}` so callers don't need a direct `geo` dependency.

### pt-project (domain model)

Core types that pt-quote reads:

- **`Project`** — aggregate root. Has `zones: Vec<Zone>`, `tiers: Vec<Tier>` (always 3), `status: ProjectStatus`.
- **`Zone`** — `{ id: ZoneId, geometry: Polygon<f64>, zone_type: ZoneType, label: Option<String> }`.
- **`Tier`** — `{ level: TierLevel, assignments: Vec<MaterialAssignment> }`.
- **`MaterialAssignment`** — `{ zone_id: ZoneId, material_id: MaterialId, overrides: Option<AssignmentOverrides> }`.
- **`AssignmentOverrides`** — `{ price_override: Option<Decimal>, depth_override_inches: Option<f64> }`.
- **`TierLevel`** — `Good | Better | Best`.
- **`ZoneId`** — newtype over `Uuid`.

A project always has 3 tiers. Assignments link a zone to a material within a tier. Multiple assignments per zone per tier are allowed (e.g., gravel base + paver surface). `overrides` lets a specific assignment use a different price or depth than the catalog material.

### pt-materials (catalog domain)

Types pt-quote needs to look up material info:

- **`Material`** — `{ id: MaterialId, name, category, unit: Unit, price_per_unit: Decimal, depth_inches: Option<f64>, extrusion, ... }`.
- **`Unit`** — `SqFt | CuYd | LinearFt | Each`. Determines how quantity is computed from geometry.
- **`MaterialId`** — newtype over `Uuid`.
- **`MaterialBuilder`** — fluent builder for test fixtures.

`price_per_unit` is `rust_decimal::Decimal` for exact currency math. `depth_inches` is `Option<f64>` (only relevant for `CuYd` materials).

### pt-quote (current state: empty stub)

`Cargo.toml` has dependencies on `serde`, `uuid`, `rust_decimal`. The `lib.rs` is a single doc comment. No types, no logic.

Missing from Cargo.toml: `pt-geo`, `pt-project`, `pt-materials` dependencies.

### Scenario tests (quoting suite)

Two scenarios directly gate this ticket:

- **S.3.1** — Quantity computation from geometry. Tests a project with 3 zones (12x15 patio, 8x20 bed, edging path), 3 materials (sq_ft, cu_yd, linear_ft), and verifies exact line item totals: $1,530.00 + $88.89 + $130.00 = $1,748.89.
- **S.3.2** — Three-tier quote generation. Same geometry, 3 tiers with different materials. Asserts Good < Better < Best, subtotal == sum(line_totals), no duplicate zone assignments.

Both currently return `ScenarioOutcome::NotImplemented`. The scenario binary (`pt-scenarios`) doesn't depend on any domain crates yet.

### Monetary precision

The project uses `rust_decimal::Decimal` for money. Materials store `price_per_unit` as `Decimal`. Overrides store `price_override` as `Option<Decimal>`. All quote outputs (line_total, subtotal, total) must use `Decimal`.

Volume computation flows through `f64` (pt-geo), so pt-quote must convert the `f64` quantity to `Decimal` before multiplying by price. The conversion point matters for precision.

## Boundaries and Constraints

1. **Pure computation.** No I/O, no database, no async. Takes types in, returns types out.
2. **No mocking.** Tests use real pt-geo, pt-project, pt-materials.
3. **Exact arithmetic for money.** All prices and totals use `Decimal`. Quantities may be `f64` (from geometry) but must convert to `Decimal` before price multiplication.
4. **Quantity rounding.** The ticket specifies cu_yd = `area × depth_inches / 12 / 27`. The scenario expects `160 × (4/12) / 27 = 1.9753... cu_yd × $45 = $88.89`. This implies rounding the final line total to 2 decimal places, not rounding the intermediate quantity.
5. **Material lookup.** pt-quote needs access to the Material catalog to resolve MaterialId → Material. The function signature must accept materials as input (slice, HashMap, or closure).
6. **Zone skip rule.** Zones with no material assignment for the current tier produce no line items.
7. **Multiple materials per zone.** A zone can have multiple assignments (e.g., gravel + pavers). Each produces a separate line item.
8. **Override handling.** `AssignmentOverrides` can override `price_per_unit` and `depth_inches` per assignment.

## Relevant Patterns

- Crates use workspace edition/license/lints.
- Types derive `Debug, Clone, PartialEq, Serialize, Deserialize`.
- IDs are newtypes over `Uuid` with `Display` impls.
- Errors are enums implementing `Display + Error`.
- Tests use `#[cfg(test)]` inline modules.
- `pt_test_utils::timed()` wraps test bodies for timeout enforcement.

## What pt-quote Must NOT Do

- Call the database or any external service.
- Import `pt-project`'s project.rs methods beyond reading struct fields.
- Round intermediate quantities (only round the final line total).
- Generate `$0.00` line items for unassigned zones.
- Handle tax computation internally (tax is provided externally or left as None/zero).
