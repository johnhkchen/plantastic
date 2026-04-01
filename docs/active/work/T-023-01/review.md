# T-023-01 Review: Baseline Polish Audit

## Summary

Audited all 8 passing scenarios against the polish rubric. Applied Option A from the
ticket: pure computation scenarios (OneStar integration) auto-rated at Polish ★★★★★
since they have no UX surface to polish. All other scenarios remain at Polish ★☆☆☆☆.

## Files Modified

| File | Change |
|------|--------|
| `tests/scenarios/src/suites/site_assessment.rs` | S.1.1 line 200: Polish::OneStar → FiveStar |
| `tests/scenarios/src/suites/site_assessment.rs` | S.1.3 line 427: Polish::OneStar → FiveStar |
| `tests/scenarios/src/suites/design.rs` | S.2.2 line 356: Polish::OneStar → FiveStar |

3 one-line changes + 3 comment additions across 2 files.

## Per-Scenario Audit Results

| Scenario | Integration | Polish Before | Polish After | Rationale |
|----------|-------------|---------------|--------------|-----------|
| S.1.1 Scan processing | ★☆☆☆☆ | ★☆☆☆☆ | **★★★★★** | Pure computation, no UX (Option A) |
| S.1.2 Satellite pre-pop | ★★☆☆☆ | ★☆☆☆☆ | ★☆☆☆☆ | API exists but no loading/error/empty UX |
| S.1.3 Sun exposure | ★☆☆☆☆ | ★☆☆☆☆ | **★★★★★** | Pure computation, no UX (Option A) |
| S.2.1 Zone drawing | ★★☆☆☆ | ★☆☆☆☆ | ★☆☆☆☆ | UI exists but no polish verified |
| S.2.2 Material catalog | ★☆☆☆☆ | ★☆☆☆☆ | **★★★★★** | Pure computation, no UX (Option A) |
| S.2.4 3D preview | ★★☆☆☆ | ★☆☆☆☆ | ★☆☆☆☆ | Protocol error support ≠ UX polish |
| S.3.1 Quantity from geo | ★★★☆☆ | ★☆☆☆☆ | ★☆☆☆☆ | ThreeStar int. → Option A doesn't apply |
| S.3.2 Three-tier quotes | ★★★☆☆ | ★☆☆☆☆ | ★☆☆☆☆ | ThreeStar int. → Option A doesn't apply |

## Dashboard Impact

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Effective savings | 44.5 min | 68.5 min | +24.0 min |
| Budget coverage | 18.5% | 28.5% | +10.0% |
| Polish debt | 62.0 min | 38.0 min | -24.0 min |
| Passing scenarios | 8 | 8 | 0 (no regressions) |

## Test Coverage

- `just check` passes (format + lint + test + scenarios).
- All 8 scenarios still pass. No regressions.
- Quoting regression tests (s_3_1_regression, s_3_2_regression) unaffected.
- 9 scenarios remain NotImplemented (unchanged).

## Open Concerns

### S.3.1/S.3.2 Computation Path Integration Rating Bug

The computation fallback paths for S.3.1 and S.3.2 return `Integration::ThreeStar`
(quoting.rs:826, 1054) but only call `compute_quote()` directly — no API, no UI, no
persistence. By the integration rubric, this is OneStar ("pure computation works in
isolation"). If corrected to OneStar, Option A would apply and their polish would
auto-rate to ★★★★★, adding ~16.0 min effective savings.

**Recommendation**: File a separate ticket to fix the integration ratings on these
computation fallback paths. This was not addressed in T-023-01 because the ticket scope
is polish audit only.

### Remaining Polish Debt

38.0 min of polish debt remains across 5 scenarios (S.1.2, S.2.1, S.2.4, S.3.1, S.3.2).
Reducing this requires actual UX work: loading states, error handling UI, empty state
prompts in the SvelteKit frontend and Bevy viewer. The scenario tests would need to
verify these UX features exist before polish ratings can increase.

### Option A Policy

The "auto ★★★★★ for pure computation" policy is now established. Future scenarios at
OneStar integration should follow this convention. When a scenario upgrades to TwoStar+
integration, its polish should be re-assessed honestly against the UX rubric.
