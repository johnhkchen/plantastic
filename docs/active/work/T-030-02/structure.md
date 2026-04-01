# T-030-02 Structure: Proposal API Route

## Files Modified

### `crates/plantastic-api/src/routes/mod.rs`
- Add `pub mod proposals;`
- Merge `proposals::routes()` into the router

### `crates/plantastic-api/src/routes/proposals.rs` (NEW)
- `pub fn routes() -> Router<AppState>` — registers `GET /projects/{id}/proposal`
- `async fn get_proposal(...)` — handler
- `fn build_proposal_input(...)` — converts quotes + zones + materials into `ProposalInput`
- `fn format_dollars(d: Decimal) -> String` — dollar formatting for TierInput.total (or import from pt-proposal)

### `crates/pt-proposal/src/render.rs`
- Make `format_dollars` pub (currently private)

### `crates/pt-proposal/src/lib.rs`
- Re-export `render::format_dollars`

### `tests/scenarios/src/api_helpers.rs`
- Add `pub async fn api_call_raw(...)` — returns `(StatusCode, Vec<u8>)` for non-JSON responses

### `tests/scenarios/src/suites/quoting.rs`
- Implement `s_3_3_branded_pdf()` — full API test
- Add `async fn s_3_3_api()` — the actual test body

### `tests/scenarios/src/progress.rs`
- Claim milestone "pt-pdf: branded quote PDF generation" with `delivered_by: Some("T-030-02")`

## Module Boundaries

```
GET /projects/{id}/proposal
    │
    ├─ verify_project_tenant()          [routes/zones.rs — existing]
    ├─ pt_repo::project::get_by_id()    [pt-repo — existing]
    ├─ pt_repo::tenant::get_by_id()     [pt-repo — existing]
    ├─ pt_repo::zone::list_by_project() [pt-repo — existing]
    ├─ pt_repo::material::list_by_tenant() [pt-repo — existing]
    ├─ pt_repo::tier_assignment::get_by_project_and_tier() × 3 [pt-repo — existing]
    │
    ├─ shared::zone_rows_to_zones()     [routes/shared.rs — existing]
    ├─ shared::material_rows_to_materials() [routes/shared.rs — existing]
    ├─ shared::build_tier() × 3         [routes/shared.rs — existing]
    │
    ├─ pt_quote::compute_quote() × 3    [pt-quote — existing]
    │
    ├─ build_proposal_input()           [routes/proposals.rs — NEW]
    ├─ state.proposal_generator.generate() [pt-proposal trait — existing]
    │
    ├─ pt_proposal::ProposalDocument    [pt-proposal — existing]
    ├─ pt_proposal::TenantBranding      [pt-proposal — existing]
    └─ pt_proposal::render_proposal()   [pt-proposal — existing, spawn_blocking]
```

## Public Interface

```rust
// Route registration
GET /projects/{id}/proposal → 200 application/pdf | 400 | 404 | 500

// Response headers
Content-Type: application/pdf
Content-Disposition: attachment; filename="proposal-{sanitized_name}.pdf"
```

## No New Dependencies

All crates already in plantastic-api's Cargo.toml: pt-repo, pt-project, pt-materials, pt-quote, pt-proposal, tokio, axum, uuid.

`pt_repo::tenant::get_by_id` is the only repo call not previously used in routes — but pt-repo is already a dependency.
