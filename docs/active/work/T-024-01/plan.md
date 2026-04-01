# T-024-01 Plan: Material Callout Scenario

## Steps

### Step 1: Add missing imports to crew_handoff.rs

Add `Integration` and `Polish` to the use statement (needed for the Pass variant).

### Step 2: Implement s_4_3_material_callouts()

Replace the stub with the full test function:

1. Build 3 materials with known callout fields using `Material::builder()`:
   - Travertine Pavers: SKU "TRAV-12x12-NAT", depth 1.0, photo "photos/trav.jpg", SitsOnTop(1.0)
   - Premium Mulch: SKU "MULCH-PREM-BRN", depth 3.0, no photo, Fills(flush=true)
   - Steel Edging: SKU "EDGE-STL-4IN", no depth, no photo, BuildsUp(4.0)

2. Build 3 zones:
   - Patio: 12×15 ft rectangle
   - Bed: 8×20 ft rectangle
   - Edging: 10×10 ft square

3. Build a Tier(Good) with assignments mapping each zone → material.

4. For each assignment, resolve material by ID, verify all callout fields against independently specified expected values. Return `ScenarioOutcome::Fail(...)` on any mismatch.

5. JSON round-trip: serialize each material, deserialize, verify callout fields survive.

6. Return `ScenarioOutcome::Pass(Integration::OneStar, Polish::OneStar)`.

### Step 3: Run `just check`

Verify format, lint, test, and scenarios all pass. Confirm S.4.3 flips from NotImplemented to Pass and dashboard shows Crew Handoff at 1.0/30.0 min.

## Verification criteria

- `just scenarios` shows S.4.3 as PASS ★☆☆☆☆ / ★☆☆☆☆
- Crew Handoff area goes from 0.0 to 1.0 effective minutes
- Total effective savings increases from 44.5 to 45.5 min
- `just check` passes with no regressions
- No other scenarios change status

## Testing strategy

This IS the test — S.4.3 is itself a scenario test. No separate unit tests needed. The scenario verifies:
- Material builder produces objects with correct callout fields
- Tier assignment lookup resolves materials correctly
- All callout fields (name, SKU, depth, photo, extrusion) are present and correct
- JSON serialization preserves callout data
