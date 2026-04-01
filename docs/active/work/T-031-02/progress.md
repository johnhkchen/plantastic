# T-031-02 Progress: Scene API + Viewer Wiring

## Completed

### Step 1: Extract shared type conversions
- Created `crates/plantastic-api/src/routes/shared.rs` with `zone_rows_to_zones()`, `material_rows_to_materials()`, `build_tier()`
- Updated `quotes.rs` to import from `shared` — removed private duplicates
- Added `pub mod shared;` and `pub mod scenes;` to `routes/mod.rs`

### Step 2: Add pt-scene dependency + error mapping
- Added `pt-scene = { path = "../pt-scene" }` to plantastic-api Cargo.toml
- Added `impl From<pt_scene::SceneError> for AppError` to error.rs
  - `MissingMaterial` → 400 Bad Request
  - `Triangulation` / `Export` → 500 Internal

### Step 3: Implement scene route
- Created `crates/plantastic-api/src/routes/scenes.rs`
- `GET /projects/{id}/scene/{tier}` route
- Loads zones, assignments, materials from DB
- Calls `generate_scene()` via `spawn_blocking` (CPU-bound)
- Uploads GLB to S3 at `scenes/{project_id}/{tier}.glb`
- Returns presigned URL (1 hour TTL) + metadata as JSON
- Registered in router via `.merge(scenes::routes())`

### Step 4: Update frontend viewer
- Updated `web/src/routes/(app)/project/[id]/viewer/+page.svelte`
- Replaced hardcoded `tierUrls` with API fetch via `apiFetch<SceneResponse>()`
- Added loading state (shows "Loading scene..." while fetching)
- Error handling via existing error banner
- Tier switching fetches new scene URL and calls `viewerRef.setTier()`

### Step 5: Update scenario S.2.4 + milestone
- Updated S.2.4 to call `pt_scene::generate_scene()` with real test data
- Verifies: valid GLB magic bytes, 2 zones, triangle count > 0
- Advanced to ThreeStar integration (from TwoStar)
- Added pt-scene dependency to tests/scenarios/Cargo.toml
- Claimed milestone in progress.rs with detailed note

### Step 6: Add smoke-scene justfile recipe
- Added `just smoke-scene` recipe (default: localhost:3000)
- Full pipeline: create project → add zone → create material → assign tier → fetch scene → verify GLB → cleanup

### Step 7: Quality gate
- `just fmt-check` — pass
- `just lint` (clippy strict) — pass
- `just test` — all 226 tests pass (0 failures, 16 ignored)
- `just scenarios` — 83.5 min / 240.0 min (34.8%), up from baseline

## Deviations from Plan

- Skipped S3 caching (was marked "optional" in ticket). Generate-on-demand is fast enough. Can add cache later if latency becomes an issue.
- Did not add `object_exists()` S3 helper since caching was skipped.
- Fixed `Fills` variant in scenario test — it uses `flush: bool` not `depth_inches: f64`.
