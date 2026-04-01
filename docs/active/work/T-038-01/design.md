# T-038-01 Design: Fix PDF Assertion

## Options evaluated

### Option 1: Verify at API level + PDF validity check

Replace the raw byte string search with:
- Assert `%PDF-` magic (already done)
- Assert reasonable PDF size (>1 KB — a real PDF with 3 tiers of content)
- Assert `Content-Type: application/pdf` header if accessible

**Pros**: Zero new deps. Doesn't duplicate S.3.1/S.3.2 quote math assertions.
Correctly scopes S.3.3 to what it uniquely proves: the pipeline works.

**Cons**: Doesn't verify content inside the PDF. A PDF with the wrong numbers
would pass. But S.3.1 already catches wrong numbers at the quote API level.

### Option 2: Use a PDF text extraction library

Add `pdf-extract` or `lopdf` to dev-deps, extract text layer, search for
dollar amounts.

**Pros**: Verifies rendered content.

**Cons**: New dependency for one test assertion. PDF text extraction is
fragile (font encoding, text fragmentation). Ticket explicitly recommends
against this.

### Option 3: Verify ProposalDocument struct before rendering

Intercept the `ProposalDocument` that gets passed to `render_proposal()` and
assert the quote totals are correct in the struct. Then trust Typst to render
them correctly.

**Pros**: Verifies the data entering the renderer without depending on PDF
internals.

**Cons**: The proposal endpoint constructs `ProposalDocument` internally — we
can't easily inspect it from the scenario test without refactoring the handler
or adding a debug endpoint. Over-engineering for a test fix.

## Decision: Option 1

The ticket recommends Option 1 or 3. Option 3 requires handler refactoring
that's out of scope. Option 1 is the simplest change that correctly scopes
S.3.3's responsibility:

1. **Remove** the `String::from_utf8_lossy` + `contains("1530")` assertion
2. **Keep** the existing `%PDF-` magic check
3. **Add** a minimum size check (PDF with 3 tiers of quotes, narrative, and
   branding should be well over 1 KB — use 10 KB as the ticket suggests)
4. **Add** a comment explaining why we don't check content: quote math is
   verified by S.3.1/S.3.2, and Typst glyph encoding makes byte search invalid

This aligns with the ticket's own analysis: "S.3.3's unique value is proving:
the PDF generation pipeline works end-to-end (API → narrative → Typst → PDF
bytes)."

## Rejected alternatives

- **Lowering size threshold**: Even 1 KB is generous — a real 3-tier proposal
  PDF should be much larger. The ticket suggests >10 KB. Using 10 KB prevents
  false passes from an empty/stub PDF.
- **Adding a separate quote API call in S.3.3**: Duplicates S.3.1 work.
  S.3.3 should test the PDF pipeline, not re-test quoting.
