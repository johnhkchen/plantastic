# T-011-01 Structure: pt-satellite crate

## New Files

### `crates/pt-satellite/Cargo.toml`

Standard workspace member. Dependencies:
- `geo.workspace = true` (for Polygon)
- `geojson.workspace = true` (for GeoJSON serialization of lot polygon)
- `serde.workspace = true`
- `serde_json.workspace = true`
- `chrono.workspace = true` (for date range in sun grid computation)
- `thiserror.workspace = true`
- `pt-solar = { path = "../pt-solar" }` (Coordinates, ExposureGrid, radiance_grid, etc.)

Dev dependencies:
- `approx = "0.5"`
- `pt-test-utils = { path = "../pt-test-utils" }`

### `crates/pt-satellite/src/lib.rs`

Module declarations and public re-exports:
```
pub mod types;
pub mod error;
pub mod traits;
pub mod embedded;
pub mod builder;

pub use types::*;
pub use error::SatelliteError;
pub use traits::{Geocoder, ParcelSource, CanopySource};
pub use embedded::EmbeddedSource;
pub use builder::BaselineBuilder;
```

### `crates/pt-satellite/src/types.rs`

Public types: `ProjectBaseline`, `LotBoundary`, `DetectedTree`, `DataSourceLabel`.
Re-exports `pt_solar::Coordinates` for convenience (avoids callers needing pt-solar dep).

### `crates/pt-satellite/src/error.rs`

`SatelliteError` enum with `thiserror::Error` derive:
- `AddressNotFound(String)`
- `NoParcelData { lat: f64, lng: f64 }`
- `CanopyUnavailable`
- `SolarError(String)`

### `crates/pt-satellite/src/traits.rs`

Three traits defining data source contracts:
- `Geocoder::geocode(&self, address: &str) -> Result<Coordinates, SatelliteError>`
- `ParcelSource::lot_boundary(&self, coords: &Coordinates) -> Result<LotBoundary, SatelliteError>`
- `CanopySource::detect_trees(&self, bounds: &LatLngBounds) -> Result<Vec<DetectedTree>, SatelliteError>`

### `crates/pt-satellite/src/embedded.rs`

`EmbeddedSource` struct implementing all three traits.
Contains hardcoded data for known SF test addresses:
- "1234 noriega st, san francisco, ca" — Inner Sunset residential lot
  - Coordinates: (37.7601, -122.4862)
  - Lot polygon: simplified 5-vertex polygon (~5,400 sqft)
  - Trees: 3 trees (mature oak, medium cypress, small plum)

Address matching is case-insensitive, whitespace-normalized.
`#[cfg(test)] mod tests` block with unit tests for the embedded data.

### `crates/pt-satellite/src/builder.rs`

`BaselineBuilder` struct with generic data source fields.
Single public method: `build(&self, address: &str) -> Result<ProjectBaseline, SatelliteError>`

Pipeline:
1. `geocoder.geocode(address)` → coordinates
2. Compute `LatLngBounds` from coordinates (padded bounding box, ~100m each direction)
3. `parcel_source.lot_boundary(&coords)` → lot boundary
4. `canopy_source.detect_trees(&bounds)` → trees
5. `pt_solar::radiance_grid(&bounds, date_range, &config)` → sun grid
6. Assemble and return `ProjectBaseline`

Sun grid config: 10m resolution, 6 sample days (fast for OneStar, sufficient for validation).
Date range: growing season March 1 – September 30 (matches S.1.3 convention).

`#[cfg(test)] mod tests` block with integration test using EmbeddedSource.

## Modified Files

### `tests/scenarios/src/suites/site_assessment.rs`

Replace `s_1_2_satellite_prepopulation` body:
- Import `pt_satellite::{BaselineBuilder, EmbeddedSource}`
- Construct builder with EmbeddedSource
- Call `build("1234 Noriega St, San Francisco, CA")`
- Assert coordinates, lot area, tree count, sun grid validity
- Return `ScenarioOutcome::Pass(Integration::OneStar)`

### `tests/scenarios/Cargo.toml`

Add dependency: `pt-satellite = { path = "../../crates/pt-satellite" }`

### `tests/scenarios/src/progress.rs`

Update the pt-satellite milestone (line 47):
- `delivered_by: Some("T-011-01")`
- `note:` describing what was delivered and what it enables

## Module Boundaries

```
pt-satellite (public API)
├── ProjectBaseline          ← returned by builder
├── LotBoundary             ← lot polygon + area + source label
├── DetectedTree            ← tree location + dimensions
├── DataSourceLabel         ← Embedded | Municipal | Satellite
├── SatelliteError          ← error enum
├── Geocoder trait          ← address → coords
├── ParcelSource trait      ← coords → lot boundary
├── CanopySource trait      ← bounds → trees
├── EmbeddedSource          ← implements all three traits
├── BaselineBuilder         ← orchestrates pipeline
└── Coordinates (re-export) ← from pt-solar
```

## Files NOT Modified

- `crates/pt-project/` — baseline type will be integrated in a future ticket
- `crates/pt-repo/` — database storage is a future concern
- `crates/plantastic-api/` — API route is T-011-02
- `Cargo.toml` (root) — workspace members are auto-discovered via `crates/*` glob
