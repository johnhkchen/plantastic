# T-038-01 Plan: Fix PDF Assertion

## Steps

### Step 1: Replace the broken assertion

In `tests/scenarios/src/suites/quoting.rs`, replace lines 907-917:

1. Remove the `String::from_utf8_lossy` + `contains("1530")` block
2. Replace with a minimum size assertion (>10 KB)
3. Update the comment to explain: quote math verified by S.3.1/S.3.2, Typst
   glyph encoding makes raw byte search invalid, size check confirms the
   pipeline produced a substantive PDF

### Step 2: Verify with `just check`

Run `just check` (no `DATABASE_URL`) to confirm:
- `cargo fmt --check` passes
- `cargo clippy` passes
- All workspace tests pass
- Scenario dashboard runs without regression

S.3.3 will show `Blocked` without `DATABASE_URL` — that's correct behavior.

## Testing strategy

- **Primary**: `just check` — the quality gate
- **With DB**: When `DATABASE_URL` is set, S.3.3 should pass at ★★☆☆☆
  (the broken assertion was the only failure point; magic byte check and
  the rest of the pipeline already work)
- **No new tests needed**: This is a test fix, not a feature. The existing
  scenario test IS the test.

## Verification criteria

1. `just check` passes cleanly
2. No scenario regressions (dashboard numbers same or higher)
3. The removed assertion (`contains("1530")`) does not appear anywhere
4. The replacement assertion is meaningful (not a no-op)
