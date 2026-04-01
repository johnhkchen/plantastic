# T-012-01 Progress — Catalog CRUD Page

## Completed

### Step 1: Fix extrusion casing bug
- Changed `{ type: 'Fills', flush: true }` → `{ type: 'fills', flush: true }` in `web/src/routes/(app)/catalog/+page.svelte` line 68
- The Rust backend expects snake_case (`fills`, not `Fills`) due to `#[serde(rename_all = "snake_case")]`

### Step 2: Add pt-materials dependency to scenario crate
- Already present in `tests/scenarios/Cargo.toml` — no change needed

### Step 3: Implement S.2.2 scenario test
- Replaced stub in `tests/scenarios/src/suites/design.rs`
- Test builds 5 materials spanning all 4 categories using `Material::builder()`
- Verifies JSON serialization: snake_case categories, units, extrusion type tags, string Decimal prices
- Verifies category filtering: 2 hardscape, 1 softscape, 1 edging, 1 fill
- Verifies texture/photo refs preserved on Flagstone material
- Returns `ScenarioOutcome::Pass(Integration::OneStar)`

### Step 4: Claim pt-materials milestone
- Updated `tests/scenarios/src/progress.rs`: `delivered_by: Some("T-012-01")`
- Wrote descriptive note covering types, builder, serde, catalog page, and path to TwoStar

### Step 5: Run quality gate
- `just fmt` fixed pre-existing formatting issues in 3 files (projects.rs, project.rs, site_assessment.rs)
- `just check` passes: fmt ✓, lint ✓, test ✓, scenarios ✓

## Deviations from plan
- None. All steps executed as planned.

## Results
- S.2.2: NotImplemented → Pass ★☆☆☆☆
- Milestones: 8/20 → 9/20
- Effective savings: 41.0 min → 48.0 min (was expecting 43.0, but S.1.2 also advanced from ★☆☆☆☆ to ★★☆☆☆ accounting for the extra 5 min — this was a pre-existing change not from this ticket)
- Dashboard: 17.1% → 20.0%
