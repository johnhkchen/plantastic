# Structure — T-002-01 pt-project & pt-materials

## File Changes

### Modified Files

#### `crates/pt-materials/Cargo.toml`

Add serde_json as dev-dependency for round-trip tests. Current deps (serde, uuid,
rust_decimal) are correct.

```toml
[dependencies]
serde.workspace = true
uuid.workspace = true
rust_decimal.workspace = true

[dev-dependencies]
serde_json.workspace = true
```

#### `crates/pt-project/Cargo.toml`

Add pt-materials dependency (for MaterialId). Add serde_json as dev-dependency
for tests. Add pt-geo for Polygon re-export.

```toml
[dependencies]
geo.workspace = true
geojson.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
pt-materials = { path = "../pt-materials" }
pt-geo = { path = "../pt-geo" }

[dev-dependencies]
# serde_json already in deps (needed for geojson conversion)
```

Note: serde_json is a regular dependency for pt-project because the geojson
conversion uses serde_json::Value for Feature properties.

#### `crates/pt-materials/src/lib.rs`

Replace stub with module declarations and re-exports.

#### `crates/pt-project/src/lib.rs`

Replace stub with module declarations and re-exports.

### New Files

#### `crates/pt-materials/src/types.rs`

Core type definitions.

```rust
// MaterialId — newtype over Uuid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialId(pub Uuid);

// MaterialCategory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialCategory { Hardscape, Softscape, Edging, Fill }

// Unit — determines how pt-quote computes quantity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit { SqFt, CuYd, LinearFt, Each }

// ExtrusionBehavior — how the material renders in 3D
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtrusionBehavior {
    SitsOnTop { height_inches: f64 },
    Fills { flush: bool },
    BuildsUp { height_inches: f64 },
}

// Material
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub id: MaterialId,
    pub name: String,
    pub category: MaterialCategory,
    pub unit: Unit,
    pub price_per_unit: Decimal,
    pub depth_inches: Option<f64>,
    pub texture_ref: Option<String>,
    pub photo_ref: Option<String>,
    pub supplier_sku: Option<String>,
    pub extrusion: ExtrusionBehavior,
}
```

Tests (inline `#[cfg(test)]`):
- MaterialId: new creates unique, equality, hash
- MaterialCategory: serde round-trip all variants
- Unit: serde round-trip all variants
- ExtrusionBehavior: serde round-trip each variant with data
- Material: full serde round-trip with all fields

#### `crates/pt-materials/src/builder.rs`

MaterialBuilder with method chaining.

```rust
pub struct MaterialBuilder { /* fields mirroring Material, most Option */ }

impl Material {
    pub fn builder(name: impl Into<String>, category: MaterialCategory) -> MaterialBuilder
}

impl MaterialBuilder {
    pub fn id(mut self, id: MaterialId) -> Self
    pub fn unit(mut self, unit: Unit) -> Self
    pub fn price_per_unit(mut self, price: Decimal) -> Self
    pub fn depth_inches(mut self, depth: f64) -> Self
    pub fn texture_ref(mut self, texture: impl Into<String>) -> Self
    pub fn photo_ref(mut self, photo: impl Into<String>) -> Self
    pub fn supplier_sku(mut self, sku: impl Into<String>) -> Self
    pub fn extrusion(mut self, behavior: ExtrusionBehavior) -> Self
    pub fn build(self) -> Material  // panics only if invariants broken
}
```

Defaults:
- id: `MaterialId::new()` (random UUID)
- unit: `Unit::SqFt`
- price_per_unit: `Decimal::ZERO`
- depth_inches: `None`
- texture_ref, photo_ref, supplier_sku: `None`
- extrusion: `ExtrusionBehavior::SitsOnTop { height_inches: 1.0 }`

Tests:
- Builder with required fields only → valid Material
- Builder with all fields → all values correct
- Builder default values are sensible

#### `crates/pt-project/src/error.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectError {
    InvalidStatusTransition { from: ProjectStatus, to: ProjectStatus },
    ZoneNotFound(ZoneId),
    DuplicateZone(ZoneId),
    GeoJsonConversion(String),
    InvalidTierCount { expected: usize, got: usize },
}

impl std::fmt::Display for ProjectError { ... }
impl std::error::Error for ProjectError {}
```

#### `crates/pt-project/src/types.rs`

Core type definitions.

```rust
// ZoneId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneId(pub Uuid);

// ZoneType
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneType { Bed, Patio, Path, Lawn, Wall, Edging }

// Zone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub id: ZoneId,
    #[serde(with = "geojson_polygon")]  // custom serde for geo::Polygon
    pub geometry: Polygon<f64>,
    pub zone_type: ZoneType,
    pub label: Option<String>,
}

// TierLevel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierLevel { Good, Better, Best }

// MaterialAssignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialAssignment {
    pub zone_id: ZoneId,
    pub material_id: MaterialId,
    pub overrides: Option<AssignmentOverrides>,
}

// AssignmentOverrides — optional per-assignment tweaks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentOverrides {
    pub price_override: Option<Decimal>,
    pub depth_override_inches: Option<f64>,
}

// Tier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tier {
    pub level: TierLevel,
    pub assignments: Vec<MaterialAssignment>,
}

// ProjectStatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus { Draft, Quoted, Approved, Complete }
```

#### `crates/pt-project/src/serde_helpers.rs`

Custom serde module for `geo::Polygon<f64>` ↔ GeoJSON geometry within normal
JSON serialization. Uses `geojson::Value` as intermediate.

```rust
pub mod geojson_polygon {
    pub fn serialize<S>(polygon: &Polygon<f64>, serializer: S) -> Result<S::Ok, S::Error>
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Polygon<f64>, D::Error>
}
```

This lets Zone derive Serialize/Deserialize normally while encoding geometry as
GeoJSON within the JSON object.

#### `crates/pt-project/src/project.rs`

Project struct and methods.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub scan_ref: Option<String>,
    pub zones: Vec<Zone>,
    pub tiers: Vec<Tier>,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new() -> Self  // empty draft project with 3 default tiers
    pub fn with_id(id: Uuid) -> Self

    // Zone CRUD
    pub fn add_zone(&mut self, zone: Zone) -> Result<(), ProjectError>
    pub fn remove_zone(&mut self, id: ZoneId) -> Result<Zone, ProjectError>
    pub fn get_zone(&self, id: ZoneId) -> Option<&Zone>
    pub fn get_zone_mut(&mut self, id: ZoneId) -> Option<&mut Zone>

    // Status
    pub fn transition_to(&mut self, status: ProjectStatus) -> Result<(), ProjectError>

    // Tier access
    pub fn tier(&self, level: TierLevel) -> &Tier
    pub fn tier_mut(&mut self, level: TierLevel) -> &mut Tier
}
```

Tests:
- new() creates project with 3 tiers (Good, Better, Best), Draft status
- add_zone + get_zone round-trip
- add duplicate zone → DuplicateZone error
- remove_zone → returns zone, subsequent get returns None
- remove nonexistent → ZoneNotFound error
- Status: Draft→Quoted ok, Draft→Complete err, Quoted→Approved ok
- Any→Draft ok (reset)
- tier(Good) returns the Good tier

#### `crates/pt-project/src/geojson.rs`

GeoJSON conversion (separate from serde — this is FeatureCollection format).

```rust
impl Project {
    pub fn to_geojson(&self) -> geojson::GeoJson
    pub fn from_geojson(geojson: &geojson::GeoJson) -> Result<Self, ProjectError>
}
```

Each Zone → Feature with:
- geometry: zone polygon as GeoJSON geometry
- properties: { "id": zone_id, "zone_type": "patio", "label": "Back patio" }

Project metadata in FeatureCollection foreign members:
- project_id, status, scan_ref, tiers, timestamps

Tests:
- Empty project round-trip
- Project with 2 zones of different types → GeoJSON → Project
- Geometry coordinates preserved exactly
- Zone properties preserved
- Tiers and status preserved

## Module Boundaries

### pt-materials
```
lib.rs           — pub mod types; pub mod builder; re-exports
├── types.rs     — MaterialId, Material, MaterialCategory, Unit, ExtrusionBehavior
└── builder.rs   — MaterialBuilder
```

Public API:
- `MaterialId`, `Material`, `MaterialCategory`, `Unit`, `ExtrusionBehavior`
- `Material::builder(name, category) -> MaterialBuilder`
- `MaterialBuilder::build() -> Material`

### pt-project
```
lib.rs              — pub mod; re-exports
├── types.rs        — ZoneId, Zone, ZoneType, Tier, TierLevel, MaterialAssignment,
│                     AssignmentOverrides, ProjectStatus
├── project.rs      — Project struct + methods
├── geojson.rs      — GeoJSON conversion impl
├── serde_helpers.rs — geojson_polygon serde module
└── error.rs        — ProjectError
```

Public API:
- All types from types.rs
- `Project` with CRUD and transition methods
- `Project::to_geojson()`, `Project::from_geojson()`
- `ProjectError`

## Dependency Graph (after changes)

```
geo ←── pt-geo ←── pt-project ──→ pt-materials
 │                    │                │
 │                    ├── geojson      ├── rust_decimal
 │                    ├── chrono       └── uuid
 │                    ├── uuid
 │                    └── serde/serde_json
 │
 └── (re-exported types: Polygon, Coord, etc.)
```

## Ordering

1. pt-materials first (no internal dependencies)
2. pt-project second (depends on pt-materials for MaterialId)
3. GeoJSON conversion last (depends on all types being defined)
