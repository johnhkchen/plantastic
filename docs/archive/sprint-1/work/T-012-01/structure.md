# T-012-01 Structure — Catalog CRUD Page

## Modified files

### 1. `web/src/routes/(app)/catalog/+page.svelte`

**Change:** Single line fix in `buildMaterialBody()` function.

```
Line 68 (current):  extrusion: editingMaterial?.extrusion ?? { type: 'Fills', flush: true },
Line 68 (fixed):    extrusion: editingMaterial?.extrusion ?? { type: 'fills', flush: true },
```

No structural changes. No new components, no new imports, no new state.

### 2. `tests/scenarios/src/suites/design.rs`

**Change:** Replace the `s_2_2_material_catalog()` stub with a real test body.

Current function body (lines 139-144): returns `ScenarioOutcome::NotImplemented`

New function body:
- Import: `use pt_materials::{Material, MaterialCategory, Unit, ExtrusionBehavior, MaterialId};`
- Import: `use rust_decimal::Decimal;` and `use std::str::FromStr;`
- Build 5 materials with `Material { ... }` (direct construction — the builder isn't needed since we're constructing with all fields known)
- Serialize each to JSON with `serde_json::to_value()`
- Assert category field is correct snake_case string
- Assert unit field is correct snake_case string
- Assert extrusion has correct `type` tag
- Assert price_per_unit serializes as a string (Decimal → JSON string)
- Filter materials by category, assert correct counts
- Return `ScenarioOutcome::Pass(Integration::OneStar)`

**Dependencies added to scenario crate:** The `pt-materials` crate is already a workspace member. Need to verify it's in the scenarios Cargo.toml dependencies.

### 3. `tests/scenarios/src/progress.rs`

**Change:** Update the pt-materials milestone (around line 102-106).

```rust
// Current:
Milestone {
    label: "pt-materials: catalog model + tenant layering",
    delivered_by: None,
    unlocks: &["S.2.2", "S.3.1", "S.3.2"],
    note: "",
}

// Updated:
Milestone {
    label: "pt-materials: catalog model + tenant layering",
    delivered_by: Some("T-012-01"),
    unlocks: &["S.2.2", "S.3.1", "S.3.2"],
    note: "pt-materials crate: Material struct with MaterialId, ...",
}
```

### 4. `tests/scenarios/Cargo.toml` (if needed)

**Change:** Add `pt-materials`, `rust_decimal`, `serde_json` to `[dependencies]` if not already present.

## No new files

No new files are created. All changes are modifications to existing files.

## No deleted files

No files are deleted.

## Module boundaries

No module boundaries change. The scenario test imports `pt_materials` types (already a public crate API). No new public APIs are added to any crate.

## Ordering

1. Fix `+page.svelte` (independent)
2. Add scenario crate dependency on pt-materials (if needed)
3. Implement scenario test in design.rs
4. Claim milestone in progress.rs
5. Run `just check` to verify

Steps 1-4 are independent of each other (no ordering dependency between the frontend fix and scenario changes). Step 5 depends on all prior steps.
