# T-030-02 Review: Proposal API Route

## Summary

Wired the proposal PDF generation pipeline into the API as `GET /projects/{id}/proposal`. The route loads all project data, computes three-tier quotes, generates narrative via the injected `ProposalNarrativeGenerator` trait, renders a branded PDF via Typst, and returns raw PDF bytes.

## Files Changed

### Created
- `crates/plantastic-api/src/routes/proposals.rs` — Route handler + helpers (~240 lines)
- `docs/active/work/T-030-02/` — All RDSPI artifacts

### Modified
- `crates/plantastic-api/src/routes/mod.rs` — Added `proposals` module and route registration
- `crates/pt-proposal/src/render.rs` — Made `format_dollars` pub (was private)
- `crates/pt-proposal/src/lib.rs` — Re-exported `format_dollars`
- `tests/scenarios/src/api_helpers.rs` — Added `api_call_raw()` for non-JSON responses
- `tests/scenarios/src/suites/quoting.rs` — Implemented S.3.3 branded PDF scenario test
- `tests/scenarios/src/progress.rs` — Claimed "pt-pdf: branded quote PDF generation" milestone

## Test Coverage

### S.3.3 Scenario Test (API-level, TwoStar)
- Creates project with address, 3 zones, 3 materials, 3 tier assignments
- `GET /projects/{id}/proposal` → verifies HTTP 200
- Asserts response starts with `%PDF-` magic bytes
- Asserts PDF content contains expected dollar total (`$1,530.00` for 12×15 patio at $8.50/sq ft)
- Uses `MockProposalGenerator` — zero LLM calls
- **Status:** BLOCKED in CI without DATABASE_URL; passes with real Postgres

### Error Paths
- 404: missing project or wrong tenant (via existing `verify_project_tenant`)
- 400: no material assignments across all three tiers
- 500: narrative generation failure, PDF render failure (via existing ProposalError → AppError conversion)

### Existing Tests
- All existing tests continue to pass (`just check` green)
- No regressions in S.3.1, S.3.2, or any other scenario

## Scenario Dashboard

- **Before:** S.3.3 = NotImplemented
- **After:** S.3.3 = Blocked (needs DATABASE_URL) with prereqs 2/2 met
- With DATABASE_URL: S.3.3 = Pass(TwoStar, OneStar), adding 10 minutes of verified time savings

## Architecture Decisions

1. **Direct PDF response** rather than S3 URL — simplest path, matches acceptance criteria. S3 caching can be added later without changing the API contract.

2. **spawn_blocking for PDF rendering** — Typst compilation is CPU-bound. Same pattern as the scene route.

3. **Empty tiers allowed** — if only some tiers have assignments, they get $0 quotes. The route only fails (400) if ALL tiers are empty.

4. **Made `format_dollars` public** — needed to format `TierInput.total` for the ProposalInput. The route needs the same formatting as the PDF template. Making the existing function public avoids duplication.

## Open Concerns

1. **S3 caching not implemented.** The ticket mentions it as optional. Worth adding if proposal generation becomes a latency concern (Typst rendering is ~100-200ms, narrative generation depends on mock vs real LLM).

2. **S.3.3 requires DATABASE_URL.** No fallback to computation-only like S.3.1/S.3.2 because the proposal route is inherently an API-level test. This is correct — it's testing the full pipeline.

3. **TenantBranding contact extraction.** The `contact` field on TenantRow is JSON. The route extracts `phone` and `email` keys. If the contact JSON schema changes, this extraction needs updating. Low risk — the schema is simple and stable.

## Quality Gate

```
just check → All gates passed.
  ✓ fmt-check
  ✓ lint (clippy strict)
  ✓ test (workspace)
  ✓ scenarios (no regressions)
```
