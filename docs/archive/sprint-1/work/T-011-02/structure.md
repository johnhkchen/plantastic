# T-011-02 Structure — Prepopulation API

## Files Modified

### 1. `crates/pt-repo/src/project.rs`
- Add `pub async fn set_baseline(pool, id, baseline_json) -> Result<(), RepoError>`
  - SQL: `UPDATE projects SET baseline = $1, updated_at = now() WHERE id = $2`
  - Returns `RepoError::NotFound` if 0 rows affected.

### 2. `crates/plantastic-api/Cargo.toml`
- Add dependency: `pt-satellite = { path = "../pt-satellite" }`

### 3. `crates/plantastic-api/src/error.rs`
- Add `From<pt_satellite::SatelliteError> for AppError`:
  - `AddressNotFound(_)` → `AppError::BadRequest`
  - `NoParcelData{..}` → `AppError::BadRequest`
  - `CanopyUnavailable` → `AppError::Internal`
  (Note: these conversions exist for logging/mapping but won't be used to fail requests.
  The handler catches satellite errors and proceeds without baseline.)

### 4. `crates/plantastic-api/src/routes/projects.rs`
- Add `baseline: Option<serde_json::Value>` to `ProjectResponse`.
- Update `From<ProjectRow> for ProjectResponse` to include `r.baseline`.
- Modify `create_project` handler:
  - After creating the project, if `body.address` is `Some`:
    - Clone address, `spawn_blocking` → construct `BaselineBuilder<EmbeddedSource>`,
      call `build(address)`.
    - On `Ok(baseline)`: serialize to `serde_json::Value`, call `set_baseline()`.
    - On `Err(e)`: log warn, continue without baseline.
  - Re-fetch project row (already does this) so response includes baseline.

### 5. `web/src/lib/stores/project.svelte.ts`
- Add to `Project` interface: `baseline: ProjectBaseline | null`
- Add `ProjectBaseline` interface with nested types:
  ```ts
  interface ProjectBaseline {
    coordinates: { latitude: number; longitude: number };
    lot_boundary: { polygon: GeoJsonGeometry; area_sqft: number; source: string };
    trees: Array<{ location: ...; height_ft: number; spread_ft: number; confidence: number }>;
    sun_grid: { width: number; height: number; values: number[]; ... };
  }
  ```

### 6. `web/src/routes/(app)/project/[id]/+page.svelte`
- Add "Site Baseline" section below the existing stats grid.
- Conditionally renders when `project.baseline` is present:
  - Lot: area in sqft, polygon vertex count
  - Trees: count + summary table (height, spread, confidence per tree)
  - Sun grid: dimensions (width × height), cell count

### 7. `tests/scenarios/src/suites/site_assessment.rs`
- Upgrade `s_1_2_satellite_prepopulation()`:
  - After building baseline, add JSON serialization round-trip:
    `serde_json::to_value(&baseline)` → `serde_json::from_value::<ProjectBaseline>()`
  - Verify deserialized baseline matches original (coordinates, lot area, tree count,
    grid dimensions).
  - Return `ScenarioOutcome::Pass(Integration::TwoStar)`.

### 8. `tests/scenarios/Cargo.toml`
- Add dependency: `serde_json = { workspace = true }` (if not already present).

### 9. `tests/scenarios/src/progress.rs`
- Update the pt-satellite milestone note to document T-011-02 additions.

## Files NOT Modified

- `crates/pt-satellite/` — no changes needed. The crate already produces what we need.
- `crates/pt-repo/Cargo.toml` — already has serde_json.
- `crates/plantastic-api/src/state.rs` — AppState unchanged (builder constructed inline).
- `crates/plantastic-api/src/main.rs` — no changes.
- Migrations — baseline column already exists.

## Module Boundaries

- `pt-satellite` remains a library crate with no API or database awareness.
- `plantastic-api` gains a dependency on `pt-satellite` and uses it in the handler layer.
- `pt-repo` gains a single function — thin data access, no domain logic.
- Frontend changes are display-only — no new API calls, just rendering data already
  returned by the existing GET endpoint.

## Public Interface Changes

### pt-repo::project
```rust
// New function
pub async fn set_baseline(pool: &PgPool, id: Uuid, baseline: &serde_json::Value) -> Result<(), RepoError>
```

### plantastic-api ProjectResponse
```rust
// Added field
baseline: Option<serde_json::Value>,
```

No breaking changes to existing callers. The baseline field is additive.
