# T-024-01 Structure: Material Callout Scenario

## Files modified

### `tests/scenarios/src/suites/crew_handoff.rs`

**Change**: Replace `s_4_3_material_callouts()` stub (currently returns `NotImplemented`) with a real test function.

**Imports added**:
```rust
use crate::registry::{Integration, Polish, Scenario, ScenarioOutcome, ValueArea};
```
(Integration and Polish already need to be imported — currently only Scenario, ScenarioOutcome, ValueArea are imported.)

**Function signature**: unchanged — `fn s_4_3_material_callouts() -> ScenarioOutcome`

**Internal structure of the function**:

1. **Material construction block** (~20 lines)
   - Build 3 materials using `Material::builder()` from pt-materials
   - Each with distinct callout profiles (SKU, depth, photo, extrusion)
   - Store in a Vec for lookup

2. **Zone construction block** (~15 lines)
   - Build 3 zones with `geo::polygon!` macro and `Zone` from pt-project
   - Simple rectangles with known dimensions

3. **Tier assignment block** (~10 lines)
   - Build a `Tier` with `MaterialAssignment` entries mapping each zone to its material

4. **Callout verification block** (~40 lines)
   - For each assignment in the tier:
     - Look up the zone by ID
     - Look up the material by ID from the catalog
     - Assert material name matches expected
     - Assert supplier_sku matches expected
     - Assert depth_inches matches expected
     - Assert photo_ref matches expected
     - Assert extrusion matches expected variant and values

5. **JSON round-trip block** (~15 lines)
   - Serialize each material to JSON
   - Deserialize back
   - Verify callout fields survive

6. **Return** `ScenarioOutcome::Pass(Integration::OneStar, Polish::OneStar)`

**No new files created. No files deleted. No other files modified.**

## Module boundaries

- The test uses only public APIs from pt-materials and pt-project.
- No new public interfaces are introduced.
- The change is entirely within the scenario test harness — no production code is touched.

## Ordering

Single atomic change — no ordering concerns.
