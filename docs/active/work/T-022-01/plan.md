# T-022-01 Plan: Polish Enum + Dashboard

## Step 1: Add Polish enum to registry.rs

Add `Polish` enum with `stars()`, `weight()`, `label()` methods.
Modify `ScenarioOutcome::Pass` from `Pass(Integration)` to
`Pass(Integration, Polish)`. Update `status_label()` and
`effective_minutes()` to destructure both fields and use the new formula.

**Verify**: `cargo check -p pt-scenarios` — will fail with unresolved
`Pass` constructors in suite files, confirming the breaking change
propagates correctly.

## Step 2: Update all suite files

Update all 10 `Pass(Integration::X)` return sites to
`Pass(Integration::X, Polish::OneStar)`. Add `Polish` to imports in
`site_assessment.rs`, `design.rs`, `quoting.rs`.

**Verify**: `cargo check -p pt-scenarios` — should compile clean.

## Step 3: Update dashboard in report.rs

Add formula explanation, polish debt calculation, and legend to the
dashboard header. The polish debt is computed from the results passed to
`print_dashboard()`.

**Verify**: `cargo run -p pt-scenarios` — dashboard should render with
new format and show the expected effective minutes change.

## Step 4: Run full quality gate

`just check` — format, lint, test, scenarios all pass.

**Verify**: Exit code 0 from `just check`. Compare effective minutes
before and after — document the expected decrease due to the new formula
weighting.

## Testing strategy

No new unit tests needed. The scenario harness IS the test — running
`cargo run -p pt-scenarios` exercises every code path (all 10 Pass sites,
the formula, the display). The dashboard output is the verification.

The effective minutes will decrease because the new formula weights the
polish dimension. With all scenarios at `Polish::OneStar`:
- OneStar/OneStar: (1+1)/10 = 0.2× (was 0.2×) — unchanged
- TwoStar/OneStar: (2+1)/10 = 0.3× (was 0.4×) — drops
- ThreeStar/OneStar: (3+1)/10 = 0.4× (was 0.6×) — drops

Expected new effective total: calculated at implementation time.
