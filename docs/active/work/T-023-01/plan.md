# T-023-01 Plan: Baseline Polish Audit

## Steps

### Step 1: Update S.1.1 polish rating
- File: `tests/scenarios/src/suites/site_assessment.rs`
- Line 199: `Polish::OneStar` → `Polish::FiveStar`
- Update comment on lines 197-198 to explain Option A rationale.

### Step 2: Update S.1.3 polish rating
- File: `tests/scenarios/src/suites/site_assessment.rs`
- Line 426: `Polish::OneStar` → `Polish::FiveStar`
- Update comment above to explain Option A rationale.

### Step 3: Update S.2.2 polish rating
- File: `tests/scenarios/src/suites/design.rs`
- Line 355: `Polish::OneStar` → `Polish::FiveStar`
- Update comment on lines 351-354 to explain Option A rationale.

### Step 4: Run `just check`
- Verify formatting, linting, tests, and scenarios all pass.
- Capture dashboard output showing updated effective savings and polish debt.

### Step 5: Verify dashboard impact
- Confirm: total effective savings increased from 44.5 to ~68.5 min.
- Confirm: polish debt decreased from 62.0 to ~38.0 min.
- Confirm: no scenario regressions (8 pass, 0 fail).

## Testing Strategy

This ticket changes only scenario metadata (polish ratings), not any computation or
logic. Testing is:

1. **`just scenarios`** — the scenario dashboard itself IS the test. Verify:
   - All 8 scenarios still pass (no regressions).
   - S.1.1, S.1.3, S.2.2 show ★★★★★ polish.
   - Remaining 5 scenarios still show ★☆☆☆☆ polish.
   - Effective savings number increased correctly.
   - Polish debt number decreased correctly.

2. **`just test`** — workspace tests pass. The quoting.rs regression tests
   (s_3_1_regression, s_3_2_regression) exercise the computation paths and
   should be unaffected.

3. **`just lint`** — no warnings introduced.

4. **`just fmt-check`** — formatting clean.

## Commit Plan

Single atomic commit: "Audit polish ratings: auto ★★★★★ for pure-computation scenarios"
