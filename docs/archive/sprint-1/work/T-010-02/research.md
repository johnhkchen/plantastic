# T-010-02 Research: pt-climate Engine

## What Exists

### No climate crate yet
No `crates/pt-climate/` directory. No climate-related types in any existing crate. No climate/weather crates in Cargo.lock.

### pt-solar (sibling crate, T-010-01)
Closest analogue — pure computation, embedded data, no I/O. Pattern to follow:
- `types.rs`: all public types + constants
- Module-per-concern: `position.rs`, `sun_hours.rs`, `seasonal.rs`, `classification.rs`, `grid.rs`
- Uses `Coordinates { latitude, longitude }` (re-export candidate for pt-climate)
- Tests use `pt_test_utils::timed()`, `approx` crate for float comparisons
- 22 tests, all <30ms

### Prior art: solar-sim TypeScript prototype
`/Volumes/ext1/swe/repos/solar-sim/src/lib/climate/` contains proven algorithms:

**frost-dates.ts**: Latitude-band lookup table (2.5-degree bands, 20N-70N + 60S-20S). Each entry: `{ latMin, latMax, coastalModifier, lastSpringDoy, firstFallDoy, varianceDays }`. Elevation adjustment: +4 days later spring frost per 300m. Coastal modifier reduces frost risk. Returns `DayOfYearRange { early, median, late }` for both spring and fall. Tropical (lat < 20): minimal/low-confidence frost dates.

**hardiness-zone.ts**: Latitude-based minimum winter temperature estimation. Maps to USDA zones (1a-13b). Coastal locations get +1 subzone warming. Returns `HardinessZone { zone, zoneNumber, subzone, minTempF, maxTempF }`.

**koppen.ts**: Köppen climate classification from monthly data. Complex — likely out of scope for V1.

**openmeteo.ts**: Network API calls. Explicitly excluded (acceptance criteria says "no network I/O at query time").

### Specification requirements
- Plant scoring weights: light requirements 50%, **season length 30%**, **climate zone 20%**
- Sunset Western Garden zones are the authority for Bay Area market
- Plant schema includes `frost_tolerance` (tender/semi-hardy/hardy) and `sunset_zones INT[]`
- pt-plants will consume pt-climate + pt-solar for scoring

### Scenario connections
- **S.2.3** (plant recommendations, 20 min savings): requires pt-plants, pt-solar, **pt-climate**, BAML AI layer. Currently NotImplemented.
- pt-climate is a prerequisite but doesn't directly flip any scenario alone.

### Testing context
- Workspace: 84 tests pass, 20 ignored. `just check` green.
- No climate milestone exists in progress.rs yet — needs to be added.
- Test data: SF (37.7749, -122.4194), Oakland (37.8044, -122.2712), San Jose (37.3382, -121.8863) per acceptance criteria.

## Key Domain Concepts

### Frost Dates
Day-of-year (1-366) for last spring frost and first fall frost. Varies by latitude, elevation, coastal proximity. Expressed as ranges (early/median/late) to capture year-to-year variance.

### Sunset Western Garden Zones
California-specific microclimate zones (1-24). More granular than USDA. Bay Area spans zones 14-17 primarily. Based on winter lows, summer highs, humidity, wind patterns. No public API — must be embedded data.

### USDA Hardiness Zones
National standard (1a-13b) based on average annual minimum winter temperature. Each zone spans 10F, each subzone 5F. Zone 9b-10b covers most of Bay Area.

### Growing Season
Calendar days between last spring frost and first fall frost. Bay Area: 250-365 days depending on microclimate. Drives `days_to_maturity` compatibility for annual plants.

## Constraints

1. **Pure computation**: no I/O, no network. All data embedded at compile time.
2. **Sunset zones require spatial data**: zone boundaries are polygons, not simple lat/lng ranges. For V1, a lookup table mapping Bay Area lat/lng rectangles to zones is acceptable. Full polygon boundaries would require a shapefile dependency.
3. **Bay Area focus**: V1 targets SF Bay Area. Coverage outside Bay Area is best-effort from latitude-band tables. Sunset zones are only defined for western US.
4. **chrono dependency**: for day-of-year ↔ date conversions.
5. **No external data fetches**: unlike solar-sim's openmeteo integration.

## Existing Patterns to Follow

- Cargo.toml: workspace edition/license/rust-version/lints, workspace deps, dev-deps include pt-test-utils + approx
- lib.rs: module declarations + re-exports
- types.rs: all public types, serde derives, doc comments
- Tests: `#[cfg(test)] mod tests` in each module, use `timed()`
- Error handling: thiserror enum if fallible, plain returns if infallible
