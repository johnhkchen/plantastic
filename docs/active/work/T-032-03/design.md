# T-032-03 Design: scan-to-viewer-pipeline

## Decision Summary

1. Fix the coordinate system in pt-scan's GLB export (Z-up → Y-up)
2. Add a `scan-to-viewer` justfile recipe that chains process → serve
3. Add a `scan_to_viewer` example in pt-scan that does the full pipeline
4. No changes to pt-scene or the Bevy viewer

## Option Analysis

### Coordinate Transform: Where?

**Option A: Fix in pt-scan export (to_glb)**
- Swap Y↔Z in position and normal arrays during GLB serialization
- All downstream consumers (viewer, API) get correct Y-up GLB automatically
- One change, no pipeline step, no runtime cost
- Consistent with glTF spec ("Y axis is up")

**Option B: Add a transform node in the glTF JSON**
- Set `nodes[0].rotation` to rotate 90° around X-axis
- Keeps raw data intact, viewer applies transform
- More "correct" from a data-provenance perspective
- But: Bevy flattens transforms on spawn — same visual result, more complexity

**Option C: Transform in a separate pipeline step**
- New function `reorient_glb()` between scan and viewer
- Most flexible but unnecessary indirection for a single axis swap

**Decision: Option A** — fix in pt-scan's `to_glb()`. The glTF spec says Y-up. Every glTF consumer expects Y-up. Outputting Z-up is a bug, not a design choice. The swap is trivial: write `[x, z, -y]` instead of `[x, y, z]` for positions and normals (negate Y→-Y to preserve handedness).

### Pipeline Integration: justfile recipe vs standalone binary

**Option A: justfile recipe chaining existing CLI**
- `just scan-to-viewer <ply>` = process-scan + serve directory
- Reuses existing `process_sample` example
- Simple, no new binary

**Option B: New standalone example**
- `scan_to_viewer.rs` that does process + write + print viewer instructions
- More self-contained, better for demos

**Decision: Both** — add a justfile recipe that calls the existing process-sample, then serves the output directory with a local HTTP server. The process_sample example already writes the GLB; we just need to chain it with serving.

### Local File Server

**Option A: python3 -m http.server**
- Universally available on macOS/Linux
- Simple, zero install
- No CORS headers by default (problem for WASM fetch)

**Option B: npx serve**
- Adds CORS headers automatically
- Requires Node.js

**Option C: Print instructions only**
- Recipe prints the GLB path and a suggested command
- User can use any server they prefer

**Decision: Option C** — the recipe processes the scan and prints instructions to serve the GLB. The viewer is a separately-built WASM app; the user is already in a dev workflow that has the viewer running. Adding auto-serve couples too many concerns. Print the GLB path and a one-liner to serve it.

### pt-scene Terrain Integration

The ticket says `pt-scene's generate_scene() can accept a terrain GLB as the base layer`. However, the simplest path is:

1. Terrain GLB and zone GLB are separate files
2. The viewer loads them sequentially (loadScene for terrain, then loadScene for zones)
3. OR: we add a multi-scene capability later (T-033+)

For this ticket: prove terrain renders standalone. The viewer already handles a single GLB. Compositing terrain + zones is a separate concern tracked in the ticket's "Later" section.

## Rejected Approaches

- **Modifying the Bevy viewer to handle Z-up**: Wrong layer. The data should be spec-compliant.
- **Adding a new crate for the pipeline**: Over-engineering. The pipeline is just: read PLY → process → export → serve. All pieces exist.
- **Embedding a web server in the example**: Couples scan processing to HTTP serving.

## Testing Strategy

- Unit test: verify GLB from `generate_terrain()` has Y-up coordinates (positions[i][1] is the vertical axis)
- Integration test: synthetic point cloud → full pipeline → verify GLB node name, coordinate system, COLOR_0 presence
- Manual verification: run the recipe on Powell & Market scan, open in viewer, confirm brick paths visible as ground plane

## Scenarios Affected

- S.1.1: coordinate fix improves GLB quality (OneStar → no star change, but data correctness)
- S.4.1: this ticket is the main blocker for "3D viewer on tablet" with real scan data
