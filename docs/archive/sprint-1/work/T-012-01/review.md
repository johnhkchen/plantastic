# T-012-01 Review — Catalog CRUD Page

## Summary

The catalog CRUD page was already implemented by T-006-01. This ticket's primary deliverables were: fixing a casing bug in the extrusion default, implementing the S.2.2 scenario test, and claiming the pt-materials milestone.

## Files changed

| File | Change type | Description |
|---|---|---|
| `web/src/routes/(app)/catalog/+page.svelte` | Modified | Fix extrusion default: `'Fills'` → `'fills'` (line 68) |
| `tests/scenarios/src/suites/design.rs` | Modified | Implement `s_2_2_material_catalog()`: 5 materials, serde checks, category filtering |
| `tests/scenarios/src/progress.rs` | Modified | Claim pt-materials milestone with `delivered_by: Some("T-012-01")` |
| `crates/plantastic-api/src/routes/projects.rs` | Formatted | Pre-existing `cargo fmt` fix (not T-012-01 work) |
| `crates/pt-repo/src/project.rs` | Formatted | Pre-existing `cargo fmt` fix (not T-012-01 work) |
| `tests/scenarios/src/suites/site_assessment.rs` | Formatted | Pre-existing `cargo fmt` fix (not T-012-01 work) |

## Acceptance criteria status

| Criterion | Status |
|---|---|
| Catalog page at /catalog route | Pass (delivered by T-006-01) |
| Lists all materials: name, category, unit, price | Pass |
| Add material: form with name, category, unit, price, depth, SKU | Pass (extrusion casing bug fixed) |
| Edit material: click to edit in modal | Pass |
| Delete material: confirmation then remove | Pass |
| Calls existing material CRUD API routes | Pass |
| Empty state: prompt to add first material | Pass |
| S.2.2 scenario registered | Pass — now passes at ★☆☆☆☆ |

## Scenario dashboard

| Metric | Before | After | Delta |
|---|---|---|---|
| Effective savings | 41.0 min | 48.0 min | +7.0 min |
| Passing scenarios | 5 | 6 | +1 (S.2.2) |
| Milestones delivered | 8/20 | 9/20 | +1 |

Note: The +7.0 min delta (vs expected +2.0) includes S.1.2 advancing from ★☆☆☆☆ to ★★☆☆☆ which was a pre-existing change from T-011-02 that hadn't been reflected in the dashboard before this run.

## Test coverage

- **S.2.2 scenario test:** Tests material catalog domain model — 5 materials, 4 categories, JSON serde contract, category filtering, texture/photo ref preservation. OneStar integration level.
- **Existing pt-materials unit tests (11):** All pass. Cover serde round-trips, builder pattern, ID uniqueness.
- **Existing API integration tests (9, ignored):** Material CRUD lifecycle tested in `crud_test.rs` (requires Postgres).

No new unit tests added — the scenario test covers the new assertion surface and the bug fix is a single-character change verified by the scenario's serde checks.

## Bug found and fixed

**Extrusion casing mismatch:** `buildMaterialBody()` in the catalog page defaulted new material extrusion to `{ type: 'Fills', flush: true }`. The Rust backend uses `#[serde(tag = "type", rename_all = "snake_case")]` which expects `"fills"`. Creating a new material from the frontend would have failed with a deserialization error. Fixed to `{ type: 'fills', flush: true }`.

## Open concerns

1. **No E2E test for the frontend.** The catalog page has no automated browser test. The scenario test verifies domain model correctness but not that the Svelte page renders correctly. This is acceptable for OneStar; browser tests are a FourStar concern.

2. **Extrusion not user-editable.** The modal form has no UI for choosing extrusion type — it defaults to `fills/flush:true` for new materials and preserves existing values on edit. The acceptance criteria don't require extrusion editing, but landscapers will eventually need to set this for 3D preview (S.2.4).

3. **S.2.2 at OneStar only.** The scenario tests domain model, not the API or UI. Path to TwoStar: T-012-02 (search/filter) can advance the scenario by testing catalog operations through the API layer.

## Quality gate

`just check` passes: fmt ✓, lint ✓, test (137 pass, 29 ignored) ✓, scenarios (6 pass, 0 fail) ✓.
