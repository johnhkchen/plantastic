# T-011-01 Design: pt-satellite crate

## Problem

Given an address, produce a `ProjectBaseline` containing lot boundary polygon, detected
trees (location + height + spread), and a sun exposure grid. For OneStar, all external
data is embedded; the design must cleanly support real data sources later.

## Option A: Trait-based data source abstraction

Define traits `Geocoder`, `ParcelSource`, `CanopySource`. Implement `EmbeddedSource` for
OneStar. The `BaselineBuilder` takes trait objects or generic params, calls sources in
sequence, then runs pt-solar for the sun grid.

**Pros**: Clean separation. Easy to add real sources later. Testable in isolation.
**Cons**: Slightly more types upfront. Generic plumbing for a crate that currently has
one implementation.

## Option B: Enum-based source selector

Single `DataSource` enum with variants `Embedded` and `Remote(config)`. Functions match
on the variant to choose behavior.

**Pros**: Simpler — no traits, no generics.
**Cons**: Adding a new source means modifying every match arm. Harder to test individual
sources in isolation. Mixes I/O and logic in the same function.

## Option C: Direct embedded implementation (no abstraction)

Hardcode the embedded data directly in the pipeline functions. Refactor when real sources
arrive.

**Pros**: Minimal code. Ship fast.
**Cons**: No seam for testing. When real sources arrive, the refactor touches everything.
Violates the project's "no mocking across crate boundaries" principle in reverse — it
makes future real implementations harder to integrate-test.

## Decision: Option A (trait-based)

Rationale:
1. The spec explicitly calls pt-satellite an I/O crate. Real sources (geocoding API,
   parcel API, canopy dataset) are coming in T-011-02 or follow-on work. The trait
   boundary is load-bearing, not speculative.
2. The trait surface is small (3 traits, ~1 method each). The cost is ~30 lines of trait
   definitions plus ~60 lines of embedded implementations.
3. It matches the project's testing philosophy: real integration tests can swap in real
   sources without changing the core pipeline.

## Architecture

```
                      ┌──────────────┐
  address ──────────▶ │ BaselineBuilder │
                      │              │
                      │  geocoder    │──▶ lat/lng
                      │  parcel_src  │──▶ lot polygon
                      │  canopy_src  │──▶ trees[]
                      │  pt-solar    │──▶ ExposureGrid
                      │              │
                      └──────┬───────┘
                             │
                             ▼
                      ProjectBaseline {
                        coordinates,
                        lot_polygon,
                        trees,
                        sun_grid,
                      }
```

### Traits

```rust
pub trait Geocoder {
    fn geocode(&self, address: &str) -> Result<Coordinates, SatelliteError>;
}

pub trait ParcelSource {
    fn lot_boundary(&self, coords: &Coordinates) -> Result<LotBoundary, SatelliteError>;
}

pub trait CanopySource {
    fn detect_trees(&self, bounds: &LatLngBounds) -> Result<Vec<DetectedTree>, SatelliteError>;
}
```

### Embedded implementation

`EmbeddedSource` implements all three traits. Stores a `HashMap<&str, EmbeddedSiteData>`
keyed by normalized address string. Ships with 1-2 known SF addresses:
- "1234 Noriega St, San Francisco, CA" — typical Inner Sunset residential lot

The embedded data is realistic: polygon coords from real SF parcel shapes (simplified),
tree positions matching Bay Area residential canopy patterns.

### Sun grid computation

Not behind a trait — always real. Calls `pt_solar::radiance_grid()` with the lot
boundary's bounding box, growing season date range (March–September), and a coarse
config (50m resolution, 6 sample days for speed in tests). This matches S.1.3's
approach.

### Output types

```rust
pub struct ProjectBaseline {
    pub coordinates: Coordinates,
    pub lot_boundary: LotBoundary,
    pub trees: Vec<DetectedTree>,
    pub sun_grid: ExposureGrid,
}

pub struct LotBoundary {
    pub polygon: Polygon<f64>,     // WGS84 lat/lng coordinates
    pub area_sqft: f64,            // computed from polygon
    pub source: DataSourceLabel,
}

pub struct DetectedTree {
    pub location: Coordinates,     // WGS84 position
    pub height_ft: f64,            // estimated height
    pub spread_ft: f64,            // estimated canopy spread (diameter)
    pub confidence: f64,           // 0.0–1.0
}

pub enum DataSourceLabel {
    Embedded,
    Municipal,
    Satellite,
}
```

### Error type

```rust
#[derive(Debug, thiserror::Error)]
pub enum SatelliteError {
    #[error("address not found: {0}")]
    AddressNotFound(String),
    #[error("no parcel data for coordinates ({lat}, {lng})")]
    NoParcelData { lat: f64, lng: f64 },
    #[error("canopy data unavailable for bounds")]
    CanopyUnavailable,
    #[error("solar computation failed: {0}")]
    SolarError(String),
}
```

## Rejected Alternatives

- **Async traits**: Not needed for OneStar (embedded data is sync). When real HTTP
  sources arrive, they can use `async-trait` or Rust's native async traits. The sync
  trait surface is simpler and sufficient for now.

- **Include pt-climate in baseline**: The acceptance criteria don't mention climate data.
  Climate enrichment can be added to `ProjectBaseline` in a follow-on ticket without
  changing the core pipeline.

- **Store grid by reference (sun_grid_key)**: The DB schema mentions `sun_grid_key` for
  production (grid stored in S3). For OneStar, we include the full `ExposureGrid` inline.
  The field can become an enum `GridRef::Inline(ExposureGrid) | GridRef::Key(String)`
  later.

## Scenario S.1.2 Test Design

```
1. Construct EmbeddedSource
2. Construct BaselineBuilder with EmbeddedSource
3. Call build("1234 Noriega St, San Francisco, CA")
4. Assert: coordinates near (37.76, -122.49)
5. Assert: lot polygon has 4+ vertices, area 3000-15000 sqft
6. Assert: 1-5 trees detected, heights 10-80 ft, spreads 5-40 ft
7. Assert: sun grid width/height > 0, values in 8-18 range (Bay Area growing season)
8. Return Pass(OneStar)
```
