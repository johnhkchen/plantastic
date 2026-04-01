# T-024-01 Research: Material Callout Scenario

## Objective

Implement S.4.3 (Material callouts with supplier info) in `tests/scenarios/src/suites/crew_handoff.rs`. This is currently `NotImplemented` with all prerequisites met.

## What exists

### Scenario infrastructure

- **Registry** (`tests/scenarios/src/registry.rs`): `Scenario`, `ScenarioOutcome`, `ValueArea`, `Integration`, `Polish` types. `ScenarioOutcome::Pass(Integration, Polish)` for passing tests. `effective_minutes = raw × (int + pol) / 10`.
- **Crew handoff suite** (`tests/scenarios/src/suites/crew_handoff.rs`): Three scenarios defined (S.4.1, S.4.2, S.4.3). All return `NotImplemented`. S.4.3 has `time_savings_minutes: 5.0` under `ValueArea::CrewHandoff` (30 min budget).
- **Pattern**: Scenario test functions return `ScenarioOutcome`. They construct domain objects, assert against independently computed expected values, and return `Pass` with star ratings on success or `Fail(String)` on assertion failure.

### pt-materials crate (`crates/pt-materials/src/`)

- **Material struct** has all fields needed for callouts:
  - `name: String`
  - `supplier_sku: Option<String>`
  - `depth_inches: Option<f64>`
  - `photo_ref: Option<String>`
  - `extrusion: ExtrusionBehavior` (SitsOnTop/Fills/BuildsUp)
- **MaterialBuilder**: `Material::builder(name, category)` with fluent `.supplier_sku()`, `.depth_inches()`, `.photo_ref()`, `.extrusion()`, `.build()`.
- **Serialization**: snake_case enums, internally-tagged extrusion, Decimal as string.

### pt-project crate (`crates/pt-project/src/types.rs`)

- **Zone**: `id: ZoneId`, `geometry: Polygon<f64>`, `zone_type: ZoneType`, `label: Option<String>`.
- **MaterialAssignment**: `zone_id: ZoneId`, `material_id: MaterialId`, `overrides: Option<AssignmentOverrides>`.
- **Tier**: `level: TierLevel`, `assignments: Vec<MaterialAssignment>`.
- **TierLevel**: Good/Better/Best.

### pt-quote crate (`crates/pt-quote/src/`)

- `compute_quote(zones, tier, materials, tax) -> Result<Quote, QuoteError>`
- `Quote` has `line_items: Vec<LineItem>` where each `LineItem` has `zone_id`, `material_id`, `material_name`, `quantity`, `unit`, `unit_price`, `line_total`.
- LineItem contains `material_name` but NOT `supplier_sku`, `depth_inches`, `photo_ref`, or `extrusion`.

### Existing scenario patterns

- **S.2.2** (design.rs): Builds materials with `Material::builder()`, tests serialization and filtering. OneStar pure-computation test.
- **S.3.1** (quoting.rs): Two paths — `s_3_1_computation()` for OneStar (no DB), `s_3_1_api()` for TwoStar (with DB). Falls back based on `DATABASE_URL`.
- **S.2.1** (design.rs): Constructs polygons with `geo::polygon!`, computes area/perimeter, asserts against hand-calculated values.

## Key observation

The "callout" is the material's metadata fields exposed per zone-material pair. The acceptance criteria say:
1. Material name present
2. Supplier SKU present (e.g., "TRAV-12x12-NAT")
3. Install depth present and matches material's `depth_inches`
4. Photo ref present when material has one
5. Extrusion behavior present

Currently, `LineItem` in pt-quote carries only `material_name` and `material_id`. For a callout, you need the full material metadata. The test can verify this by:
- Building materials with known callout fields
- Building zones with known geometry
- Building tier assignments mapping zones → materials
- For each assignment, looking up the material by ID and verifying all callout fields are present and match expected values

This is a data-model/computation test at OneStar — it proves the domain types carry the data a crew foreman needs. No new types or API routes needed.

## Constraints

- S.4.3 at OneStar + OneStar: `5.0 × 2/10 = 1.0` effective minute (Crew Handoff goes from 0.0 to 1.0).
- Expected values must be independently computed in the test, not extracted from the system.
- No mocking across crate boundaries — use real pt-materials and pt-project types.
- `just check` must pass after implementation.

## Dependencies

- T-012-01 (pt-materials): delivered ✓
- T-013-02 (Bevy viewer): delivered ✓ (milestone, not directly used at OneStar)
- T-022-01 (depends_on in ticket): must be complete

## Files to modify

- `tests/scenarios/src/suites/crew_handoff.rs` — replace `s_4_3_material_callouts` stub with real test.
