# T-015-02 Review: Mesh Generation & Export

## Summary

T-015-02 extends the pt-scan crate with mesh generation and export capabilities. The full scan pipeline is now: PLY → parse → filter → classify → **triangulate → decimate → glTF + PNG + metadata**.

## Files Changed

### Created
| File | Lines | Purpose |
|------|-------|---------|
| `crates/pt-scan/src/mesh.rs` | ~370 | Delaunay triangulation + meshopt decimation + normal computation |
| `crates/pt-scan/src/export.rs` | ~570 | glTF binary export, PNG plan view, pipeline orchestrator |

### Modified
| File | Change |
|------|--------|
| `crates/pt-scan/Cargo.toml` | Added delaunator, meshopt, image (png), serde_json |
| `crates/pt-scan/src/lib.rs` | Added mesh/export modules + re-exports |
| `crates/pt-scan/src/error.rs` | Added MeshGeneration, ExportError variants |
| `crates/pt-scan/tests/integration.rs` | Added 3 integration tests |
| `tests/scenarios/src/suites/site_assessment.rs` | S.1.1 now validates mesh gen + export |
| `tests/scenarios/src/progress.rs` | Milestone updated to T-015-02, expanded note |

## Test Coverage

### Before: 18 tests (15 unit + 3 integration)
### After: 31 tests (25 unit + 6 integration)

| Module | Tests | What's covered |
|--------|-------|----------------|
| mesh.rs | 5 unit | triangulate (square, insufficient, default color), decimate (passthrough, reduction) |
| export.rs | 5 unit | generate_terrain outputs, GLB magic+version, GLB JSON chunk, PNG magic, metadata consistency |
| integration.rs | 3 new | full pipeline PLY→artifacts, GLB structure validity, PNG decodability |
| S.1.1 scenario | 1 | end-to-end with mesh gen + export validation |

All expected values in tests are computed independently (rule 2). No mocking across crate boundaries (rule 3). All tests use `timed()` wrapper (rule 9).

## Scenario Dashboard

### Before
```
Effective savings: 54.0 min / 240.0 min (22.5%)
S.1.1  Scan processing  PASS ★☆☆☆☆
```

### After
```
Effective savings: 54.0 min / 240.0 min (22.5%)
S.1.1  Scan processing  PASS ★☆☆☆☆
```

Star rating unchanged (OneStar) because the new capability is still pure computation — no API integration. The S.1.1 test now exercises the full pipeline including mesh generation and export, but the star level correctly reflects that this isn't reachable via API yet. T-016-01 (scan upload API) is the path to TwoStar.

No regressions in any scenario. All 7 passing scenarios remain passing.

## Acceptance Criteria Checklist

- [x] Delaunay triangulation of ground points → triangle mesh
- [x] Mesh decimation to configurable target (default ~50k triangles)
- [x] Vertex colors preserved through triangulation if available
- [x] glTF binary (.glb) export of terrain mesh
- [x] Top-down orthographic projection → PNG image (plan view)
  - [x] Configurable resolution (pixels_per_meter, default 30.0 ≈ 10 px/ft)
  - [x] Ground colored by elevation + vertex color
  - [x] Above-ground points rendered as darker overlay (canopy_overlay option)
- [x] Metadata JSON: bbox, elevation range, point count, triangle count, processing time
- [x] End-to-end test: PLY → (terrain.glb, planview.png, metadata.json)
- [x] S.1.1 scenario registered and passing at ★☆☆☆☆
- [x] Milestone claimed: "pt-scan: PLY parsing + mesh generation" → T-015-02

### Deferred: Bevy cross-validation
The acceptance criteria include "The generated glTF loads in the Bevy viewer from T-013-01 (cross-validate)." The GLB format is correct per spec (magic, version, chunk structure, all accessors and bufferViews properly defined with POSITION/NORMAL/COLOR_0). However, loading it in the actual Bevy WASM viewer requires a running browser environment which is outside the CI/test scope. The GLB structure tests validate format correctness. Manual cross-validation can be done by copying a generated .glb to `apps/viewer/assets/models/`.

## Architecture Decisions

1. **Modules in pt-scan, not separate crate**: scan → mesh → export is one pipeline. Splitting would add dependency indirection with no benefit.
2. **Manual GLB construction**: Simple enough for our single-mesh case. `gltf-json` crate would have been overkill.
3. **meshopt for decimation**: Industry-standard QEM simplification. The C dependency compiles cleanly via cc crate.
4. **delaunator for triangulation**: Fast batch 2D Delaunay, ideal for one-shot scan processing. v1 API is stable.
5. **pixels_per_meter** (not pixels_per_foot): Consistent with metric coordinates used throughout the scan pipeline.

## Open Concerns

1. **Performance at scale**: The current implementation handles the test cases (500-1000 points) in <100ms. Real iPhone scans produce ~300-400K ground points after filtering. Delaunay triangulation at that scale is O(n log n) and should take ~1-2 seconds; meshopt decimation is fast. Not tested at that scale in this ticket — may need profiling once real PLY files are available.

2. **Coordinate system**: The code assumes PLY X/Y/Z maps directly to glTF coordinates. SiteScape exports may need axis remapping (e.g., Y-up vs Z-up). This should be validated when real scan data is available and adjusted in the parser or export layer.

3. **PNG quality**: The scanline triangle rasterizer is simple but correct. No anti-aliasing. For presentation-quality plan views, a library like tiny-skia could be substituted later.

## Quality Gate

```
just check ✓
  fmt-check ✓
  lint      ✓  (clippy strict, 0 warnings)
  test      ✓  (31 pt-scan tests + full workspace)
  scenarios ✓  (54.0 min, 0 regressions)
```
