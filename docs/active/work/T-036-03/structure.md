# T-036-03 Structure: Scan-to-Quote Demo

## Files Created

### crates/pt-scan/examples/scan_to_quote.rs (~250 lines)

Single-file example binary. Sections:

1. **Imports & constants** (~20 lines)
   - pt-scan, pt-features, pt-quote, pt-project, pt-materials, geo, rust_decimal
   - Scan config constants (same as process_sample.rs)
   - Material pricing constants

2. **main()** (~80 lines)
   - Parse CLI args (ply path, --live flag)
   - Run pipeline stages 1-7 with timing
   - Print three-tier quote summary
   - Write terrain GLB

3. **gap_to_zone()** (~20 lines)
   - `fn gap_to_zone(gap: &Gap, candidates: &[FeatureCandidate]) -> Zone`
   - Build oriented rectangle polygon from gap midpoint + dimensions
   - Returns Zone with type Bed, labeled "Planter"

4. **build_materials()** (~40 lines)
   - Returns `(Vec<Material>, Vec<Material>, Vec<Material>)` for good/better/best
   - Each vec has: plant material (SqFt pricing), soil (CuYd), optional edging (LinearFt)

5. **build_tier()** (~15 lines)
   - `fn build_tier(level: TierLevel, zone: &Zone, materials: &[Material]) -> Tier`
   - Creates MaterialAssignment for each material → the zone

6. **print_quote()** (~30 lines)
   - Formats a Quote as the investor-ready output
   - Converts SqFt line items to "N × Plant Name" display
   - Aligns dollar amounts

7. **Helpers** (~20 lines)
   - `format_bytes()`, `format_count()` — reused from process_sample.rs
   - `plant_count()` — compute integer plant count from sqft quantity and spacing

## Files Modified

### Justfile (+3 lines)

Add recipe:
```
# Run the full scan-to-quote demo pipeline
scan-to-quote path="assets/scans/samples/powell-market-downsampled.ply":
    cargo run -p pt-scan --example scan_to_quote --release -- "{{path}}"
```

### crates/pt-scan/Cargo.toml (+dependencies for example)

Add dev-dependencies needed by the scan_to_quote example:
- pt-features (workspace)
- pt-quote (workspace)
- pt-project (workspace)
- pt-materials (workspace)
- rust_decimal (workspace)
- geo (workspace)
- tokio (workspace, features = ["rt", "macros"])

These are dev-dependencies, not regular dependencies — pt-scan doesn't depend on
pt-quote at the library level.

## Module Boundaries

- No new library code in any crate
- All glue logic lives in the example file
- The example depends on public APIs of 5 crates but adds no coupling between them
- pt-quote remains pure computation with no knowledge of scans
- pt-scan remains pure geometry with no knowledge of quotes

## Interface Contracts

The example calls these public functions in order:

1. `pt_scan::process_scan_timed(reader, &config)` → `(PointCloud, ScanReport)`
2. `pt_scan::cluster_obstacles(&cloud.obstacles, &cluster_config)` → `ClusterResult`
3. `pt_scan::extract_candidates(&clusters, &cloud.obstacles, &ground_plane)` → `Vec<FeatureCandidate>`
4. `pt_features::FeatureClassifier::classify(&mock, &candidates, addr, zone)` → `Vec<ClassifiedFeature>`
5. `pt_scan::measure_gaps(&candidates, &ground_plane, &gap_config)` → `Vec<Gap>`
6. `pt_quote::compute_quote(&zones, &tier, &materials, None)` → `Quote`
7. `pt_scan::generate_terrain(&cloud, &export_config)` → `TerrainOutput`

No new traits, structs, or public functions in any crate.

## Data Flow

```
FeatureCandidate.height_ft/spread_ft → MockFeatureClassifier → "tree trunk" label
Gap.area_sqft + clear_width_ft + clear_length_ft → gap_to_zone() → Zone polygon
Zone + Material catalog → Tier assignments → compute_quote() → Quote with $$ amounts
```

## Output Artifacts

Running `just scan-to-quote`:
- stdout: pipeline progress + three-tier quote summary
- `{stem}-terrain.glb`: terrain mesh for Bevy viewer
- `{stem}-planview.png`: orthographic plan view
