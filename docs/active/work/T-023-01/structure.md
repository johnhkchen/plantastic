# T-023-01 Structure: Baseline Polish Audit

## Files Modified

### 1. `tests/scenarios/src/suites/site_assessment.rs`

**Line 199** — S.1.1 return value:
```
ScenarioOutcome::Pass(Integration::OneStar, Polish::OneStar)
→ ScenarioOutcome::Pass(Integration::OneStar, Polish::FiveStar)
```
Update comment above to explain Option A rationale.

**Line 426** — S.1.3 return value:
```
ScenarioOutcome::Pass(Integration::OneStar, Polish::OneStar)
→ ScenarioOutcome::Pass(Integration::OneStar, Polish::FiveStar)
```
Update comment above to explain Option A rationale.

### 2. `tests/scenarios/src/suites/design.rs`

**Line 355** — S.2.2 return value:
```
ScenarioOutcome::Pass(Integration::OneStar, Polish::OneStar)
→ ScenarioOutcome::Pass(Integration::OneStar, Polish::FiveStar)
```
Update comment above to explain Option A rationale.

## Files NOT Modified (with rationale)

### `tests/scenarios/src/suites/quoting.rs`
- S.3.1 (line 826) and S.3.2 (line 1054) computation paths: stays at Polish::OneStar.
  Integration rating is ThreeStar (not OneStar), so Option A doesn't apply.
  The integration rating bug is noted but out of scope.
- S.3.1 (line 348) and S.3.2 (line 653) API paths: stays at Polish::OneStar.
  Integration is TwoStar; no UX polish verified.

### `tests/scenarios/src/suites/site_assessment.rs`
- S.1.2 (line 335): stays at Polish::OneStar. TwoStar integration, no UX polish verified.

### `tests/scenarios/src/suites/design.rs`
- S.2.1 (line 136): stays at Polish::OneStar. TwoStar integration, no UX polish verified.
- S.2.4 (line 477): stays at Polish::OneStar. TwoStar integration, no UX polish verified.

### `tests/scenarios/src/registry.rs`
- Polish enum definition unchanged. The existing 5-star enum covers all needed ratings.

### `tests/scenarios/src/report.rs`
- Dashboard rendering unchanged. Formula already handles all Polish variants.

## Change Pattern

Each change follows the same pattern:
1. Update the `Polish::OneStar` → `Polish::FiveStar` in the `ScenarioOutcome::Pass()` return.
2. Update the inline comment to explain the rating: "FiveStar polish: pure computation,
   no UX surface (Option A from T-023-01)."

Total: 3 one-line changes + 3 comment updates across 2 files.

## No New Files Created

No new modules, types, or test files needed. This is a data-only change to existing
scenario return values.
