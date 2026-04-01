# T-024-01 Review: Material Callout Scenario

## Summary

Implemented S.4.3 (Material callouts with supplier info) in `tests/scenarios/src/suites/crew_handoff.rs`. The scenario validates that zone-material assignments carry all the metadata a crew foreman needs: material name, supplier SKU, install depth, product photo reference, and extrusion behavior.

## Files changed

| File | Change |
|------|--------|
| `tests/scenarios/src/suites/crew_handoff.rs` | Added `Integration`/`Polish` imports; replaced `s_4_3_material_callouts()` stub with ~170-line test |

No new files created. No production code modified.

## What the test verifies

1. **Material construction**: 3 materials built via `Material::builder()` with distinct callout profiles spanning all relevant field combinations (with/without SKU, depth, photo; three extrusion variants).
2. **Tier assignment resolution**: Given a Tier with MaterialAssignment entries, the test resolves each material by ID from a catalog and verifies all callout fields against independently specified expected values.
3. **JSON round-trip**: Each material survives serde JSON serialization and deserialization with all callout fields intact.

## Test coverage

- **Callout fields tested**: name, supplier_sku, depth_inches, photo_ref, extrusion (all 5 acceptance criteria fields)
- **Material categories covered**: Hardscape, Softscape, Edging
- **Extrusion variants covered**: SitsOnTop, Fills, BuildsUp
- **Optional field combinations**: Some/None for SKU, depth, photo across different materials
- **Serialization**: JSON round-trip for all 3 materials

## Dashboard impact

- S.4.3: NotImplemented → PASS ★☆☆☆☆/★☆☆☆☆
- Crew Handoff: 0.0 → 1.0 effective minutes
- No regressions in existing scenarios

## Star rating rationale

- **Integration OneStar**: Pure computation/data-model test. No API routes or UI involved.
- **Polish OneStar**: Bare computation, no UX component.
- **Effective**: 5.0 × (1+1)/10 = 1.0 minute

## Path to higher stars

- **TwoStar**: Verify callout data via GET /materials API response (requires DATABASE_URL and Postgres). The material catalog API already returns all these fields — the upgrade is exercising the API path in the scenario.
- **ThreeStar+**: Verify callout display in the Bevy viewer UI (requires pt-scene for zone→glTF generation).

## Open concerns

None. This is a straightforward domain-model test using existing, well-tested crate APIs. All quality gates pass.

## Quality gate

```
just check — PASS
  fmt-check: ✓
  lint: ✓
  test: ✓
  scenarios: ✓ (S.4.3 PASS, no regressions)
```
