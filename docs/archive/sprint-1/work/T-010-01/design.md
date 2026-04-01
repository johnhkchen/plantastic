# Design — T-010-01: pt-solar engine

## Decision: Direct NOAA algorithm implementation, mirroring prototype architecture

### Options considered

**Option A: Port SunCalc.js to Rust**
Pros: Exact algorithm match, easy validation. Cons: SunCalc is MIT-licensed but porting means maintaining a fork of someone else's design decisions. SunCalc has browser-specific assumptions (Date object behavior).

**Option B: Use a Rust solar position crate (e.g., `sun` or `solar-calc`)**
Pros: Less code to write. Cons: These crates have different APIs, different accuracy profiles, and add a dependency we'd need to audit. The `sun` crate doesn't handle polar conditions. None match SunCalc's specific algorithm, making validation against the prototype harder.

**Option C: Implement NOAA solar position formulas directly** (chosen)
Pros: ~50 lines of trig, zero dependencies beyond std, exact control over accuracy, can validate against both SunCalc and timeanddate.com. Cons: Must get the math right (but it's well-documented and the prototype provides reference outputs).

### Why Option C

The solar position calculation is a small, well-understood algorithm. The NOAA formulas are public domain and documented at the NOAA Solar Calculator. SunCalc itself is just a wrapper around these formulas. Implementing directly gives us:
- No dependency risk
- WASM-compatible (no system dependencies)
- Exact control for future optimizations (SIMD, lookup tables)
- Easy to validate: same inputs should give same outputs as SunCalc

### Architecture decisions

**1. Module layout mirrors the prototype's logical separation:**
- `position.rs` — sun position (altitude, azimuth) for a point in time
- `sun_hours.rs` — daily sun hours integration via 5-minute sampling
- `seasonal.rs` — date range aggregation (daily, monthly, annual)
- `classification.rs` — light category from sun hours
- `grid.rs` — spatial grid of sun hours over a bounding box
- `types.rs` — shared types (Coordinates, SolarPosition, etc.)

**2. Types design:**
- `Coordinates { latitude: f64, longitude: f64 }` — simple struct, not geo::Coord (which uses x/y semantics that confuse lat/lng)
- `SolarPosition { altitude_degrees: f64, azimuth_degrees: f64 }` — explicit units in field names
- `LightCategory` enum — `FullSun`, `PartSun`, `PartShade`, `FullShade` (Rust naming, serde maps to snake_case)
- `PolarCondition` enum — `Normal`, `MidnightSun`, `PolarNight`
- `ExposureGrid` — struct with `Vec<f32>` values (not Float32Array but same idea), dimensions, bounds, metadata

**3. No shade/tree integration:**
The prototype's exposure-grid.ts has extensive tree shadow logic. T-010-01 only needs theoretical sun hours (no obstacles). The grid API will accept bounds + date range + config and return sun hours per cell. Shade integration comes later with pt-satellite/pt-scene.

**4. Sample-based integration (not analytical):**
The prototype's 5-minute sampling approach is proven: 288 samples/day, count positive altitude samples, convert to hours. This is simple, accurate to within ~2.5 minutes, and fast enough (<2ms/year in JS). In Rust it will be even faster. No reason to use a more complex analytical approach.

**5. Grid sampling strategy:**
Follow prototype: sample N representative days evenly distributed across the date range (default 12), not every day. Average the results. This gives accurate seasonal patterns while keeping grid computation fast.

**6. Performance budget:**
- `sun_position()`: ~100ns (just trig)
- `daily_sun_hours()`: ~30us (288 position calculations)
- `annual_sun_hours()`: ~10ms (365 days)
- `radiance_grid()` 20x20 at 12 sample days: ~150ms
- All well within the ticket's targets (5ms single point, 500ms grid)

### What was rejected

- **Analytical day-length formulas** — faster but less accurate at extreme latitudes and don't generalize to shade integration later.
- **Pre-computed lookup tables** — unnecessary complexity; raw computation is fast enough.
- **Async grid computation** — no need in Rust; this is CPU-bound work that completes in milliseconds.
- **`rust_decimal` for sun hours** — these are physical measurements, not monetary values. f64 is appropriate.
- **Separate `error.rs`** — this crate is pure math. Invalid inputs (lat > 90) return f64::NAN or are clamped. No Result types needed for the core API. Grid config validation can use simple panics for obviously invalid inputs (resolution <= 0).

### Scenario S.1.3 implementation plan

The scenario test will:
1. Create a grid over a known SF Bay Area location (37.7749, -122.4194)
2. Compute sun hours for a growing season (March-September)
3. Verify: average sun hours per cell is between 8-16 hours (Bay Area typical)
4. Verify: every cell has a valid light classification
5. Verify: grid dimensions match expected cell count for the bounds/resolution
6. Return `ScenarioOutcome::Pass(Integration::OneStar)`
