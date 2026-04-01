# T-030-02 Plan: Proposal API Route

## Step 1: Export `format_dollars` from pt-proposal

- Make `format_dollars` in `crates/pt-proposal/src/render.rs` `pub`
- Re-export in `crates/pt-proposal/src/lib.rs`
- Verify: `cargo check -p pt-proposal`

## Step 2: Create the proposal route handler

File: `crates/plantastic-api/src/routes/proposals.rs`

1. `pub fn routes() -> Router<AppState>` — register `GET /projects/{id}/proposal`
2. `async fn get_proposal(tenant, state, path)`:
   - verify_project_tenant
   - Load project row, tenant row
   - Load zones, materials
   - Load 3 tier assignments (Good, Better, Best)
   - Check at least one tier has assignments → else 400
   - Compute 3 quotes
   - Build ProposalInput with TierInput/ZoneSummary
   - Call state.proposal_generator.generate()
   - Build ProposalDocument with branding, quotes, narrative
   - render_proposal via spawn_blocking
   - Return PDF bytes with correct headers

## Step 3: Register the route in mod.rs

- Add `pub mod proposals;` to `routes/mod.rs`
- Merge `proposals::routes()` into the router

- Verify: `cargo check -p plantastic-api`

## Step 4: Add `api_call_raw` helper for scenario tests

File: `tests/scenarios/src/api_helpers.rs`

- New function: `pub async fn api_call_raw(app, method, uri, tenant_id, body) -> Result<(StatusCode, Vec<u8>), String>`
- Same as `api_call` but returns raw bytes instead of parsing JSON

## Step 5: Implement S.3.3 scenario test

File: `tests/scenarios/src/suites/quoting.rs`

1. Replace `s_3_3_branded_pdf` stub with runtime check + `s_3_3_api()` call
2. `s_3_3_api()`:
   - Setup: pool, migrations, tenant, router
   - Create project with address
   - Create 3 zones (reuse S.3.1 geometry: patio 12×15, bed 8×20, edging 10×10)
   - Create 3 materials (reuse S.3.1 materials: Travertine $8.50, Mulch $45, Steel Edge $3.25)
   - Set tier assignments for all 3 tiers (same materials for simplicity)
   - GET /projects/{id}/proposal
   - Assert 200
   - Assert bytes start with `%PDF-`
   - Search bytes for "$1,530.00" (patio total in the PDF)
   - Return Pass(TwoStar, OneStar)

## Step 6: Claim milestone in progress.rs

- Update "pt-pdf: branded quote PDF generation" milestone:
  - `delivered_by: Some("T-030-02")`
  - Write note explaining what was delivered

## Step 7: Verify

- `just check` — format + lint + test + scenarios
- Run scenarios to verify S.3.3 passes

## Testing Strategy

- **Unit tests:** None needed in the route itself — the handler is pure plumbing
- **Integration test:** S.3.3 scenario covers the full API path with real DB + mock LLM
- **Verification criteria:**
  - PDF bytes start with `%PDF-`
  - PDF contains expected dollar amounts as strings
  - 404 on missing project
  - 400 on no assignments
  - MockProposalGenerator used (no LLM calls)
