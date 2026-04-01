# Research — T-002-01 pt-project & pt-materials

## What Exists

### Workspace Layout

Rust workspace at `/Cargo.toml` with `members = ["crates/*", "tests/scenarios"]`.
Five crates exist: pt-geo (complete), pt-project (stub), pt-materials (stub),
pt-quote (stub), pt-test-utils (complete). Workspace dependencies centralize
geo, geojson, serde, serde_json, uuid, chrono, rust_decimal.

### pt-geo (Complete — T-001-02)

Pure free-function API over the `geo` crate. Provides:
- `area::area_sqft`, `area::multi_area_sqft`
- `perimeter::perimeter_ft`, `perimeter::multi_perimeter_ft`
- `volume::volume_cuft`, `volume::volume_cuyd`
- `boolean::union`, `boolean::difference` (+ multi variants)
- `simplify::simplify`, `simplify::simplify_multi`

Re-exports: `Polygon<f64>`, `MultiPolygon<f64>`, `LineString`, `Coord`, `coord!`, `polygon!`.

Patterns established:
- Module-per-concern (~30 lines each)
- Concrete f64, no generics
- Inline `#[cfg(test)]` modules with known-value assertions
- No Result types; degenerate inputs → defined outputs (0.0)
- `approx` for float comparison in tests

### pt-project Stub

`Cargo.toml` already declares deps: geo, geojson, serde, serde_json, uuid, chrono.
`src/lib.rs` is a single doc comment line. No types, no modules.

### pt-materials Stub

`Cargo.toml` already declares deps: serde, uuid, rust_decimal.
`src/lib.rs` is a single doc comment line. No types, no modules.

### pt-quote Stub

Empty — commented-out deps, single doc comment. Will consume pt-project and
pt-materials (T-002-02). Its existence informs our API design: pt-quote will
iterate zones, look up materials, compute quantities via pt-geo, multiply prices.

### pt-test-utils

Provides `timed()` and `run_with_timeout()` for enforcing 10-second test ceiling.
Available as a dev-dependency if needed.

### Scenario Harness

17 scenarios, all `NotImplemented`. Relevant to this ticket:
- **S.3.1** — Quantity computation from geometry (25 min). Needs pt-project zones
  with known geometry + pt-materials with known prices. Expected values computed
  independently in the test.
- **S.3.2** — Three-tier quote generation (15 min). Three tiers with different
  materials, Good < Better < Best assertion.

Both scenarios require pt-quote (T-002-02), not this ticket. But we must design
types that S.3.1 and S.3.2 can construct and use.

## Specification Constraints

### Project Model (spec §4)

```
Project
├── ScanRef (PLY + terrain.glb + planview.png + metadata.json)
├── Baseline (lot polygon, trees[], sun exposure grid)
├── Zone[] (id, geometry, zone_type, label)
├── Tier[3] (good, better, best)
│   └── MaterialAssignment[] (zone_id, material_id, overrides)
└── Status (draft → quoted → approved → complete)
```

Ticket scope: Project, Zone, Tier, MaterialAssignment, Status. ScanRef and Baseline
are out of scope (future tickets for scan pipeline and satellite pre-pop).

### Material Model (spec §6)

Material: name, category (hardscape/softscape/edging/fill), unit (sq_ft/cu_yd/
linear_ft/each), price_per_unit, depth_inches, texture_ref, photo_ref, supplier_sku,
extrusion_behavior (SitsOnTop/Fills/BuildsUp).

Per-tenant catalog layered on platform starter set. Tenant ownership is out of scope
for this ticket (T-003 database layer), but MaterialId and the struct shape must
support it.

### GeoJSON Serialization

Canonical format for Project. Zone geometry stored as GeoJSON Feature with properties
for zone_type, label, id. Project is a FeatureCollection. Round-trip must be lossless.

### Quote Engine Integration (spec §7)

pt-quote walks zones, computes quantity per zone based on unit type:
- sq_ft → `area_sqft(zone.geometry)`
- cu_yd → `volume_cuyd(area_sqft(zone.geometry), depth_inches / 12.0)`
- linear_ft → `perimeter_ft(zone.geometry)`
- each → count (1 per assignment)

This means pt-materials `Unit` enum must directly map to pt-geo functions, and
zones must expose their `geometry` as `&Polygon<f64>`.

## Dependency Graph

```
pt-geo (done) ← pt-project (this ticket)
                     ↑
                 pt-quote (T-002-02)
                     ↑
               pt-materials (this ticket)
```

pt-project depends on pt-geo for Polygon re-exports. pt-materials is standalone
(no geo dependency). pt-quote will depend on both plus pt-geo.

## Shared Boundary Types

MaterialId and ZoneId cross crate boundaries. Options:
1. Each crate defines its own Id type (uuid newtype)
2. A shared pt-types crate
3. Plain `Uuid` with type aliases

The ticket says they "share boundary types (MaterialId, ZoneId)". Since these are
just `Uuid` newtypes used for type safety, each crate can define its own. pt-quote
will import both. No shared crate needed — it would be premature.

## Existing Patterns to Follow

- Workspace `.workspace = true` for deps
- Module-per-concern in lib.rs
- Inline `#[cfg(test)]` modules
- Doc comments on all public items
- Known-value test assertions (independently computed)
- No `#[ignore]` without scenario ID
- No mocking across crate boundaries

## Key Constraints

1. **Decimal for money**: `rust_decimal::Decimal` for prices. Float-point drift in
   currency is unacceptable. Workspace already provides `rust_decimal` with
   `serde-with-str` feature.

2. **GeoJSON round-trip**: The `geojson` crate (v0.24) provides `TryFrom<geojson::Value>`
   for `geo::Polygon<f64>`. We need custom serialization for the full Project because
   zone properties (type, label, id) go in Feature properties, not geometry.

3. **Three tiers**: The spec says exactly three (good/better/best). The data model should
   enforce this (fixed array or named struct, not unbounded Vec).

4. **Extrusion behavior**: This determines how pt-quote computes quantity AND how the
   3D scene generator extrudes geometry. The enum must carry enough data for both.

5. **depth_inches on Material**: Used by pt-quote to compute volume. Stored in inches
   (industry convention), converted to feet for pt-geo::volume_cuyd.

## Assumptions

- ScanRef and Baseline are placeholder fields for now (Option or omitted)
- Tenant ownership (tenant_id on Material) is deferred to T-003
- No database persistence in this ticket — pure domain types with serde
- Zone CRUD means add/remove/update zones on a Project in memory
- Status transitions should be validated (draft → quoted, not draft → complete)
