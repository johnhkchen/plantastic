# T-031-02 Design: Scene API + Viewer Wiring

## Decision: API Route Design

### Option A: Generate-on-demand, upload to S3, return presigned URL
- Route: `GET /projects/{id}/scene/{tier}`
- Flow: load data → `spawn_blocking(generate_scene)` → upload GLB to S3 → return presigned GET URL
- Pros: Simple, stateless, matches scan upload pattern
- Cons: Slow on first call (~100-500ms for generation + S3 upload), no caching

### Option B: Same as A but with S3 cache check
- Flow: check S3 for `scenes/{project_id}/{tier}/{version}.glb` → if exists, return presigned URL → else generate + upload
- Version key from `project.updated_at` timestamp (zones/assignments change → project updated_at changes)
- Pros: Fast on subsequent calls, simple cache invalidation
- Cons: Slightly more complex, stale cache risk if updated_at isn't bumped on zone/assignment changes

### Option C: Pre-generate on zone/assignment save, serve from S3
- Pros: Fastest viewer load
- Cons: Wasteful (generates on every edit), complex invalidation, over-engineering for MVP

**Decision: Option B** — S3 cache with version key. The cache check is a single HeadObject call (~5ms), and generation is fast enough that cache misses are acceptable. Version key from `updated_at` is simple and correct enough for now.

## Response Schema

```json
{
  "url": "https://s3.us-west-2.amazonaws.com/...",
  "metadata": {
    "zone_count": 3,
    "triangle_count": 1248,
    "tier": "good"
  }
}
```

The viewer page fetches this, extracts `url`, and passes it to `loadScene`/`setTier`.

## Shared Type Conversions

The `zone_rows_to_zones()`, `material_rows_to_materials()`, and `build_tier()` functions in quotes.rs are duplicated work. Options:

1. **Extract to a shared module** — `routes/shared.rs` or `routes/conversions.rs`
2. **Make them pub in quotes.rs** — works but awkward cross-module coupling
3. **Duplicate in scenes.rs** — simple but DRY violation

**Decision: Option 1** — Extract to `routes/shared.rs`. Both quotes.rs and scenes.rs import from there. Clean, low-risk refactor.

## S3 Key Schema

```
scenes/{project_id}/{tier}/{version}.glb
```

Where `version` is `updated_at` formatted as epoch seconds (e.g., `1711929600`). This gives us:
- Natural cache invalidation when project data changes
- Easy cleanup (delete `scenes/{project_id}/` prefix when project deleted)
- No collision between tiers

## Cache Check Strategy

Use `head_object` to check if the key exists. If it does, skip generation and return presigned URL directly. If not, generate, upload, return URL.

Considered: storing version in a DB column. Rejected — adds migration complexity for a simple cache. S3 HeadObject is cheap.

## Frontend Changes

Replace hardcoded `tierUrls` with API calls:
- On mount: fetch `GET /api/projects/{id}/scene/good` → extract URL → pass to Viewer
- On tier switch: fetch `GET /api/projects/{id}/scene/{tier}` → extract URL → pass to `setTier`
- Add loading state during fetch
- Error handling: if scene generation fails, show error banner

Get project ID from the route params via `+layout.ts` (already extracts `params.id`).

## Scenario Update

S.2.4 advances to ThreeStar by verifying `generate_scene()` produces valid glTF from known project data (zones + materials → GLB with correct node names and triangle count > 0).

## Smoke Test

`just smoke-scene` recipe: lightweight integration check that the scene pipeline works. Calls the API with a test project, verifies GLB response. Requires running API + database, so it's a manual dev check, not part of `just check`.

## SceneError → AppError Mapping

- `MissingMaterial` → 400 Bad Request (client's fault: assignment references nonexistent material)
- `Triangulation` → 500 Internal (shouldn't happen with valid geometry)
- `Export` → 500 Internal
