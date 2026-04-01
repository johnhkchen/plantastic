# T-036-02 Plan: BAML Planter Estimation

## Step 1: Add planter.baml and regenerate client

**Files**: `baml_src/planter.baml`
**Action**: Create BAML types (PlantSelection, PlanterStyle, PlanterEstimate) and EstimatePlanter function with prompt. Add powell_market_gap test case.
**Verify**: `baml-cli generate` succeeds, new types appear in `baml_client/types/classes.rs`, new function in `baml_client/functions/async_client.rs`

## Step 2: Create pt-planter crate scaffold

**Files**: `crates/pt-planter/Cargo.toml`, `crates/pt-planter/src/lib.rs`, `crates/pt-planter/src/error.rs`
**Action**: Create Cargo.toml with workspace deps. Create lib.rs with baml_client path include and module declarations. Create error.rs with PlanterError enum.
**Verify**: `cargo check -p pt-planter` compiles (with dead_code allowed)

## Step 3: Implement compute.rs — quantity and cost computation

**Files**: `crates/pt-planter/src/compute.rs`
**Action**: Implement `compute_style()` with plant_count, soil_volume, cost formulas. Add `infer_price_category()` keyword matcher. Define pricing constants.
**Verify**: Unit tests for compute_style with known inputs:
- 16 sqft gap, 6" spacing → plant_count = floor(16 / (6/12)^2) = floor(64) = 64
- 16 sqft, 8" soil depth → soil_volume = 16 * 8 / (12 * 27) = 0.395 cuyd
- Cost arithmetic matches independent calculation

## Step 4: Implement estimator.rs — trait + BAML impl

**Files**: `crates/pt-planter/src/estimator.rs`
**Action**: Define `PlanterInput` struct and `PlanterEstimator` trait. Implement `BamlPlanterEstimator` calling `B.EstimatePlanter.call()`.
**Verify**: Compiles. (Cannot test without API key — BAML impl tested via mock pattern)

## Step 5: Implement mock.rs — Powell & Market fixture

**Files**: `crates/pt-planter/src/mock.rs`
**Action**: Build `MockPlanterEstimator` returning 3 styles with real SF plants:
- Style 1 "Drought-Tolerant Groundcover": Dymondia (4" spacing), Carex pansa (8" spacing); 4" soil depth
- Style 2 "Mixed Perennial Bed": Salvia 'Hot Lips' (12" spacing), Heuchera (10" spacing), Blue Fescue (8" spacing); 8" soil depth
- Style 3 "Statement Planting": Japanese Maple (48" spacing), Hakonechloa (12" spacing), Mondo Grass (6" spacing); 10" soil depth
Also `MockFailingEstimator` for error paths.
**Verify**: Unit test calling mock, checking 3 styles with expected plant names

## Step 6: Implement claude_cli.rs — CLI fallback

**Files**: `crates/pt-planter/src/claude_cli.rs`
**Action**: `ClaudeCliEstimator` following exact pattern from pt-features/pt-proposal: build_prompt → call_cli → B.EstimatePlanter.parse()
**Verify**: Compiles. (Manual testing only — requires claude CLI)

## Step 7: Wire up lib.rs re-exports

**Files**: `crates/pt-planter/src/lib.rs`
**Action**: Add all module declarations and pub re-exports for trait, impls, error, compute, BAML types.
**Verify**: `cargo check -p pt-planter` clean, `cargo test -p pt-planter` all passing

## Step 8: Add integration tests

**Files**: `crates/pt-planter/tests/estimate.rs`
**Action**: Integration tests:
- Mock estimation returns 3 styles with valid fields
- Computed quantities for mock output match independent arithmetic
- Empty adjacent features still works
- Failing estimator returns error
**Verify**: `cargo test -p pt-planter` — all pass, under 10s with timed()

## Step 9: Add scenario S.4.1

**Files**: `tests/scenarios/src/suites/design.rs`
**Action**: Add scenario "Planter estimation from measured gaps" (10 min time savings). Test creates gap fixture, runs mock estimator, verifies 3 styles, verifies computed costs.
**Verify**: `cargo run -p pt-scenarios` — S.4.1 passes, no regressions

## Step 10: Quality gate

**Action**: Run `just check` (fmt + lint + test + scenarios). Fix any issues.
**Verify**: All four gates pass clean.

## Testing Strategy Summary

| Test | Type | What it verifies |
|------|------|-----------------|
| compute_style unit tests | Unit | plant_count, soil_volume, cost formulas |
| infer_price_category tests | Unit | keyword matching for plant pricing |
| mock_returns_three_styles | Integration | Mock output shape and content |
| mock_computed_costs | Integration | End-to-end: estimate → compute → verify |
| failing_estimator | Integration | Error propagation |
| S.4.1 scenario | Scenario | Full pipeline: gap → estimate → compute |
