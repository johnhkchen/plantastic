# T-038-01 Review: Fix PDF Assertion

## Summary

Replaced the broken raw-byte string search in S.3.3 (Branded PDF export)
with a minimum PDF size assertion. The old assertion searched for `"1530"`
in `String::from_utf8_lossy` of PDF bytes, which never worked because Typst
encodes text as glyph IDs, not raw ASCII.

## Changes

### Modified: `tests/scenarios/src/suites/quoting.rs` (lines 907-917)

**Before**: `String::from_utf8_lossy(&pdf_bytes).contains("1530")` — always
fails because Typst PDF content streams use glyph IDs.

**After**: `pdf_bytes.len() < 10_000` — asserts the PDF is substantive
(>10 KB). A 3-tier proposal with narrative, branding, and comparison table
should be well over 10 KB. Combined with the existing `%PDF-` magic check
(lines 898-905), this confirms the pipeline produced a real PDF.

Comment updated to explain rationale: quote math is verified by S.3.1/S.3.2,
Typst glyph encoding makes byte search invalid.

## Test coverage

- **S.3.3 scenario**: Fixed. Will pass at ★★☆☆☆ when DATABASE_URL is set.
  Shows BLOCKED without DATABASE_URL (correct behavior).
- **Existing unit test**: `render_produces_valid_pdf()` in
  `crates/pt-proposal/tests/render_test.rs` already tests PDF magic bytes
  on a mock document — unchanged and passing.
- **S.3.1/S.3.2**: Continue to verify quote math at the API level — unchanged.
- **No regressions**: Scenario dashboard shows same results before and after.

## Quality gate

- `cargo fmt --check`: pass
- `cargo clippy --workspace --all-targets -D warnings`: pass
- `cargo run -p pt-scenarios`: no regressions
- `just test`: pre-existing timeout failure in `pt-scan` Powell Market tests
  (SIGKILL after 60s) — unrelated to this ticket

## Open concerns

1. **Powell Market test timeouts**: `pt-scan` integration tests
   (`test_powell_market_candidates`, `test_powell_market_gaps`,
   `test_powell_market_two_clusters`) are killed after 60s. This is a
   pre-existing issue, not introduced by this ticket. Should be tracked
   separately if not already.

2. **S.3.3 requires DATABASE_URL**: Cannot fully verify the fix without a
   Postgres instance. The logical correctness is clear (removed an assertion
   that could never pass, replaced with a meaningful validity check), but
   end-to-end confirmation requires running with a database.

3. **10 KB threshold**: Chosen based on the ticket's recommendation. If the
   Typst template or narrative content changes significantly, this threshold
   may need adjustment. It's generous — actual proposal PDFs should be
   much larger.
