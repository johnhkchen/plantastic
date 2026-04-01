# T-031-02 Plan: Scene API + Viewer Wiring

## Step 1: Extract shared type conversions

- Create `crates/plantastic-api/src/routes/shared.rs`
- Move `zone_rows_to_zones()`, `material_rows_to_materials()`, `build_tier()` from quotes.rs
- Make them `pub(crate)`
- Update quotes.rs to import from `super::shared`
- Add `pub mod shared;` to routes/mod.rs
- **Verify**: `just test-crate plantastic-api` — no behavior change

## Step 2: Add pt-scene dependency + error mapping + S3 helper

- Add `pt-scene = { path = "../pt-scene" }` to plantastic-api/Cargo.toml
- Add `impl From<pt_scene::SceneError> for AppError` to error.rs
- Add `object_exists()` to s3.rs
- **Verify**: `cargo check -p plantastic-api`

## Step 3: Implement scene route

- Create `crates/plantastic-api/src/routes/scenes.rs`
- Define `SceneResponse { url, metadata }` with Serialize
- Implement `get_scene` handler:
  1. Verify tenant, parse tier
  2. Load project row (for `updated_at` version key)
  3. Build S3 key: `scenes/{project_id}/{tier}/{epoch}.glb`
  4. Check if cached via `object_exists()`
  5. If not cached: load zones/assignments/materials, convert types, `spawn_blocking(generate_scene)`, upload GLB
  6. Generate presigned URL (1 hour TTL)
  7. Return `SceneResponse`
- Register in routes/mod.rs
- **Verify**: `cargo check -p plantastic-api`

## Step 4: Add unit test for scene route

- Test: scene response contains valid URL and metadata fields
- Test: SceneError maps to correct HTTP status codes
- **Verify**: `just test-crate plantastic-api`

## Step 5: Update frontend viewer page

- Create `web/src/routes/(app)/project/[id]/viewer/+page.ts` — exports `projectId` from params
- Update `+page.svelte`:
  - Replace `tierUrls` with API fetch on mount and tier switch
  - Add `loading` state
  - Handle errors via error banner
- **Verify**: manual check (frontend changes)

## Step 6: Update scenario S.2.4 + milestone

- Add pt-scene + dependencies to tests/scenarios/Cargo.toml
- Update `s_2_4_3d_preview()` to call `generate_scene()` with test zone + material data
- Verify valid GLB output (magic bytes, zone count, triangles)
- Advance to ThreeStar integration
- Update milestone in progress.rs: set `delivered_by`, write note
- **Verify**: `just scenarios`

## Step 7: Add smoke-scene justfile recipe

- Add recipe that documents the manual integration check
- **Verify**: recipe appears in `just --list`

## Step 8: Final gate

- `just check` — format + lint + test + scenarios
- Run scenario dashboard before and after comparison

## Testing Strategy

- **Unit tests**: SceneError→AppError mapping, S3 key generation
- **Scenario test**: S.2.4 calls real `generate_scene()` with test data, verifies valid GLB
- **Integration test** (ignored, requires Postgres+S3): full route test — would be a future ticket
- **Smoke test**: manual `just smoke-scene` recipe for deployed environment
