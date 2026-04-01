# T-032-03 Research: scan-to-viewer-pipeline

## Goal

Wire the end-to-end path: PLY file → pt-scan → terrain GLB → Bevy viewer.
Prove "I scanned a real place and now I can orbit it in 3D in the browser."

## Existing Pieces

### pt-scan (crates/pt-scan/)

- `process_scan_timed(reader, config) → (PointCloud, ScanReport)` — full pipeline with timing
- `generate_terrain(cloud, config) → TerrainOutput` — Delaunay triangulation → decimation → GLB + PNG
- `TerrainOutput { mesh_glb: Vec<u8>, plan_view_png: Vec<u8>, metadata: TerrainMetadata }`
- GLB contains: POSITION (VEC3 f32), NORMAL (VEC3 f32), COLOR_0 (VEC4 u8 normalized), indices (u32)
- Node name: `"terrain"` — single mesh, no hierarchy
- Generator tag: `"plantastic/pt-scan"`
- Existing CLI example: `crates/pt-scan/examples/process_sample.rs` — writes `{stem}-terrain.glb`

### pt-scene (crates/pt-scene/)

- `generate_scene(zones, assignments, materials, tier) → SceneOutput` — zone-based extruded meshes
- GLB contains: POSITION, NORMAL, per-zone named nodes, PBR materials (no vertex colors)
- Generator tag: `"plantastic/pt-scene"`
- No current ability to accept a terrain GLB as base layer
- Zone scenes and terrain scenes are independent glTF files

### Bevy Viewer (apps/viewer/)

- `bridge.rs`: `LoadSceneCommand { url }` via postMessage `{ type: "loadScene", url: "..." }`
- `scene.rs`: `asset_server.load(&cmd.url)` → poll `gltf_assets.get()` → spawn `SceneRoot`
- Keep-until-ready pattern: old scene stays until new one loads
- Camera: PanOrbitCamera at (3,3,5) looking at origin, orbit/pan/zoom with mouse+touch
- Lighting: DirectionalLight at -45° pitch, configurable yaw, ambient fill at 200 brightness
- Picking: Pointer<Click> → entity Name → zoneTapped postMessage
- **Already loads arbitrary glTF from URL** — the viewer is format-agnostic

### Justfile

- `process-scan path` recipe exists, calls `cargo run -p pt-scan --example process_sample --release`
- No scan-to-viewer recipe yet
- No local file server recipe

### Coordinate Systems

- pt-scan GLB: raw PLY coordinates (meters, Z-up from LiDAR)
- pt-scene GLB: Polygon X → Scene X, Polygon Y → Scene Z, height → Scene Y (feet, Y-up)
- Bevy: Y-up coordinate system
- **Mismatch**: pt-scan outputs Z-up (LiDAR convention), Bevy expects Y-up
- The glTF spec says +Y is up. Bevy's glTF loader expects this.
- pt-scan's to_glb() writes raw positions as-is — Z is the vertical axis from the PLY
- This means terrain will render sideways unless we rotate or remap coordinates

### File Serving for Local Dev

- Bevy WASM viewer runs in browser — can't load file:// URLs due to CORS
- Need a local HTTP server to serve GLB files
- Options: `python3 -m http.server`, `npx serve`, or Rust-based `miniserve`
- The justfile recipe should handle this automatically

## Key Findings

1. **The viewer already loads any glTF URL** — no viewer changes needed for basic loading
2. **Coordinate mismatch is the main technical challenge** — pt-scan writes Z-up, Bevy wants Y-up
3. **pt-scene terrain integration is out of scope** — the ticket says "Later (T-033+): classified features become named mesh nodes." The terrain renders standalone first.
4. **Vertex colors work in Bevy** — Bevy's glTF loader supports COLOR_0 attribute; StandardMaterial will use vertex colors when present (no explicit material needed)
5. **Camera framing** — default camera at (3,3,5) may be too close for terrain spanning 30+ meters. The recipe should print instructions to zoom out.
6. **The process_sample example already produces the GLB** — the scan-to-viewer recipe can chain it

## Constraints

- The viewer is a WASM app compiled separately (excluded from workspace)
- For local dev, we need a simple HTTP server — `python3 -m http.server` is universal
- The PLY file is 294 MB, gitignored — not available in CI
- Integration test must use synthetic data (existing pattern in pt-scan/tests/integration.rs)

## Relevant Scenarios

- S.1.1 (Scan processing) — already passes at OneStar
- S.2.4 (3D preview per tier) — passes at ThreeStar (zone scenes, not terrain)
- S.4.1 (3D viewer on tablet) — NOT IMPLEMENTED, prereqs met
- This ticket advances S.4.1 by proving terrain rendering in the viewer

## Open Questions

1. Should the coordinate transform (Z-up → Y-up) happen in pt-scan's export or in a new pipeline step?
2. Should the justfile recipe start a local server automatically or just print the command?
3. What camera position is appropriate for a terrain mesh spanning ~30m × ~30m?
