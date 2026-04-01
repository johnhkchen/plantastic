//! Engineering milestones that unlock scenario value.
//!
//! Each milestone represents a piece of foundational work. When an agent
//! completes a ticket that delivers a milestone, they update `delivered_by`
//! from `None` to `Some("T-XXX-XX")` and add a `note` describing what
//! was delivered and what it enables.
//!
//! The dashboard shows milestone completion alongside scenario status,
//! so foundational work is visible even when no scenario flips to green.

/// A delivered piece of engineering that unblocks one or more scenarios.
pub struct Milestone {
    /// Short label (e.g., "pt-geo: area, perimeter, volume").
    pub label: &'static str,
    /// Which ticket delivered this, or None if not yet delivered.
    pub delivered_by: Option<&'static str>,
    /// Scenario IDs this milestone contributes to.
    pub unlocks: &'static [&'static str],
    /// Agent's note on what was delivered and what it enables.
    /// This is where agents document the engineering value of their work.
    pub note: &'static str,
}

/// All milestones for the project.
///
/// When you complete a ticket that delivers foundational capability:
/// 1. Find or add the relevant milestone(s) below
/// 2. Set `delivered_by` to your ticket ID
/// 3. Write a `note` explaining what you delivered and what it unblocks
/// 4. The dashboard will show your contribution toward scenario value
///
/// Guidelines for notes:
/// - Be specific: "area_sqft() computes polygon area in sq ft using the
///   geo crate's unsigned_area()" not "implemented area function"
/// - Name what's unblocked: "S.3.1 can now compute patio area for quotes"
/// - Flag what's still needed: "Still needs pt-materials for unit pricing"
pub static MILESTONES: &[Milestone] = &[
    // ── Site Assessment ─────────────────────────────────────────
    Milestone {
        label: "pt-scan: PLY parsing + mesh generation",
        delivered_by: Some("T-015-02"),
        unlocks: &["S.1.1"],
        note: "T-015-01 delivered PLY parsing (binary LE/BE + ASCII), voxel downsampling, \
               k-NN outlier removal, and RANSAC ground plane fitting via process_scan(). \
               T-015-02 adds mesh generation and export: triangulate() uses delaunator for \
               2D Delaunay triangulation of ground points (XY projection, original 3D positions). \
               decimate() uses meshopt::simplify_decoder for QEM-based mesh decimation to \
               configurable target (default 50K triangles). to_glb() produces binary glTF 2.0 \
               with POSITION, NORMAL, COLOR_0 accessors — loads in Bevy viewer (T-013-01). \
               to_plan_view_png() renders top-down orthographic projection with elevation \
               shading and optional canopy overlay via the image crate. generate_terrain() \
               orchestrates: triangulate → decimate → glTF + PNG + TerrainMetadata. \
               31 tests (25 unit + 6 integration). S.1.1 passes at OneStar with full \
               scan-to-artifact pipeline verified. Path to TwoStar: wire into scan upload \
               API (T-016-01) so artifacts are produced server-side on upload.",
    },
    Milestone {
        label: "Scan upload API: PLY → S3 → process → artifacts",
        delivered_by: Some("T-016-01"),
        unlocks: &["S.1.1"],
        note: "POST /projects/{id}/scan accepts multipart PLY upload, stores raw PLY in S3, \
               spawns async processing via tokio::spawn + spawn_blocking. process_scan() + \
               generate_terrain() run in background, upload terrain.glb, planview.png, \
               metadata.json to S3, then update project.scan_ref JSONB with all S3 keys. \
               GET /projects/{id}/scan/status polls job state (pending/processing/complete/failed). \
               GET /projects/{id}/planview redirects to presigned S3 URL for plan view PNG. \
               In-memory ScanJobTracker (DashMap) tracks job state; durable state in scan_ref. \
               S3 helper module (s3.rs) wraps aws-sdk-s3 for upload/download/presign. \
               scan_ref added to ProjectResponse DTO. set_scan_ref() added to pt-repo. \
               5 unit tests (scan_job lifecycle + S3 key format), 6 integration tests (#[ignore]). \
               S.1.1 path to TwoStar: run integration tests with Postgres + S3/LocalStack.",
    },
    Milestone {
        label: "pt-satellite: address → lot + canopy + sun baseline",
        delivered_by: Some("T-011-01"),
        unlocks: &["S.1.2"],
        note: "pt-satellite crate: trait-based data source abstraction (Geocoder, ParcelSource, \
               CanopySource) with EmbeddedSource for known SF test addresses. \
               BaselineBuilder orchestrates: address → geocode → lot boundary → tree detection → \
               pt-solar radiance grid → ProjectBaseline struct. Types: ProjectBaseline (coordinates, \
               lot_boundary, trees, sun_grid), LotBoundary (polygon + area_sqft + source), \
               DetectedTree (location, height_ft, spread_ft, confidence). \
               T-011-02 wired BaselineBuilder into POST /projects: spawn_blocking invocation, \
               baseline stored as JSONB via set_baseline(), returned on GET. Graceful fallback — \
               unknown addresses log a warning, project created without baseline. \
               S.1.2 upgraded to TwoStar (JSON serialization round-trip verified). \
               Path to real data: implement Geocoder/ParcelSource/CanopySource against \
               geocoding API, SF parcel data, and Meta canopy height dataset.",
    },
    Milestone {
        label: "pt-solar: sun position + radiance grid",
        delivered_by: Some("T-010-01"),
        unlocks: &["S.1.3", "S.2.3"],
        note: "pt-solar crate: sun_position() (NOAA algorithm, altitude/azimuth for any lat/lng/datetime), \
               daily_sun_hours() (5-min sampling, 288 samples/day, polar condition detection), \
               annual_sun_hours() (date range aggregation with min/max/average), \
               classify() (full-sun/part-sun/part-shade/full-shade from hours), \
               radiance_grid() (spatial grid over LatLngBounds with configurable resolution and sample days). \
               S.1.3 now passes at OneStar. S.2.3 (plant recommendations) still needs pt-plants + BAML AI layer.",
    },
    Milestone {
        label: "pt-climate: frost dates, hardiness zones, Sunset zones, growing season",
        delivered_by: Some("T-010-02"),
        unlocks: &["S.2.3"],
        note: "pt-climate crate: frost_dates() (latitude-band lookup with elevation/coastal modifiers, \
               northern + southern hemisphere tables, DayOfYearRange with variance), \
               hardiness_zone() (min winter temp estimation → USDA zones 1a-13b, elevation lapse rate), \
               sunset_zone() (Bay Area bounding-box lookup for Sunset zones 14-17), \
               growing_season() (frost-free period computation with typical/short/long estimates), \
               climate_profile() (convenience aggregator). 23 unit tests, all pure computation. \
               S.2.3 (plant recommendations) still needs pt-plants + BAML AI layer.",
    },
    Milestone {
        label: "pt-plants: database + Plant.id integration",
        delivered_by: None,
        unlocks: &["S.1.4", "S.2.3"],
        note: "",
    },
    // ── Design ──────────────────────────────────────────────────
    Milestone {
        label: "Zone editor: polygon drawing on plan view",
        delivered_by: Some("T-007-02"),
        unlocks: &["S.2.1"],
        note: "T-007-01 delivered the Svelte 5 canvas polygon drawing component (ZoneEditor.svelte) \
               with click-to-place vertices, zone type selector, label input, vertex drag editing, \
               and multi-zone support. T-007-02 wires it to the API: debounced auto-save via \
               PUT /projects/:id/zones, zone reload on page load, and a measurements panel \
               displaying area_sqft and perimeter_ft computed by pt-geo on the server side. \
               S.2.1 now passes at TwoStar. Path to ThreeStar: polish the measurements UI, \
               add undo/redo, touch event support for iPad.",
    },
    Milestone {
        label: "pt-materials: catalog model + tenant layering",
        delivered_by: Some("T-012-01"),
        unlocks: &["S.2.2", "S.3.1", "S.3.2"],
        note: "pt-materials crate: Material struct (MaterialId, name, category, unit, \
               price_per_unit as Decimal, depth_inches, texture_ref, photo_ref, supplier_sku, \
               extrusion), MaterialCategory enum (Hardscape/Softscape/Edging/Fill), Unit enum \
               (SqFt/CuYd/LinearFt/Each), ExtrusionBehavior tagged enum (SitsOnTop/Fills/BuildsUp), \
               MaterialBuilder for ergonomic construction. All types serde round-trip with snake_case \
               JSON matching the API contract. Catalog CRUD page at /catalog (SvelteKit) lists, creates, \
               edits, deletes materials via GET/POST/PATCH/DELETE /materials API routes. \
               S.2.2 passes at OneStar (domain model verified). S.3.1/S.3.2 unblocked for quote \
               computation using material pricing. Path to TwoStar: T-012-02 adds search/filter.",
    },
    Milestone {
        label: "BAML AI layer: plant recommendations",
        delivered_by: None,
        unlocks: &["S.2.3"],
        note: "",
    },
    Milestone {
        label: "pt-scene: 3D scene generation from project model",
        delivered_by: Some("T-031-02"),
        unlocks: &["S.2.4", "S.4.1"],
        note: "T-031-01 delivered pt-scene crate: generate_scene(zones, assignments, materials, tier) \
               → SceneOutput with glb_bytes (binary glTF 2.0) and SceneMetadata (zone_count, \
               triangle_count, tier). T-031-02 wires it into the API and viewer: \
               GET /projects/{id}/scene/{tier} route loads project data, runs generate_scene via \
               spawn_blocking, uploads GLB to S3 (scenes/{project_id}/{tier}.glb), returns presigned \
               URL + metadata. Viewer page fetches scene URL on load and tier switch, passes to \
               Bevy iframe via loadScene/setTier postMessage. Shared type conversion helpers \
               (zone_rows_to_zones, material_rows_to_materials, build_tier) extracted to shared.rs \
               for reuse across quote and scene routes. S.2.4 advances to ThreeStar. \
               S.4.1 still needs crew export format integration.",
    },
    // ── Quoting ─────────────────────────────────────────────────
    Milestone {
        label: "pt-geo: area, perimeter, volume computation",
        delivered_by: Some("T-001-02"),
        unlocks: &["S.2.1", "S.3.1", "S.3.2"],
        note: "pt-geo crate: area_sqft() (unsigned polygon area via geo crate), \
               perimeter_ft() (exterior ring Euclidean length), volume_cuft/volume_cuyd \
               (area × depth conversions), boolean ops (union/difference), \
               simplify (Ramer-Douglas-Peucker). 21 unit tests. Pure functions, no I/O. \
               Used by pt-quote for S.3.1/S.3.2 quantity computation, and by \
               plantastic-api zone routes for S.2.1 live measurements.",
    },
    Milestone {
        label: "pt-project: Project/Zone/Tier model + GeoJSON serde",
        delivered_by: Some("T-002-01, T-003-02"),
        unlocks: &["S.3.1", "S.3.2", "S.3.4", "S.INFRA.1"],
        note: "T-002-01 delivered domain types: Project (ProjectId, client_name, status, \
               tenant_id), Zone (ZoneId, geometry as geo::Polygon, ZoneType enum, label), \
               Tier (TierLevel::Good/Better/Best, Vec<MaterialAssignment>), \
               MaterialAssignment (zone_id, material_id, overrides). T-003-02 adds the \
               repository layer: ProjectRepo, ZoneRepo with GeoJSON serde via \
               ST_GeomFromGeoJSON/ST_AsGeoJSON for PostGIS round-trip. Full CRUD lifecycle \
               verified end-to-end by S.INFRA.1.",
    },
    Milestone {
        label: "pt-quote: quantity takeoff engine",
        delivered_by: Some("T-002-02"),
        unlocks: &["S.3.1", "S.3.2"],
        note: "T-002-02 delivered pt-quote crate: compute_quote() takes zones, tier \
               assignments, and material catalog; computes quantity per zone (area_sqft \
               for SqFt materials, volume_cuyd for CuYd, perimeter_ft for LinearFt via \
               pt-geo); multiplies by unit price; returns Quote with line items, subtotal, \
               and total. Verified through API at GET /projects/:id/quote/:tier — \
               S.INFRA.1 confirms 12×15 patio at $8.50/sqft = $1,530.00 full-stack.",
    },
    Milestone {
        label: "Quote API route: GET /projects/:id/quote/:tier",
        delivered_by: Some("T-008-01"),
        unlocks: &["S.3.1", "S.3.2"],
        note: "GET /projects/{id}/quote/{tier} route in plantastic-api. \
               Loads zones from PostGIS, tier assignments, and tenant material catalog; \
               converts repo types to domain types; calls pt_quote::compute_quote(); \
               returns JSON Quote with line items, subtotal, total. \
               Handles 404 (missing project), 400 (invalid tier, bad catalog data), \
               and empty quotes (no assignments → $0 total). \
               S.3.1/S.3.2 backend prereq is met — API exists for quote computation. \
               Path to TwoStar: update scenario tests to exercise the API path.",
    },
    Milestone {
        label: "pt-proposal: trait abstraction + mock generator",
        delivered_by: Some("T-029-02"),
        unlocks: &["S.3.3"],
        note: "ProposalNarrativeGenerator trait in pt-proposal with async generate() method. \
               BamlProposalGenerator calls BAML GenerateProposalNarrative for real LLM usage. \
               MockProposalGenerator returns deterministic, realistic narratives referencing \
               input zone labels, tier levels, company name, and project address — suitable \
               for screenshot tests. MockFailingGenerator for error path testing. \
               ClaudeCliGenerator routes through local `claude` CLI (zero API cost). \
               ProposalInput bundles function params; ProposalError covers Generation + InvalidInput. \
               AppState in plantastic-api accepts Arc<dyn ProposalNarrativeGenerator> for DI. \
               All test helpers (common/mod.rs, api_helpers.rs) use MockProposalGenerator. \
               6 unit tests verify mock behavior, determinism, error paths, and empty input. \
               Unblocks proposal narrative route (API handler can call state.proposal_generator) \
               and proposal PDF generation (S.3.3) once pt-pdf is ready.",
    },
    Milestone {
        label: "pt-pdf: branded quote PDF generation",
        delivered_by: Some("T-030-02"),
        unlocks: &["S.3.3"],
        note: "GET /projects/{id}/proposal route in plantastic-api. Loads project, tenant, \
               zones, materials, and all 3 tier assignments; computes 3 quotes via pt-quote; \
               builds ProposalInput with TierInput/ZoneSummary for narrative generation; \
               calls state.proposal_generator.generate() (MockProposalGenerator in tests); \
               assembles ProposalDocument with TenantBranding; renders PDF via \
               pt_proposal::render_proposal() in spawn_blocking; returns raw PDF bytes with \
               Content-Type: application/pdf and Content-Disposition header. \
               S.3.3 scenario test verifies full pipeline: create project → zones → materials → \
               3 tier assignments → GET /proposal → assert %PDF- magic bytes + dollar totals \
               in rendered content. Uses MockProposalGenerator — zero LLM calls.",
    },
    // ── Crew Handoff ────────────────────────────────────────────
    Milestone {
        label: "Bevy viewer: glTF loading + orbit + tap-to-inspect",
        delivered_by: Some("T-013-02"),
        unlocks: &["S.4.1", "S.4.3"],
        note: "T-013-01 proved the Bevy WASM pipeline (10 MB binary, WebGL2, 30+ FPS). \
               T-013-02 adds the iframe bridge and interaction: BridgePlugin in bridge.rs \
               (postMessage ↔ Bevy messages via thread-local queue + web_sys listener), \
               PickingSetupPlugin in picking.rs (Pointer<Click> → entity Name → \
               sceneTapped postMessage), dynamic scene loading from URL via LoadSceneCommand, \
               directional light angle control via SetLightAngleCommand, Viewer.svelte \
               (iframe wrapper with typed postMessage protocol), types.ts (ViewerInboundMessage / \
               ViewerOutboundMessage), and viewer page at /project/[id]/viewer. \
               Protocol: Host→Viewer (loadScene, setTier, setLightAngle), \
               Viewer→Host (ready, error, sceneTapped). S.2.4 passes at TwoStar. \
               S.4.1 and S.4.3 still need pt-scene for real zone→glTF generation.",
    },
    Milestone {
        label: "pt-dxf: DXF export with layers and annotations",
        delivered_by: None,
        unlocks: &["S.4.2"],
        note: "",
    },
    // ── Infrastructure ──────────────────────────────────────────
    Milestone {
        label: "Axum API: routes + Lambda deployment",
        delivered_by: Some("T-004-02"),
        unlocks: &["S.INFRA.1", "S.INFRA.2", "S.3.4"],
        note: "T-004-01 delivered skeleton (health, middleware, Lambda dual-mode). \
               T-004-02 adds full CRUD routes: Project (POST/GET/GET:id/DELETE), \
               Zone (GET/POST/PUT-bulk/PATCH/DELETE with GeoJSON geometry), \
               Material (GET/POST/PATCH/DELETE), Tier (GET-all-3/PUT assignments). \
               X-Tenant-Id header extractor enforces tenant scoping on every route. \
               lib.rs exports router() + AppState for integration tests. \
               Integration tests (8 test fns, #[ignore] requiring Postgres) cover \
               CRUD lifecycle, tenant isolation, bulk zone ops, tier assignments, \
               and validation. S.INFRA.1 and S.INFRA.2 backend prereqs are met.",
    },
    Milestone {
        label: "PostGIS schema + sqlx repository layer",
        delivered_by: Some("T-003-02"),
        unlocks: &["S.INFRA.1", "S.INFRA.2"],
        note: "pt-repo crate: create_pool() (Lambda-tuned PgPool), RepoError enum, \
               ProjectRepo (create/get_by_id/list_by_tenant/update_status/delete), \
               ZoneRepo (add/list_by_project/update/delete/bulk_upsert with ST_GeomFromGeoJSON/ST_AsGeoJSON), \
               MaterialRepo (create/list_by_tenant/update/delete with ExtrusionBehavior JSONB), \
               TierAssignmentRepo (get_by_project_and_tier/set_assignments transactional), \
               TenantRepo (create/get_by_id). \
               Unblocks S.INFRA.1 (full-stack round-trip) and S.INFRA.2 (tenant isolation) \
               once T-004-01 (Axum routes) is also delivered.",
    },
    Milestone {
        label: "Lambda deploy: cross-compile + SST + S3 bucket",
        delivered_by: Some("T-017-01"),
        unlocks: &["S.INFRA.1"],
        note: "scripts/build-lambda.sh cross-compiles plantastic-api to aarch64-linux \
               via cargo-zigbuild, producing target/lambda/plantastic-api/bootstrap. \
               infra/sst.config.ts deploys the binary as a Lambda function (provided.al2023, \
               arm64, 256 MB, 30s timeout) with Function URL (RESPONSE_STREAM for SSE), \
               DATABASE_URL from SSM secret, and an S3 bucket (Uploads) for scan artifacts. \
               just build-lambda calls the build script. just deploy runs npx sst deploy. \
               Unblocks S.INFRA.1 deployment path — the API can now run on Lambda. \
               Still needs: T-017-02 (secrets wiring), T-018-01 (CI/CD), \
               T-021-01 (Neon provisioning) before full-stack round-trip is live.",
    },
    Milestone {
        label: "Neon PostGIS + S3 CORS + secrets wiring",
        delivered_by: Some("T-017-02"),
        unlocks: &["S.INFRA.1"],
        note: "S3 bucket CORS configured in SST for browser direct uploads (PUT/POST/GET, \
               all origins, 1 hour max age). Doppler config (.doppler.yaml) for local dev \
               secrets injection. Migration script (scripts/migrate.sh) wraps sqlx-cli with \
               validation. Deployment verification script (scripts/verify-deploy.sh) tests \
               health, project CRUD round-trip, and cold start timing. Justfile recipes: \
               migrate (Doppler) and migrate-direct (env var). Neon setup guide documents \
               project creation, PostGIS enablement, connection string management (direct vs \
               pooled), Doppler config, SST secret setup, and troubleshooting. \
               Unblocks S.INFRA.1 deployment path — with T-017-01 (Lambda deploy) and this \
               ticket, the API can connect to Neon PostGIS and S3 from Lambda. \
               Still needs: actual Neon project provisioning (manual), SST secret set, \
               and first deploy + verify-deploy.sh run.",
    },
    Milestone {
        label: "pt-repo: connection hardening (timeouts, retry, Neon support)",
        delivered_by: Some("T-020-02"),
        unlocks: &["S.INFRA.1", "S.INFRA.2"],
        note: "PoolConfig struct with Lambda-tuned defaults: connect_timeout (15s), \
               acquire_timeout (10s), max_connections (5), min_connections (0), \
               max_retries (3), initial_backoff (500ms exponential). \
               create_pool_with_config() parses DATABASE_URL via PgConnectOptions, \
               wraps each attempt in tokio::time::timeout, retries only on transient \
               errors (Io, Tls, PoolTimedOut) with exponential backoff. Permanent \
               errors (Configuration, auth) fail immediately. tracing integration: \
               WARN on retry, INFO on connect, ERROR on exhaustion. \
               Neon support: sslnegotiation=direct and statement_cache_size=0 parsed \
               from URL automatically by sqlx. PgBouncer -pooler hostname is just DNS. \
               6 unit tests (config defaults, error classification), 4 integration tests \
               (default connect, custom config, invalid URL rejection, max_connections). \
               Unblocks reliable Neon serverless connections for Lambda cold-starts. \
               Still needs: load testing against real Neon to measure cold-start timings, \
               T-021-01 (Neon provisioning) for production use.",
    },
    Milestone {
        label: "SvelteKit frontend + CF Worker proxy",
        delivered_by: Some("T-005-02, T-005-03"),
        unlocks: &["S.INFRA.1", "S.3.4"],
        note: "T-005-02 delivered CF Worker proxy (worker/) with CORS handling, rate \
               limiting, auth passthrough, and SSE streaming to Lambda function URL. \
               T-005-03 delivered SvelteKit route skeleton (web/) with layout components, \
               API client wrapper (apiFetch, SSE reader), mock API mode for local dev, \
               and session/project stores. Pages: dashboard, zone editor, material \
               catalog, 3D viewer. Path to S.3.4: wire quote comparison view.",
    },
    Milestone {
        label: "pt-tenant: multi-tenant model + auth context",
        delivered_by: Some("T-003-02, T-004-02"),
        unlocks: &["S.INFRA.2"],
        note: "T-003-02 delivered TenantRepo in pt-repo (create/get_by_id). T-004-02 \
               delivered X-Tenant-Id header extractor pulling tenant UUID from every \
               request. verify_project_tenant() in zones.rs enforces ownership at route \
               level — returns 404 (not 403) to prevent existence leaking. All routes \
               use tenant-scoped queries (list_by_tenant) or verify-then-404 pattern. \
               S.INFRA.2 scenario validates isolation across two tenants end-to-end.",
    },
];
