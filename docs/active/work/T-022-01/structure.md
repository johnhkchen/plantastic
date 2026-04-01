# T-022-01 Structure: Polish Enum + Dashboard

## Files Modified

### 1. `tests/scenarios/src/registry.rs`

**Add `Polish` enum** (after `Integration` impl block, ~line 100):
- `Polish` enum: `OneStar` through `FiveStar`
- `impl Polish`: `stars()`, `weight()`, `label()` — identical shape to `Integration`
- Same `#[derive(Debug, Clone, Copy)]` and `#[allow(dead_code, clippy::enum_variant_names)]`

**Modify `ScenarioOutcome::Pass`**:
- Change `Pass(Integration)` to `Pass(Integration, Polish)`

**Modify `ScenarioOutcome::status_label()`**:
- Destructure as `Pass(int_level, pol_level)`
- Format: `PASS {int_level.label()} / {pol_level.label()}`

**Modify `ScenarioOutcome::effective_minutes()`**:
- Destructure as `Pass(int_level, pol_level)`
- Formula: `raw_minutes * (int_level.stars() + pol_level.stars()) as f64 / 10.0`

**No change to**: `symbol()`, `counts_as_delivered()`, `Scenario`, `ScenarioResult`
— these use wildcard patterns on `Pass`.

### 2. `tests/scenarios/src/report.rs`

**Modify `print_dashboard()`**:
- Add formula explanation line after "Effective savings" line
- Add "Polish debt" summary line
- Add legend line explaining dual rating format

**Modify `print_area_section()`**:
- No structural change — all scenario outcome access goes through methods

**No change to**: `print_milestone_section()`, `progress_bar()`,
`count_by_status()`, `wrap_text()`, `exit_code()`, `prerequisite_summary()`
— these either use wildcard patterns or don't touch `Pass`.

### 3. `tests/scenarios/src/suites/site_assessment.rs`

3 return sites → add `Polish::OneStar`:
- Line 199: `Pass(Integration::OneStar)` → `Pass(Integration::OneStar, Polish::OneStar)`
- Line 335: `Pass(Integration::TwoStar)` → `Pass(Integration::TwoStar, Polish::OneStar)`
- Line 426: `Pass(Integration::OneStar)` → `Pass(Integration::OneStar, Polish::OneStar)`

Add `Polish` to import line.

### 4. `tests/scenarios/src/suites/design.rs`

3 return sites → add `Polish::OneStar`:
- Line 136: `Pass(Integration::TwoStar)` → `Pass(Integration::TwoStar, Polish::OneStar)`
- Line 355: `Pass(Integration::OneStar)` → `Pass(Integration::OneStar, Polish::OneStar)`
- Line 477: `Pass(Integration::TwoStar)` → `Pass(Integration::TwoStar, Polish::OneStar)`

Add `Polish` to import line.

### 5. `tests/scenarios/src/suites/quoting.rs`

4 return sites → add `Polish::OneStar`:
- Line 348: `Pass(Integration::TwoStar)` → `Pass(Integration::TwoStar, Polish::OneStar)`
- Line 653: `Pass(Integration::TwoStar)` → `Pass(Integration::TwoStar, Polish::OneStar)`
- Line 826: `Pass(Integration::ThreeStar)` → `Pass(Integration::ThreeStar, Polish::OneStar)`
- Line 1054: `Pass(Integration::ThreeStar)` → `Pass(Integration::ThreeStar, Polish::OneStar)`

Add `Polish` to import line.

### 6. `tests/scenarios/src/suites/crew_handoff.rs` — NO CHANGE

All scenarios return `NotImplemented`. No `Pass` sites. No `Integration` import.

### 7. `tests/scenarios/src/suites/infrastructure.rs` — NO CHANGE

All scenarios return `NotImplemented`. No `Pass` sites. No `Integration` import.

## Module boundaries

All changes are within `tests/scenarios/src/`. No other crates reference
`ScenarioOutcome` or `Integration`. The change is fully contained.

## Public interface changes

- `ScenarioOutcome::Pass` gains a second field — breaking for external constructors
  and destructuring, but no external consumers exist
- `Polish` enum is new public API
- `effective_minutes()` return values change due to new formula
