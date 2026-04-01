# T-024-01 Design: Material Callout Scenario

## Problem

S.4.3 validates that zone-material assignments carry the metadata a crew foreman needs: material name, supplier SKU, install depth, product photo reference, and extrusion behavior. Currently `NotImplemented`.

## Options considered

### Option A: Test callout via pt-quote LineItem

Build zones + materials + tier, call `compute_quote()`, check LineItem fields.

**Rejected**: LineItem only has `material_name` and `material_id` — it doesn't carry `supplier_sku`, `depth_inches`, `photo_ref`, or `extrusion`. Adding those to LineItem just for this test would be feature creep beyond the ticket scope.

### Option B: Test callout via material lookup from tier assignments

Build materials with known callout fields, build zones with known geometry, build tier assignments mapping zones to materials. For each assignment, resolve the material by ID from the catalog, then verify all callout fields are present and match independently specified expected values.

**Chosen**: This directly tests the data model's ability to carry callout data. The "callout" is the material metadata itself, resolved per zone-material pair. This requires no new types, no API routes, and proves the contract at the domain layer. Matches the acceptance criteria exactly.

### Option C: Test via JSON serialization round-trip

Similar to Option B but serialize the callout data to JSON and assert on the JSON shape.

**Rejected for primary test**: JSON serialization is already covered by S.2.2. However, we can include a JSON check as a secondary verification to prove the data survives serialization (important for API consumption).

## Design decision

**Option B** with a JSON round-trip check added as secondary verification.

### Test structure

1. **Build 3 materials** with distinct callout profiles:
   - Travertine Pavers: hardscape, has SKU, depth, photo, SitsOnTop extrusion
   - Premium Mulch: softscape, has SKU, depth, no photo, Fills extrusion
   - Steel Edging: edging, has SKU, no depth, no photo, BuildsUp extrusion

2. **Build 3 zones** with simple geometry:
   - Patio zone (12×15 ft rectangle)
   - Bed zone (8×20 ft rectangle)
   - Edging zone (10×10 ft square)

3. **Build tier assignments** mapping each zone to its material:
   - Patio → Travertine Pavers
   - Bed → Premium Mulch
   - Edging → Steel Edging

4. **For each assignment**, resolve the material by ID, verify:
   - `material.name` matches expected
   - `material.supplier_sku` matches expected (Some or None)
   - `material.depth_inches` matches expected (Some or None)
   - `material.photo_ref` matches expected (Some or None)
   - `material.extrusion` matches expected variant

5. **JSON round-trip**: serialize each material to JSON, deserialize back, verify callout fields survive.

### Expected values (independently computed in the test)

| Zone    | Material          | SKU            | Depth | Photo              | Extrusion          |
|---------|-------------------|----------------|-------|--------------------|--------------------|
| Patio   | Travertine Pavers | TRAV-12x12-NAT | 1.0"  | photos/trav.jpg    | SitsOnTop(1.0")    |
| Bed     | Premium Mulch     | MULCH-PREM-BRN | 3.0"  | (none)             | Fills(flush=true)  |
| Edging  | Steel Edging      | EDGE-STL-4IN   | (none)| (none)             | BuildsUp(4.0")     |

### Star rating

- **Integration: OneStar** — pure computation, no API/UI. Domain model test.
- **Polish: OneStar** — bare computation, no UX.
- Effective minutes: `5.0 × (1 + 1) / 10 = 1.0`

### Path to TwoStar (documented but not implemented)

Verify callout data via GET /materials API response, proving the data survives the full API round-trip. Requires DATABASE_URL.

## Risks

- None significant. This is a pure data-model test using existing crate APIs.
- The test uses `Material::builder()` which is already proven in S.2.2.
