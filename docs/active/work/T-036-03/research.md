# T-036-03 Research: Scan-to-Quote Demo

## Objective

Stitch the full pipeline — scan → detect → measure → design → quote — into a single
CLI example that produces a three-tier landscaping quote from a PLY scan file.

## Existing Pipeline Components

### 1. pt-scan (crates/pt-scan)

- `process_scan(reader, config) → PointCloud` — parse PLY, downsample, outlier removal, RANSAC ground fit
- `process_scan_timed(reader, config) → (PointCloud, ScanReport)` — same + timing
- `cluster_obstacles(points, config) → ClusterResult` — DBSCAN clustering
- `extract_candidates(clusters, points, ground_plane) → Vec<FeatureCandidate>` — geometric summaries
- `measure_gaps(candidates, ground_plane, config) → Vec<Gap>` — pairwise gap measurement
- `generate_terrain(cloud, config) → TerrainOutput` — GLB mesh + plan-view PNG

Key types: `PointCloud { ground, obstacles, metadata }`, `FeatureCandidate` (cluster_id,
centroid, height_ft, spread_ft, etc.), `Gap { area_sqft, clear_width_ft, clear_length_ft, ... }`

### 2. pt-features (crates/pt-features)

- `FeatureClassifier` trait: `classify(candidates, address, climate_zone) → Vec<ClassifiedFeature>`
- `MockFeatureClassifier` — geometry-based heuristics, deterministic
- `ClaudeCliClassifier` — routes to `claude` CLI (subscription, zero API cost)
- `BamlFeatureClassifier` — real BAML API calls

`ClassifiedFeature` fields: cluster_id, label, category (tree/structure/hardscape/utility),
species, confidence, reasoning, landscape_notes.

### 3. pt-quote (crates/pt-quote)

- `compute_quote(zones, tier, materials, tax) → Quote`
- Pure computation, no I/O
- `Zone { id, geometry: Polygon<f64>, zone_type, label }` from pt-project
- `Material` built via `Material::builder(name, category).unit().price_per_unit().build()`
- `Tier { level: TierLevel, assignments: Vec<MaterialAssignment> }`
- `Quote { tier, line_items: Vec<LineItem>, subtotal, tax, total }`

Units: SqFt (area), CuYd (volume = area × depth), LinearFt (perimeter), Each (count).

### 4. pt-scene (crates/pt-scene)

- `generate_scene(zones, materials, assignments) → SceneOutput` — glTF .glb output
- Triangulates zone polygons into named mesh nodes

### 5. pt-materials (crates/pt-materials)

- `Material::builder(name, category)` → chain `.unit()`, `.price_per_unit()`, `.depth_inches()`, `.extrusion()` → `.build()`
- Categories: Hardscape, Softscape, Edging, Fill
- `ExtrusionBehavior`: SitsOnTop, Fills, BuildsUp

### 6. pt-project (crates/pt-project)

- `Zone`, `ZoneId`, `ZoneType` (Bed, Patio, Path, Lawn, Wall, Edging)
- `Tier`, `TierLevel` (Good, Better, Best), `MaterialAssignment`
- `AssignmentOverrides { price_override, depth_override_inches }`

## Existing Example: process_sample.rs

Located at `crates/pt-scan/examples/process_sample.rs`. Runs stages 1-8:
parse → downsample → outlier removal → RANSAC → terrain gen → clustering →
gap measurement → annotated plan view. Already has a `just process-scan` recipe.

This is the natural base to extend. It already does steps 1-4 of the ticket's pipeline.
The remaining steps are: classify features, bridge gap→zone, build materials/tiers,
compute quotes, print three-tier summary.

## Test Data

- `assets/scans/samples/powell-market-downsampled.ply` (1.8 MB) — the target scan
- Processing yields: 121,656 points, ~29,460 obstacles, 2+ clusters (tree trunks),
  measured gaps between features

## Gap Between Gap and Zone

The central design challenge: `measure_gaps()` returns `Gap { area_sqft, clear_width_ft,
clear_length_ft, midpoint }` — a rectangular approximation. `compute_quote()` needs
`Zone { geometry: Polygon<f64> }`. The bridge: create a rectangle polygon from the
gap's midpoint, width, and length. This is straightforward geometry.

## Mock vs Real LLM

The ticket requires both paths:
- Mock generators: < 10s, CI-friendly, deterministic
- ClaudeCliGenerator: < 2min, uses subscription

For the demo, the mock path is the default. A `--live` flag can switch to Claude CLI.

## Numbers Must Come From pt-quote

The ticket explicitly states: "The numbers must come from pt-quote computation, not LLM output."
This means the demo must construct Zones + Materials + Tiers from the scan analysis,
then call `compute_quote()` to get the actual dollar amounts.

## Relevant Scenarios

- S.3.1: Quantity computation from geometry (already passing at OneStar)
- S.3.2: Three-tier quote generation (already passing at OneStar)
- This ticket doesn't add new scenarios but exercises S.3.1 + S.3.2 end-to-end from real scan data

## Workspace Cargo.toml

The example will need dependencies on: pt-scan, pt-features, pt-quote, pt-project,
pt-materials, pt-geo (transitive), tokio (for async classifier), serde_json (for report).

## Summary

All building blocks exist. The work is stitching: bridge gap measurements into Zone
polygons, define three material tiers (Good/Better/Best) with realistic landscaping
materials, call compute_quote() for each, and format the output as the investor-ready
demo described in the ticket.
