# T-030-02 Design: Proposal API Route

## Decision: Direct PDF Response

**Chosen approach:** `GET /projects/{id}/proposal` returns raw PDF bytes directly.

- `Content-Type: application/pdf`
- `Content-Disposition: attachment; filename="proposal-{project_name}.pdf"`
- No S3 caching in v1 (optional enhancement later per ticket)

**Why:** Simplest path to S.3.3. The route mirrors the quote route's data loading but the output is bytes, not JSON. S3 caching adds complexity (cache key = hash of project+assignments version) that isn't required by the acceptance criteria. Can be added later without changing the API contract — the response is always PDF bytes.

**Rejected: JSON wrapper with S3 URL.** Would require the client to make two requests (get URL, then download). The scene route uses this because GLBs are large and reused. Proposals are small (<1MB) and generated on demand.

## Pipeline Sequence

1. **Verify tenant** — `verify_project_tenant()`
2. **Load project + tenant** — need `ProjectRow` for name/address, `TenantRow` for branding
3. **Load zones + materials** — same as quote route
4. **Load 3 tier assignments** — `get_by_project_and_tier()` × 3
5. **Validate** — at least one tier must have assignments (else 400)
6. **Compute 3 quotes** — `pt_quote::compute_quote()` × 3
7. **Build ProposalInput** — convert quotes + zones into TierInput/ZoneSummary
8. **Generate narrative** — `state.proposal_generator.generate()` (async, uses mock in tests)
9. **Build ProposalDocument** — combine branding + quotes + narrative
10. **Render PDF** — `render_proposal()` (CPU-bound, use `spawn_blocking`)
11. **Return bytes** — with correct headers

## TierInput Construction

Each `TierInput` needs:
- `tier_level`: "Good" / "Better" / "Best" (from TierLevel Debug format)
- `total`: pre-formatted "$X,XXX.XX" string (use the same `format_dollars` pattern)
- `zones`: Vec<ZoneSummary> built from zones + assignments + materials

The `ZoneSummary` needs zone label, zone_type, area_sqft, and list of material names assigned to that zone in this tier.

## Error Cases

- **404:** project not found or wrong tenant
- **400:** no tier assignments for any tier — "project has no material assignments"
- **500:** narrative generation failure, PDF render failure

## Handling Empty Tiers

Some tiers may have no assignments while others do. Include all three in the proposal — a tier with no assignments gets an empty quote ($0). This is more useful than failing: it shows the client "Good tier: not yet configured" while Better/Best have content.

Actually, re-reading the acceptance criteria: "400 (no assignments)" means the route should fail if there are NO assignments at all. If at least one tier has assignments, proceed. Empty tiers get $0 quotes.

## TenantBranding Mapping

`TenantRow` → `TenantBranding`:
- `name` → `company_name`
- `logo_url` → `logo_url`
- `brand_color` → `primary_color`
- `contact` (JSON) → extract `phone` and `email` if present

## Scenario Test Strategy

Need a new `api_call_raw()` helper in `api_helpers.rs` that returns `(StatusCode, Vec<u8>)` instead of parsing JSON. The S.3.3 test:

1. Reuse S.3.1/S.3.2 setup pattern (create project, zones, materials, 3 tier assignments)
2. `GET /projects/{id}/proposal`
3. Assert status 200
4. Assert response starts with `%PDF-` (bytes `[0x25, 0x50, 0x44, 0x46, 0x2D]`)
5. Search PDF bytes for expected dollar total strings (e.g., "1,530.00" from the patio)
6. Integration level: TwoStar (full API with real DB, mock LLM)

## Dollar Formatting in Route

The `format_dollars` function in `render.rs` is private. We need a way to format totals for `TierInput.total`. Options:
- Make `format_dollars` pub in pt-proposal
- Duplicate a simple formatter in the route
- Use the Quote's `total` Decimal and format inline

Best: make `render::format_dollars` pub in pt-proposal. It's the canonical formatter and avoids duplication.
