# T-010-02 Structure: pt-climate Engine

## New Crate: `crates/pt-climate/`

### Module Layout

```
crates/pt-climate/
├── Cargo.toml
└── src/
    ├── lib.rs              # Module declarations + re-exports
    ├── types.rs            # All public types + constants
    ├── frost.rs            # Frost date lookup: latitude-band tables, elevation/coastal modifiers
    ├── hardiness.rs        # USDA hardiness zone lookup from latitude + modifiers
    ├── sunset.rs           # Sunset Western Garden zone lookup (Bay Area bounding-box table)
    └── growing_season.rs   # Growing season computation from frost dates
```

## File Details

### `Cargo.toml`
```toml
[package]
name = "pt-climate"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
chrono.workspace = true
serde.workspace = true

[dev-dependencies]
pt-test-utils = { path = "../pt-test-utils" }

[lints]
workspace = true
```

Minimal deps: chrono for day-of-year ↔ date conversions, serde for serialization.

### `lib.rs`
Module declarations + re-export key types and functions at crate root:
- `ClimateProfile`, `FrostDates`, `HardinessZone`, `SunsetZone`, `GrowingSeason`
- `frost_dates()`, `hardiness_zone()`, `sunset_zone()`, `growing_season()`, `climate_profile()`
- `Coordinates`, `Confidence`, `DayOfYearRange`

### `types.rs`
All public types:
- `Coordinates { latitude: f64, longitude: f64 }` — serde derives
- `DayOfYearRange { early: u16, median: u16, late: u16 }` — represents variance
- `Confidence { High, Medium, Low }` — data quality indicator
- `FrostDates { last_spring_frost, first_fall_frost, confidence }`
- `HardinessZone { zone: u8, subzone: char, min_temp_f: f64, max_temp_f: f64 }`
- `SunsetZone { zone: u8, description: &'static str }`
- `GrowingSeason { typical_days, short_days, long_days, frost_free_start, frost_free_end }`
- `ClimateProfile { frost_dates, hardiness_zone, sunset_zone: Option, growing_season }`

Internal types (not pub):
- `FrostLookupEntry { lat_min, lat_max, coastal_modifier_days, last_spring_doy, first_fall_doy, variance_days }` — in frost.rs

### `frost.rs`
Public API:
- `pub fn frost_dates(coords: &Coordinates, elevation_m: Option<f64>, coastal: bool) -> FrostDates`

Internals:
- `NORTHERN_FROST_TABLE: &[FrostLookupEntry]` — const array, 2.5-degree bands from 20N to 70N
- `SOUTHERN_FROST_TABLE: &[FrostLookupEntry]` — const array, 2.5-degree bands from 60S to 20S
- Latitude band matching → base frost dates
- Elevation adjustment: +4 days per 300m (later spring, earlier fall)
- Coastal adjustment: apply coastal_modifier_days (earlier spring, later fall)
- Tropical fallback (|lat| < 20): minimal frost dates, Low confidence
- Out-of-range (|lat| > 70): extreme dates, Low confidence

Tests: SF, Oakland, San Jose frost dates against known horticulture references.

### `hardiness.rs`
Public API:
- `pub fn hardiness_zone(coords: &Coordinates, coastal: bool) -> HardinessZone`

Internals:
- Minimum winter temperature estimation from latitude
- Zone calculation: (min_temp_f + 60) / 10, clamped to 1-13
- Subzone: 'a' if in lower half of 10F range, 'b' if upper
- Coastal modifier: +5F to min temp (warmer near coast)
- Formula validated against known Bay Area zones (SF=10b, Oakland=10a, San Jose=9b)

Tests: Known Bay Area locations with expected zones.

### `sunset.rs`
Public API:
- `pub fn sunset_zone(coords: &Coordinates) -> Option<SunsetZone>`

Internals:
- `BAY_AREA_ZONES: &[SunsetZoneEntry]` — bounding boxes mapping to zones 14-17
  - Zone 14: inland valleys (e.g., Livermore, Concord)
  - Zone 15: inland hills/urban (e.g., San Jose, inland East Bay)
  - Zone 16: coastal influenced (e.g., Oakland, Berkeley, SF south)
  - Zone 17: coastal/ocean-moderated (e.g., SF, Pacifica, Half Moon Bay)
- Point-in-bounding-box test, ordered from most specific to least
- Returns None for locations outside Bay Area coverage

Tests: Known city → zone mappings for Bay Area.

### `growing_season.rs`
Public API:
- `pub fn growing_season(frost_dates: &FrostDates) -> GrowingSeason`

Internals:
- `typical_days = first_fall_frost.median - last_spring_frost.median` (handles year wraparound for southern hemisphere)
- `short_days = first_fall_frost.early - last_spring_frost.late`
- `long_days = first_fall_frost.late - last_spring_frost.early`
- `frost_free_start = last_spring_frost.median`
- `frost_free_end = first_fall_frost.median`

Tests: Computed growing season for Bay Area locations matches known ~280-365 day range.

## Convenience Function

`climate_profile()` in lib.rs: calls all four functions, assembles `ClimateProfile`.
```rust
pub fn climate_profile(coords: &Coordinates, elevation_m: Option<f64>, coastal: bool) -> ClimateProfile
```

## Workspace Changes

### `Cargo.toml` (workspace root)
No new workspace dependencies needed (chrono and serde already in workspace).

### `tests/scenarios/src/progress.rs`
Add milestone:
```rust
Milestone {
    label: "pt-climate: frost dates, hardiness zones, growing season",
    delivered_by: Some("T-010-02"),
    unlocks: &["S.2.3"],
    note: "...",
}
```

## Ordering Constraints

1. types.rs first (all other modules depend on types)
2. frost.rs (independent)
3. hardiness.rs (independent)
4. sunset.rs (independent)
5. growing_season.rs (depends on FrostDates from types, but not on frost.rs)
6. lib.rs (ties modules together, adds climate_profile())
7. Milestone claim last
