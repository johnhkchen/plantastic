---
id: T-038-01
story: S-038
title: fix-pdf-assertion
type: task
status: open
priority: critical
phase: research
depends_on: []
---

## Context

S.3.3 (Branded PDF export) fails because the test searches for `"1530"` in raw PDF bytes. Typst encodes text as glyph IDs in the PDF content stream, not raw ASCII, so `String::from_utf8_lossy` can't find the total.

## Acceptance Criteria

- S.3.3 scenario test verifies PDF correctness without relying on raw byte string search
- Options (pick one):
  1. **Verify at API level**: assert the Quote JSON totals are correct before PDF rendering, then only check PDF is valid (%PDF- magic, reasonable size). The quote math is already verified by S.3.1/S.3.2.
  2. **Use a PDF text extractor**: add `pdf-extract` or `lopdf` to dev-deps, extract text layer, search that.
  3. **Check Typst input**: verify the ProposalDocument struct passed to render_proposal() contains the correct totals. The rendering is trusted (Typst is a mature tool).
- Recommendation: Option 1 or 3 — don't add a PDF parsing dep just for a test assertion. The quote math is the system's responsibility; Typst rendering is a library's responsibility.
- S.3.3 passes at ★★☆☆☆ with DATABASE_URL
- `just check` (no DATABASE_URL) still passes

## Implementation Notes

- The current test flow: create project → zones → materials → assignments → GET /proposal → check PDF bytes
- The quote totals ($1,530.00 patio, $88.89 mulch, $130.00 edging) are already verified by S.3.1 via the quote API
- S.3.3's unique value is proving: the PDF generation pipeline works end-to-end (API → narrative → Typst → PDF bytes)
- Checking `%PDF-` magic + `Content-Type: application/pdf` + reasonable size (>10KB) is sufficient for the pipeline proof
