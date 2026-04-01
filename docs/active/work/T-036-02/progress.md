# T-036-02 Progress: BAML Planter Estimation

## Completed

### Step 1: planter.baml + client regeneration
- Created `baml_src/planter.baml` with PlantSelection, PlanterStyle, PlanterEstimate types
- EstimatePlanter function with landscape designer prompt
- powell_market_gap test case
- Fixed Jinja conditional syntax (`{% if %}` not `{{ #if }}`)
- `baml-cli generate` succeeded, new types in baml_client

### Step 2: pt-planter crate scaffold
- `crates/pt-planter/Cargo.toml` with workspace deps
- `crates/pt-planter/src/error.rs` with PlanterError enum

### Step 3: compute.rs — quantity and cost computation
- `compute_style()` with plant_count, soil_volume_cuyd, cost formulas
- `infer_price_category()` keyword matcher for grasses/annuals/succulents
- Pricing constants: $15 grass, $8 annual, $12 succulent/default, $45/cuyd soil
- Area split evenly among plant selections per style
- 5 unit tests, all passing

### Step 4: estimator.rs — trait + BAML impl
- PlanterInput struct, PlanterEstimator trait
- BamlPlanterEstimator calling B.EstimatePlanter.call()

### Step 5: mock.rs — Powell & Market fixture
- 3 styles with real SF Bay Area plants:
  - Drought-Tolerant Groundcover: Dymondia (12"), Carex pansa (12"), 4" soil
  - Mixed Perennial Bed: Salvia (12"), Coral Bells (10"), Blue Fescue (8"), 8" soil
  - Urban Woodland Edge: Hakonechloa (8"), Coral Bells (8"), Mondo Grass (4"), 10" soil
- MockFailingEstimator for error paths
- Adjusted spacing to ensure realistic cost ordering (budget < premium)

### Step 6: claude_cli.rs — CLI fallback
- ClaudeCliEstimator following exact pt-features/pt-proposal pattern
- build_prompt, call_cli, B.EstimatePlanter.parse()

### Step 7: lib.rs wiring
- All modules declared, all public types re-exported
- Clean compile, no warnings

### Step 8: Integration tests
- 4 integration tests in tests/estimate.rs
- All passing: mock_returns_three_styles, mock_computed_costs_match_arithmetic,
  failing_estimator_returns_error, empty_adjacent_features_works

### Step 9: Scenario S.2.3
- Implemented previously-NotImplemented S.2.3 "Plant recommendations"
- Tests gap → mock estimator → compute → verify arithmetic
- Passes at OneStar integration / OneStar polish

### Step 10: Quality gate
- `cargo check -p pt-planter -p pt-scenarios`: clean
- `cargo clippy -p pt-planter -p pt-scenarios -- -D warnings`: clean
- `cargo test -p pt-planter`: 9/9 pass
- `cargo run -p pt-scenarios`: S.2.3 passes, no regressions
- `just check` blocked by pre-existing pt-analyzer compile errors from T-035-01
  (concurrent ticket with incomplete baml_client integration — not introduced by this work)

## Deviations from Plan
- Used S.2.3 (existing NotImplemented scenario) instead of creating new S.4.1
- Adjusted mock fixture spacing values to ensure cost ordering (budget < mid < premium)
- BAML conditional syntax: `{% if %}` not `{{ #if }}` (discovered during generate)
