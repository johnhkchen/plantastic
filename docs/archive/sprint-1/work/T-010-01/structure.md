# Structure — T-010-01: pt-solar engine

## New files

### `crates/pt-solar/Cargo.toml`
```toml
[package]
name = "pt-solar"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
chrono.workspace = true
serde.workspace = true

[dev-dependencies]
approx = "0.5"
pt-test-utils = { path = "../pt-test-utils" }

[lints]
workspace = true
```

Dependencies: only `chrono` (dates) and `serde` (serialization). No `geo` — we use our own `Coordinates` type since geo::Coord uses x/y not lat/lng. `approx` for float comparison in tests.

### `crates/pt-solar/src/lib.rs`
Module declarations and re-exports. Public API surface:
```rust
pub mod classification;
pub mod grid;
pub mod position;
pub mod seasonal;
pub mod sun_hours;
pub mod types;

// Re-export key types at crate root
pub use types::*;
pub use classification::LightCategory;
pub use position::sun_position;
pub use sun_hours::daily_sun_hours;
pub use seasonal::annual_sun_hours;
pub use grid::{ExposureGrid, GridConfig, radiance_grid};
```

### `crates/pt-solar/src/types.rs`
Shared types:
- `Coordinates { latitude: f64, longitude: f64 }` — Serialize, Deserialize, Debug, Clone, Copy
- `SolarPosition { altitude_degrees: f64, azimuth_degrees: f64 }` — Debug, Clone, Copy
- `PolarCondition` enum — Normal, MidnightSun, PolarNight
- `DailySunData { date: NaiveDate, sun_hours: f64, polar_condition: PolarCondition }`
- `SeasonalSummary { start: NaiveDate, end: NaiveDate, average_sun_hours: f64, min_sun_hours: f64, max_sun_hours: f64, daily_data: Vec<DailySunData> }`
- Constants: `SAMPLING_INTERVAL_MINUTES: u32 = 5`, `SAMPLES_PER_DAY: u32 = 288`

### `crates/pt-solar/src/position.rs`
Core solar position algorithm (~80 lines):
- `sun_position(coords: &Coordinates, dt: NaiveDateTime) -> SolarPosition`
- Internal helpers: `julian_date()`, `solar_declination()`, `hour_angle()`, `solar_altitude()`, `solar_azimuth()`
- Unit tests: Portland summer/winter solstice noon altitude, azimuth at noon, negative altitude at night

### `crates/pt-solar/src/sun_hours.rs`
Daily sun hours integration (~40 lines):
- `daily_sun_hours(coords: &Coordinates, date: NaiveDate) -> DailySunData`
- Iterates 288 5-minute samples, counts altitude > 0, converts count to hours
- Polar condition detection: if all 288 are positive → MidnightSun, if none → PolarNight
- Unit tests: Portland solstices/equinoxes, Singapore year-round, Tromso polar cases

### `crates/pt-solar/src/seasonal.rs`
Date range aggregation (~60 lines):
- `annual_sun_hours(coords: &Coordinates, start: NaiveDate, end: NaiveDate) -> SeasonalSummary`
- Iterates each day in range, calls `daily_sun_hours`, computes min/max/average
- Unit tests: full year summary for Portland

### `crates/pt-solar/src/classification.rs`
Light categories (~30 lines):
- `LightCategory` enum: FullSun, PartSun, PartShade, FullShade
- `classify(sun_hours: f64) -> LightCategory`
- `LightCategory::label() -> &'static str`
- `LightCategory::sun_hours_range() -> &'static str`
- Unit tests: boundary values (2.0, 4.0, 6.0)

### `crates/pt-solar/src/grid.rs`
Spatial grid computation (~100 lines):
- `LatLngBounds { south: f64, west: f64, north: f64, east: f64 }`
- `GridConfig { resolution_meters: f64, sample_days: u32 }`
- `ExposureGrid { bounds, resolution_meters, width, height, values: Vec<f32>, sample_days_used, compute_time_ms }`
- `radiance_grid(bounds: &LatLngBounds, date_range: (NaiveDate, NaiveDate), config: &GridConfig) -> ExposureGrid`
- Internal: `grid_dimensions()`, `cell_center()`, `generate_sample_dates()`, meters-to-degrees conversion
- Unit tests: grid dimensions, cell centers, small grid computation

## Modified files

### `tests/scenarios/Cargo.toml`
Add dependency: `pt-solar = { path = "../../crates/pt-solar" }`

### `tests/scenarios/src/suites/site_assessment.rs`
Replace `s_1_3_sun_exposure_analysis` stub with real implementation that:
1. Creates a grid over SF (37.7749, -122.4194) with 50m resolution
2. Computes for March-September growing season
3. Validates average sun hours in expected range
4. Validates light classifications exist for all cells
5. Returns `Pass(Integration::OneStar)`

### `tests/scenarios/src/progress.rs`
Update the pt-solar milestone:
- `delivered_by: Some("T-010-01")`
- `note:` describing what was delivered

## Module boundaries

```
pt-solar (no external crate deps except chrono + serde)
  types.rs        ← no internal deps
  position.rs     ← depends on types
  sun_hours.rs    ← depends on types, position
  seasonal.rs     ← depends on types, sun_hours
  classification.rs ← depends on types (just sun_hours f64 threshold)
  grid.rs         ← depends on types, position, sun_hours, classification
  lib.rs          ← re-exports from all modules
```

No circular dependencies. Each module depends only on types and lower-level modules.

## File count
- New: 8 files (Cargo.toml + 7 .rs files)
- Modified: 3 files (scenarios Cargo.toml, site_assessment.rs, progress.rs)
- Total: 11 files
