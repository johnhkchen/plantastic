# T-030-02 Research: Proposal API Route

## Existing Route Pattern

The quote route (`crates/plantastic-api/src/routes/quotes.rs`) is the reference:
- `GET /projects/{id}/quote/{tier}` — loads zones, tier assignments, materials via `pt-repo`
- Converts repo types → domain types via shared helpers in `routes/shared.rs`
- Calls `pt_quote::compute_quote()` (pure, no I/O)
- Returns `Json<Quote>`
- Uses `TenantId` extractor (X-Tenant-Id header), `verify_project_tenant()` for auth

The scene route (`routes/scenes.rs`) extends this pattern with:
- `spawn_blocking` for CPU-bound work (triangulation)
- S3 upload via `crate::s3::upload_bytes()`
- Presigned URL via `crate::s3::presigned_get_url()`
- Returns `Json<SceneResponse>` with URL + metadata

## Proposal Pipeline Components

### pt-proposal crate exports

- **`ProposalNarrativeGenerator` trait** — `async fn generate(&self, input: &ProposalInput) -> Result<ProposalContent, ProposalError>`
- **`ProposalInput`** — `company_name`, `project_name`, `project_address`, `tiers: Vec<TierInput>`
- **`TierInput`** (BAML type) — `tier_level`, `total` (pre-formatted "$X,XXX.XX"), `zones: Vec<ZoneSummary>`
- **`ZoneSummary`** (BAML type) — `label`, `zone_type`, `area_sqft`, `materials: Vec<String>`
- **`ProposalContent`** (BAML type) — `intro_paragraph`, `tier_narratives`, `zone_callouts`, `closing_paragraph`
- **`render_proposal(data: &ProposalDocument) -> Result<Vec<u8>, ProposalError>`** — Typst → PDF
- **`ProposalDocument`** — `project_name`, `project_address`, `date`, `branding: TenantBranding`, `narrative: ProposalContent`, `good_quote`, `better_quote`, `best_quote`
- **`TenantBranding`** — `company_name`, `logo_url`, `primary_color`, `phone`, `email`
- **`MockProposalGenerator`** — deterministic narratives referencing input details

### AppState injection

`state.proposal_generator: Arc<dyn ProposalNarrativeGenerator>` already in AppState. Tests use `MockProposalGenerator`, production uses `BamlProposalGenerator`.

### Data sources

- **ProjectRow** — `client_name`, `address`, `tenant_id` (from `pt_repo::project::get_by_id`)
- **TenantRow** — `name`, `logo_url`, `brand_color`, `contact` (from `pt_repo::tenant::get_by_id`)
- **Zone/Material/TierAssignment** — same loading as quote route, but need all 3 tiers
- `pt_repo::tier_assignment::get_by_project_and_tier()` — need to call 3x (Good, Better, Best)

### Error handling

`ProposalError` already converts to `AppError` in `error.rs:77-89`:
- `InvalidInput` → 400
- `Generation` → 500
- `Render` → 500

`S3Error` → 500, `RepoError::NotFound` → 404 — all wired up.

### Quote computation

`pt_quote::compute_quote(zones, tier, materials, tax)` — pure function, returns `Quote` with `tier`, `line_items`, `subtotal`, `tax`, `total`. Need to call 3x with 3 different tiers.

### Scenario S.3.3

Currently `ScenarioOutcome::NotImplemented` at `quoting.rs:662`. Needs:
- Full API test: create project → zones → materials → tier assignments (3 tiers) → GET /proposal → verify PDF
- PDF starts with `%PDF-`
- PDF contains expected dollar totals as strings
- Uses MockProposalGenerator (already in api_helpers::router)

### Progress milestone

`"pt-pdf: branded quote PDF generation"` at `progress.rs:232-237` — `delivered_by: None`. Needs claiming.

## Key Constraint: All 3 Tiers Required

Unlike `/quote/{tier}` which returns one tier, the proposal needs all three quotes. If any tier has no assignments, that's likely a 400 error ("incomplete tier assignments").

## Response Format Decision

The ticket says "return bytes" with `Content-Type: application/pdf`. This is a direct PDF download, not a JSON response with a URL. The optional S3 caching is secondary.

## api_helpers limitation

`api_helpers::api_call()` returns `(StatusCode, Value)` — it parses the response body as JSON. For a PDF response, we need a raw bytes helper. The scenario test will need a new helper function that returns raw bytes instead of JSON.
