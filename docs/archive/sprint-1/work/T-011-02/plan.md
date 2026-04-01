# T-011-02 Plan — Prepopulation API

## Step 1: pt-repo — add set_baseline function

**Files**: `crates/pt-repo/src/project.rs`

Add `set_baseline(pool, id, baseline_json)`:
- SQL: `UPDATE projects SET baseline = $1, updated_at = now() WHERE id = $2`
- Check rows_affected, return `RepoError::NotFound` if 0.

**Verification**: Compiles. No standalone unit test needed — tested via integration.

## Step 2: plantastic-api — add pt-satellite dependency and error mapping

**Files**: `crates/plantastic-api/Cargo.toml`, `crates/plantastic-api/src/error.rs`

- Add `pt-satellite = { path = "../pt-satellite" }` to Cargo.toml.
- Add `From<pt_satellite::SatelliteError> for AppError`.

**Verification**: `cargo check -p plantastic-api` compiles.

## Step 3: plantastic-api — extend ProjectResponse with baseline

**Files**: `crates/plantastic-api/src/routes/projects.rs`

- Add `baseline: Option<serde_json::Value>` to `ProjectResponse`.
- Update the `From<ProjectRow>` impl to include `baseline`.

**Verification**: `cargo check -p plantastic-api`.

## Step 4: plantastic-api — wire BaselineBuilder into create_project handler

**Files**: `crates/plantastic-api/src/routes/projects.rs`

Modify `create_project`:
1. After `pt_repo::project::create()` succeeds and we have the project `id`:
2. If `body.address` is `Some(ref addr)`:
   a. Clone the address string.
   b. `tokio::task::spawn_blocking(move || { ... })` — inside:
      - Construct `BaselineBuilder::new(EmbeddedSource, EmbeddedSource, EmbeddedSource)`
      - Call `builder.build(&address)`
   c. `.await` the join handle.
   d. On `Ok(Ok(baseline))`:
      - `serde_json::to_value(&baseline)` — should not fail.
      - `pt_repo::project::set_baseline(&state.pool, id, &json_val).await`.
   e. On `Ok(Err(satellite_err))`: `tracing::warn!("baseline generation failed: {satellite_err}")`, continue.
   f. On `Err(join_err)`: `tracing::error!("baseline task panicked: {join_err}")`, continue.
3. Re-fetch via `get_by_id` (already done) — now includes baseline.

**Verification**: `cargo check -p plantastic-api`. Manual test with known address confirms
baseline present in response.

## Step 5: Scenario S.1.2 — upgrade to TwoStar

**Files**: `tests/scenarios/src/suites/site_assessment.rs`, `tests/scenarios/Cargo.toml`

- Ensure `serde_json` dep exists in scenarios Cargo.toml.
- After existing baseline validation, add:
  - `serde_json::to_value(&baseline)` → `from_value::<ProjectBaseline>()` round-trip.
  - Verify deserialized fields match (coordinates, lot area, tree count, grid dims).
- Change return to `ScenarioOutcome::Pass(Integration::TwoStar)`.

**Verification**: `cargo run -p pt-scenarios` — S.1.2 shows TwoStar.

## Step 6: Update milestone in progress.rs

**Files**: `tests/scenarios/src/progress.rs`

Update the pt-satellite milestone note to include T-011-02's contribution.

**Verification**: `cargo run -p pt-scenarios` — milestone note visible.

## Step 7: Frontend — extend Project type and display baseline

**Files**: `web/src/lib/stores/project.svelte.ts`,
          `web/src/routes/(app)/project/[id]/+page.svelte`

- Add `baseline` field to `Project` interface (nullable JSON object).
- Add baseline display section to project page.

**Verification**: Visual inspection. No automated frontend tests yet.

## Step 8: Quality gate

Run `just check` (fmt + lint + test + scenarios).

- Expect: all existing tests pass, S.1.2 at TwoStar, no regressions.
- Fix any clippy/fmt issues.

## Testing Strategy

| What | How | Where |
|------|-----|-------|
| set_baseline SQL | Integration test via API round-trip | plantastic-api tests |
| Baseline in POST response | Existing ignored integration tests + scenario | Both |
| Baseline in GET response | Existing ignored integration tests + scenario | Both |
| JSON round-trip | Scenario S.1.2 serialization check | pt-scenarios |
| Unknown address fallback | Unit check: verify project created without baseline | plantastic-api |
| Frontend baseline display | Manual visual check | Browser |

The key automated gate: S.1.2 passing at TwoStar proves the JSON serialization path
works. The integration tests (currently `#[ignore]` requiring Postgres) prove the full
API round-trip works when Postgres is available.
