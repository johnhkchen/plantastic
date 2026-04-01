# T-031-02 Research: Scene API + Viewer Wiring

## What Exists

### pt-scene crate (T-031-01, done)
- `crates/pt-scene/src/scene.rs` — `generate_scene(zones, assignments, materials, tier) -> Result<SceneOutput, SceneError>`
- `SceneOutput` has `glb_bytes: Vec<u8>` and `metadata: SceneMetadata`
- `SceneMetadata` has `zone_count`, `triangle_count`, `tier`
- `SceneError` variants: `MissingMaterial`, `Triangulation`, `Export`
- 6 passing tests covering single/multi zone, empty, missing material, extrusion height, label fallback
- CPU-bound: triangulation (earcutr) + glTF binary export

### plantastic-api route patterns
- `crates/plantastic-api/src/routes/quotes.rs` — canonical pattern for tier-parameterized routes:
  1. Extract `TenantId` from header, `project_id` and `tier_str` from path
  2. `verify_project_tenant()` — 404 if not owned
  3. `parse_tier()` — converts "good"/"better"/"best" to `TierLevel`
  4. Load zones, assignments, materials from pt-repo
  5. Convert repo rows to domain types via `zone_rows_to_zones()`, `material_rows_to_materials()`, `build_tier()`
  6. Call pure computation, return JSON
- These conversion helpers are private to quotes.rs — need reuse

### AppState (state.rs)
- `pool: PgPool`, `s3_client: aws_sdk_s3::Client`, `s3_bucket: String`
- `scan_jobs: Arc<ScanJobTracker>`, `proposal_generator: Arc<dyn ProposalNarrativeGenerator>`
- S3 client and bucket already available

### S3 helpers (s3.rs)
- `upload_bytes(client, bucket, key, bytes, content_type)` — PutObject
- `presigned_get_url(client, bucket, key, expires_secs)` — presigned GET
- `download_bytes(client, bucket, key)` — GetObject
- `S3Error` type, already has `impl From<S3Error> for AppError`

### spawn_blocking pattern
- `scan.rs:229-239` — canonical: `tokio::task::spawn_blocking(move || { ... }).await.map_err()?`
- Used for CPU-bound scan processing and satellite baseline generation
- Error handling: `.map_err(|e| AppError::Internal(format!("task panicked: {e}")))?.map_err(AppError::from)?`

### Bevy viewer (apps/viewer/)
- `bridge.rs` — postMessage protocol: `loadScene(url)`, `setTier(tier, url)`, `setLightAngle(degrees)`
- `scene.rs` — handles `LoadSceneCommand` and `SetTierCommand`, loads glTF from URL via `asset_server.load()`
- Keep-until-ready: old scene stays visible while new one loads
- No changes needed to Bevy code — it already loads from any URL

### SvelteKit viewer page
- `web/src/routes/(app)/project/[id]/viewer/+page.svelte`
- Hardcoded `tierUrls` pointing to `/viewer/assets/models/test_scene.glb` for all tiers
- `switchTier(tier)` calls `viewerRef?.setTier(tier, tierUrls[tier])`
- Uses `apiFetch()` from `$lib/api` — sends `X-Tenant-Id` and `Authorization` headers
- No `+page.ts` data loader — tier URLs are inline constants

### API client (web/src/lib/api/)
- `apiFetch<T>(path, options)` — GET by default, adds auth headers, base URL is `/api`
- Route becomes `/api/projects/{id}/scene/{tier}` from frontend perspective

### Scenario S.2.4
- `tests/scenarios/src/suites/design.rs:386` — currently TwoStar (protocol verified)
- Comment says path to ThreeStar: "pt-scene generates real glTF from zones + materials"
- Needs to verify that `generate_scene()` produces valid glTF from known project data

### Progress milestone
- `tests/scenarios/src/progress.rs:153` — "pt-scene: 3D scene generation from project model"
- `delivered_by: None`, `unlocks: ["S.2.4", "S.4.1"]`

### Justfile
- `smoke url` recipe exists — calls `scripts/verify-deploy.sh`
- No `smoke-scene` recipe yet
- Pattern for new recipes is clear

## Key Constraints

1. **Type conversion helpers are private** — `zone_rows_to_zones()`, `material_rows_to_materials()`, `build_tier()` in quotes.rs are not `pub`. Scene route needs the same conversions.
2. **Scene generation is CPU-bound** — must use `spawn_blocking` to avoid blocking the Tokio runtime
3. **S3 caching** — ticket says "optional" cache. Key pattern: `scenes/{project_id}/{tier}/{version}.glb` where version comes from `updated_at`
4. **CORS on presigned URL** — Bevy viewer loads glTF via `asset_server.load(url)` which is a standard HTTP GET. S3 bucket already has CORS configured (T-017-02).
5. **Response format** — viewer expects a URL string, not the GLB bytes. Route should return JSON with `url` and `metadata`.
6. **No Bevy changes needed** — confirmed by reading bridge.rs and scene.rs

## Risks

- `#![allow(dead_code)]` on pt-scene lib.rs — the TODO says "Remove once scene.rs wires up mesh.rs and glb.rs". Since T-031-01 is done and tests pass, this should be safe but worth verifying the allow is still there.
- Presigned URL expiry: if viewer caches the URL and it expires, scene load will fail silently. Need reasonable TTL (e.g., 1 hour).
