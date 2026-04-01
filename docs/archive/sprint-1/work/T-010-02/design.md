# T-010-02 Design: pt-climate Engine

## Decision 1: Data Embedding Strategy

### Options
**A) Latitude-band lookup tables (embedded const arrays)**: Port the solar-sim frost-date tables. 2.5-degree bands with coastal/elevation modifiers. Compile-time constant. No files, no parsing, no I/O.
**B) Embedded JSON/CSV files via include_str!**: More data-friendly but adds parsing at runtime.
**C) SQLite bundled database**: Overkill for static lookup tables.

### Decision: Option A — const arrays
Matches pt-solar pattern (constants in types.rs). Frost date tables are ~20 entries for northern hemisphere. USDA zones are a simple temperature formula. Sunset zones need a small Bay Area lookup table (~10 entries). All fit cleanly in const arrays.

## Decision 2: Sunset Zone Implementation

### Options
**A) Full polygon boundaries from shapefile**: Accurate but requires a shapefile dependency, large embedded data, and spatial intersection logic.
**B) Lat/lng bounding-box lookup table for Bay Area**: A table of ~10 rectangular regions mapping to Sunset zones 14-17. Approximate but covers the target market.
**C) Nearest-city lookup**: Map to known reference cities. Coarse.

### Decision: Option B — bounding-box lookup for Bay Area
The spec says "Sunset Western Garden zones are the authority for the Bay Area market." Full polygon data is a V2 concern. For V1, a lookup table of Bay Area subregions mapping to zones 14-17 covers the target market. Locations outside Bay Area return None (no Sunset zone applicable or not enough data). This is honest about coverage rather than guessing.

## Decision 3: Type Design

### Decision: Rust-idiomatic types with serde

```rust
// Day-of-year range for frost dates (captures variance)
pub struct DayOfYearRange { pub early: u16, pub median: u16, pub late: u16 }

// Complete frost date pair for a location
pub struct FrostDates {
    pub last_spring_frost: DayOfYearRange,
    pub first_fall_frost: DayOfYearRange,
    pub confidence: Confidence,
}

pub enum Confidence { High, Medium, Low }

// USDA hardiness zone
pub struct HardinessZone {
    pub zone: u8,        // 1-13
    pub subzone: char,   // 'a' or 'b'
    pub min_temp_f: f64, // avg annual min
    pub max_temp_f: f64,
}

// Sunset Western Garden zone (Bay Area specific)
pub struct SunsetZone {
    pub zone: u8,  // 14-17 for Bay Area
    pub description: &'static str,
}

// Growing season computed from frost dates
pub struct GrowingSeason {
    pub typical_days: u16,        // median-to-median
    pub short_days: u16,          // late spring to early fall
    pub long_days: u16,           // early spring to late fall
    pub frost_free_start: u16,    // day-of-year
    pub frost_free_end: u16,      // day-of-year
}

// Full climate profile for a location
pub struct ClimateProfile {
    pub frost_dates: FrostDates,
    pub hardiness_zone: HardinessZone,
    pub sunset_zone: Option<SunsetZone>,
    pub growing_season: GrowingSeason,
}
```

## Decision 4: Coordinates Type

### Options
**A) Define own Coordinates struct**: Duplicates pt-solar's identical type.
**B) Re-export from pt-solar**: Creates a dependency on pt-solar.
**C) Accept (f64, f64)**: Less ergonomic but no coupling.
**D) Define own, identical struct**: Both crates have their own. pt-plants (consumer of both) converts as needed.

### Decision: Option D — own Coordinates struct
pt-climate and pt-solar are sibling crates, not parent-child. Both will be consumed by pt-plants. Defining an identical `Coordinates { latitude, longitude }` in both is cleaner than coupling. If this becomes a pattern problem, a shared `pt-types` crate can be extracted later (YAGNI for now).

## Decision 5: Error Handling

### Decision: No error type needed
All functions are infallible. Lookup tables always return a result (worst case: low-confidence fallback for out-of-range latitudes). Sunset zone returns Option (None outside Bay Area). No I/O, no parsing, no failure paths.

## Decision 6: Elevation and Coastal Modifiers

### Decision: Follow solar-sim algorithm
- Frost dates: +4 days per 300m elevation (later spring frost, earlier fall frost)
- Coastal modifier: reduces frost risk (earlier spring, later fall) by variance amount
- Hardiness zone: coastal locations get +1 subzone warming
- Accept elevation_m: Option<f64> and coastal: bool as parameters
- Default: elevation 0m, not coastal (conservative)

## What Was Rejected

- **Köppen climate classification**: Complex monthly data requirement. Not needed for V1 plant scoring (Sunset zones serve this purpose for Bay Area).
- **Network API calls (Open-Meteo)**: Explicitly excluded by acceptance criteria.
- **Full polygon Sunset zone data**: Large data, shapefile parsing dependency. V2 concern.
- **Shared Coordinates crate**: Premature abstraction. Two identical structs is fine.
- **Result return types**: No failure paths in pure lookup/computation functions.
