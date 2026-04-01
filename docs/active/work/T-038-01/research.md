# T-038-01 Research: Fix PDF Assertion

## Problem

S.3.3 (Branded PDF export) fails because the test at
`tests/scenarios/src/suites/quoting.rs:912-917` searches for `"1530"` or
`"1,530"` in raw PDF bytes via `String::from_utf8_lossy`. Typst encodes text
as glyph IDs in the PDF content stream, not raw ASCII, so the substring search
always fails.

## Current test flow (quoting.rs:662-920)

1. Check `DATABASE_URL` → TwoStar integration test
2. Create tenant, project, 3 zones, 3 materials, tier assignments
3. `GET /projects/{id}/proposal` → raw PDF bytes
4. Assert `%PDF-` magic bytes (lines 898-905) — **this works**
5. Assert `"1530"` or `"1,530"` in lossy UTF-8 of PDF bytes — **this fails**
6. Return `ScenarioOutcome::Pass(TwoStar, OneStar)` if all pass

## What S.3.3 actually proves

S.3.3's unique value is the **pipeline proof**: API → quote computation →
narrative generation (mock) → Typst rendering → valid PDF bytes. The quote
math ($1,530.00 patio, $88.89 mulch, $130.00 edging) is already verified by
S.3.1 and S.3.2 through the quote API.

## PDF generation pipeline

1. `crates/plantastic-api/src/routes/proposals.rs:23-139` — handler
   - Loads project, tenant, zones, materials, tier assignments
   - Calls `pt_quote::compute_quote()` for 3 tiers
   - Builds `ProposalInput`, generates narrative via `state.proposal_generator`
   - Constructs `ProposalDocument` with quotes + branding + narrative
   - Calls `pt_proposal::render_proposal(&doc)` in `spawn_blocking`
   - Returns PDF bytes with `Content-Type: application/pdf`

2. `crates/pt-proposal/src/render.rs:207-234` — `render_proposal()`
   - Converts `ProposalDocument` → `TemplateData` (pre-formats dollars)
   - Serializes to JSON, passes to Typst engine as input variable
   - Compiles Typst template → `PagedDocument` → PDF bytes

3. `crates/pt-proposal/templates/proposal.typ` — Typst template
   - Three-tier comparison table with formatted dollar amounts
   - Text is typeset by Typst → glyph IDs in PDF stream (not ASCII)

## Key types

- `ProposalDocument` (render.rs:32-43): project_name, project_address, date,
  branding, narrative, good_quote, better_quote, best_quote
- `Quote` (pt-quote): tier, line_items, subtotal, tax, total (Decimal)
- `TemplateData` (render.rs:47-58): serializable version with formatted strings

## Existing PDF validity test

`crates/pt-proposal/tests/render_test.rs:196-208` — `render_produces_valid_pdf()`
already asserts `%PDF-` magic on a mock document. This is a unit test (no DB).

## Content-Type verification

The API handler sets `Content-Type: application/pdf` (proposals.rs:131).
The scenario test currently does NOT verify this header — it only checks
status code and raw bytes.

## Constraints

- S.3.3 must pass at ★★☆☆☆ with `DATABASE_URL`
- `just check` (no `DATABASE_URL`) must still pass — S.3.3 returns `Blocked`
- No new crate dependencies (ticket recommends against `pdf-extract`/`lopdf`)
- Quote math already verified by S.3.1/S.3.2 — don't duplicate
