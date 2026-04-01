# T-031-02 Structure: Scene API + Viewer Wiring

## Files Modified

### `crates/plantastic-api/Cargo.toml`
- Add `pt-scene = { path = "../pt-scene" }` dependency

### `crates/plantastic-api/src/routes/mod.rs`
- Add `pub mod scenes;`
- Add `.merge(scenes::routes())` to router

### `crates/plantastic-api/src/routes/shared.rs` (new)
- Extract from quotes.rs: `zone_rows_to_zones()`, `material_rows_to_materials()`, `build_tier()`
- All `pub(crate)`

### `crates/plantastic-api/src/routes/quotes.rs`
- Remove private conversion functions
- Import from `super::shared`

### `crates/plantastic-api/src/routes/scenes.rs` (new)
- `pub fn routes() -> Router<AppState>` — single route `GET /projects/{id}/scene/{tier}`
- `async fn get_scene(...)` — handler
- `SceneResponse` struct — `{ url: String, metadata: SceneMetadata }`
- `scene_s3_key()` — builds S3 key from project_id, tier, version
- `scene_exists()` — HeadObject check
- Uses `spawn_blocking` for CPU-bound generation

### `crates/plantastic-api/src/error.rs`
- Add `impl From<pt_scene::SceneError> for AppError`

### `crates/plantastic-api/src/s3.rs`
- Add `object_exists(client, bucket, key) -> Result<bool, S3Error>` — HeadObject wrapper

### `web/src/routes/(app)/project/[id]/viewer/+page.svelte`
- Remove hardcoded `tierUrls`
- Add `page` import from `$app/stores` for project ID (or get from layout)
- On mount: call `apiFetch<SceneResponse>(`/projects/${id}/scene/${activeTier}`)` → pass URL to Viewer
- On tier switch: fetch new tier's scene URL, call `setTier(tier, url)`
- Add `loading` state, show spinner or disable tier buttons during fetch
- Error handling: catch fetch errors, show in error banner

### `web/src/routes/(app)/project/[id]/viewer/+page.ts` (new)
- Load function: extract `params.id` → return `{ projectId: params.id }`

### `tests/scenarios/src/suites/design.rs`
- Update `s_2_4_3d_preview()` to call `pt_scene::generate_scene()` with test data
- Verify: valid glTF magic bytes, zone_count matches, triangle_count > 0
- Advance to ThreeStar integration

### `tests/scenarios/src/progress.rs`
- Update "pt-scene: 3D scene generation" milestone: `delivered_by: Some("T-031-02")`
- Add note describing API route, S3 caching, viewer wiring

### `tests/scenarios/Cargo.toml`
- Add `pt-scene`, `pt-project`, `pt-materials`, `pt-geo` dependencies for S.2.4 test

### `justfile`
- Add `smoke-scene` recipe

## Files NOT Modified

- `apps/viewer/` — no Bevy changes needed, viewer already handles loadScene/setTier
- `web/src/lib/components/viewer/Viewer.svelte` — no changes, already accepts sceneUrl prop
- `web/src/lib/components/viewer/types.ts` — no changes, protocol types already correct

## Module Boundaries

```
plantastic-api
  routes/
    mod.rs          ← adds scenes
    shared.rs       ← extracted conversions (used by quotes + scenes)
    quotes.rs       ← imports shared
    scenes.rs       ← new route, imports shared + pt-scene
  error.rs          ← SceneError mapping
  s3.rs             ← object_exists helper
```

## Ordering

1. Extract shared conversions (quotes.rs → shared.rs) — refactor, no behavior change
2. Add pt-scene dependency + SceneError mapping + S3 helper
3. Implement scenes.rs route
4. Update frontend viewer page
5. Update scenario + milestone
6. Add justfile recipe
