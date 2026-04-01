# T-012-01 Design — Catalog CRUD Page

## Situation

The catalog CRUD page is already implemented (T-006-01) and the backend API is complete (T-004-02). The remaining work is:
1. A casing bug in the frontend extrusion default
2. S.2.2 scenario test needs a real implementation
3. The pt-materials milestone needs claiming

## Decision 1: Fix extrusion casing bug

**The bug:** `buildMaterialBody()` in the catalog page defaults new material extrusion to `{ type: 'Fills', flush: true }`. Rust serde expects `"fills"` (snake_case via `rename_all`). Creating a new material would fail with a deserialization error.

**Fix:** Change to `{ type: 'fills', flush: true }`.

**Rejected:** Adding case-insensitive deserialization on the Rust side — this would be a workaround that masks the real issue and adds complexity to the backend for a frontend-only problem.

**Rejected:** Custom deserializer with aliases — same reasoning, fix the source.

## Decision 2: S.2.2 scenario test — what to test

S.2.2 is "Material catalog search and filter" (10 min savings). The full scenario includes search/filter (T-012-02). For this ticket, we test what's delivered now: catalog CRUD works.

**Option A: OneStar — domain model verification**
Test that pt-materials types correctly model a landscaper's catalog. Build several materials with different categories/units/prices using the builder pattern. Verify they serialize to JSON matching the API contract. Verify filtering by category works in-memory (proving the data model supports it).

**Option B: TwoStar — API-level test**
Spin up the Axum router in-process and POST/GET/PATCH/DELETE materials. This would prove the API works end-to-end.

**Option C: OneStar — minimal stub**
Just test that Material struct can be constructed. This would be stat-padding.

**Chosen: Option A.** Rationale:
- The scenario harness runs as a sync binary (`cargo run -p pt-scenarios`), not an async runtime with a database. TwoStar would require a running Postgres instance, which the scenario binary doesn't support.
- Option A tests real capability: multiple materials, correct serde for the API contract, category-based filtering. This is how the catalog model underpins the UI — the domain types must serialize correctly for the frontend to work.
- Option C would violate testing rule #5 (no stat-padding).

**Integration level: OneStar.** Pure computation works in isolation. Path to TwoStar: T-012-02 adds search/filter and could test via the API layer.

## Decision 3: Milestone claim

The milestone "pt-materials: catalog model + tenant layering" (unlocks S.2.2, S.3.1, S.3.2) should be claimed by this ticket. The pt-materials crate was built earlier, but the catalog PAGE that makes it usable is this ticket's delivery. The milestone note should reference:
- What: pt-materials types (Material, MaterialCategory, Unit, ExtrusionBehavior), builder, serde
- What it enables: S.2.2 (catalog UI — this ticket), S.3.1/S.3.2 (quote computation uses material pricing)
- What's still needed: search/filter (T-012-02) for full S.2.2

## Decision 4: No page changes beyond bug fix

The catalog page is complete for the acceptance criteria. No features should be added (search/filter is T-012-02, texture/photo display is cosmetic and not in AC). The only code change to the page is the extrusion casing fix.

## Scenario test design

```
s_2_2_material_catalog():
1. Build 5 materials spanning all 4 categories using Material::builder()
   - Hardscape: Travertine Pavers (SqFt, $8.50, SitsOnTop)
   - Softscape: Premium Mulch (CuYd, $45.00, Fills)
   - Edging: Steel Edging (LinearFt, $3.25, BuildsUp)
   - Fill: Pea Gravel (CuYd, $38.00, Fills)
   - Hardscape: Flagstone (SqFt, $12.00, SitsOnTop)

2. Verify JSON serialization matches API contract:
   - category serializes to snake_case string
   - unit serializes to snake_case string
   - extrusion serializes with internally tagged type
   - price_per_unit serializes as string decimal

3. Verify category filtering (in-memory):
   - Filter to hardscape → 2 materials
   - Filter to softscape → 1 material
   - Filter to edging → 1 material
   - Filter to fill → 1 material

4. Return Pass(OneStar)
```

## Files changed

| File | Change |
|---|---|
| `web/src/routes/(app)/catalog/+page.svelte` | Fix extrusion default casing |
| `tests/scenarios/src/suites/design.rs` | Implement s_2_2_material_catalog() |
| `tests/scenarios/src/progress.rs` | Claim pt-materials milestone |
