# Design — T-002-01 pt-project & pt-materials

## Problem

Define the core domain types that every other component operates on. pt-project owns
the Project/Zone/Tier/MaterialAssignment model with GeoJSON serialization. pt-materials
owns Material/Category/Unit/ExtrusionBehavior with builder pattern. Types must be
ergonomic for pt-quote to consume and for scenario tests to construct.

## Decision 1: ID Types

### Option A: Plain `Uuid` everywhere
- Pro: Simple, no wrapper boilerplate
- Con: `fn assign(zone: Uuid, material: Uuid)` — easy to swap arguments silently

### Option B: Newtype wrappers per crate ✓
```rust
// pt-project
pub struct ZoneId(pub Uuid);
// pt-materials
pub struct MaterialId(pub Uuid);
```
- Pro: Type safety prevents mixing zone and material IDs at compile time
- Con: Minor boilerplate (derive Serde, Display, Hash, Eq)
- Con: pt-quote must import both crate's ID types

**Selected**: Type safety is worth the boilerplate. A `ZoneId`/`MaterialId` mix-up
would be a silent logic bug — exactly what types should prevent.

## Decision 2: Tier Representation

### Option A: Fixed struct with named fields
```rust
pub struct Tiers {
    pub good: Tier,
    pub better: Tier,
    pub best: Tier,
}
```
- Pro: Enforces exactly 3, named access
- Con: Can't iterate without macro/array conversion; harder to generalize

### Option B: `[Tier; 3]` with TierLevel enum ✓
```rust
pub enum TierLevel { Good, Better, Best }
pub struct Tier {
    pub level: TierLevel,
    pub assignments: Vec<MaterialAssignment>,
}
// Project stores Vec<Tier> but constructors/validators ensure exactly 3
```
- Pro: Iterable, enum is indexable, flexible for pt-quote iteration
- Con: Runtime check for length 3 instead of compile-time

### Option C: Vec<Tier> unconstrained
- Pro: Maximum flexibility
- Con: Violates spec (exactly three tiers), no compile-time or construction-time safety

**Selected**: Option B. TierLevel enum + Vec<Tier> with a constructor that enforces
3 tiers. Iteration is critical for pt-quote (walk each tier, compute quote). The
enum carries semantic meaning for display ("Good", "Better", "Best").

## Decision 3: GeoJSON Serialization Strategy

### Option A: Manual geojson crate construction
```rust
fn to_geojson(project: &Project) -> geojson::FeatureCollection { ... }
fn from_geojson(fc: &geojson::FeatureCollection) -> Result<Project, Error> { ... }
```
- Pro: Full control over Feature properties, explicit mapping
- Con: Verbose, error-prone manual JSON property extraction

### Option B: Serde with custom Serialize/Deserialize
- Pro: Integrates with serde ecosystem
- Con: GeoJSON is not a simple JSON shape — zones have geometry + properties,
  project is a FeatureCollection. Custom serde impls would be complex.

### Option C: Dedicated conversion functions with geojson crate types ✓
```rust
impl Project {
    pub fn to_geojson(&self) -> geojson::GeoJson { ... }
    pub fn from_geojson(geojson: &geojson::GeoJson) -> Result<Self, ProjectError> { ... }
}
```
- Pro: Methods on Project, uses geojson crate types, explicit error handling
- Con: Not transparent serde (callers must call conversion explicitly)
- Mitigation: Also derive Serde on Project for normal JSON serialization; GeoJSON
  is a separate format used for specific interchange (PostGIS, frontend map)

**Selected**: Option C. GeoJSON is a domain-specific serialization format, not the
default JSON representation. Project gets both `#[derive(Serialize, Deserialize)]`
for normal JSON and explicit `to_geojson()`/`from_geojson()` for GeoJSON format.

## Decision 4: Geometry in Zone

### Option A: Zone stores `geo::Polygon<f64>` directly ✓
```rust
pub struct Zone {
    pub id: ZoneId,
    pub geometry: Polygon<f64>,
    pub zone_type: ZoneType,
    pub label: Option<String>,
}
```
- Pro: Direct access for pt-geo functions, no conversion
- Con: Serde for geo::Polygon requires custom impl or geojson conversion

### Option B: Zone stores GeoJSON string, parses on demand
- Pro: Cheap to store
- Con: Parse cost on every access, error handling on every geometry use

**Selected**: Option A. Zone stores `Polygon<f64>` directly. For regular JSON serde,
we'll use the geojson crate's `From<Polygon<f64>>` to serialize geometry as GeoJSON
within the JSON representation. This keeps the in-memory model clean for pt-geo/pt-quote
while supporting serialization.

## Decision 5: ExtrusionBehavior

### Option A: Enum with associated data ✓
```rust
pub enum ExtrusionBehavior {
    SitsOnTop { height_inches: f64 },
    Fills { flush: bool },
    BuildsUp { height_inches: f64 },
}
```
- Pro: Matches spec exactly, data co-located with variant
- Con: f64 for height (but this is geometry, not money — f64 is fine)

### Option B: Enum + separate height field on Material
- Pro: Flat structure
- Con: Height only meaningful for some variants; nullable field smell

**Selected**: Option A. Enum with data matches the spec and is idiomatic Rust.

## Decision 6: Material Builder Pattern

### Option A: `MaterialBuilder` with method chaining ✓
```rust
Material::builder("Travertine Pavers", MaterialCategory::Hardscape)
    .unit(Unit::SqFt)
    .price_per_unit(dec!(8.50))
    .extrusion(ExtrusionBehavior::SitsOnTop { height_inches: 1.0 })
    .build()
```
- Pro: Sensible defaults, readable test construction, validates on build
- Con: More code than a simple constructor

### Option B: `Default` trait + struct update syntax
- Pro: Less code
- Con: Name and category have no sensible default — can't impl Default

**Selected**: Option A. Builder takes required fields (name, category), everything
else has defaults. Tests benefit from concise construction with only relevant
fields specified.

## Decision 7: Error Handling

### Option A: Custom error enum per crate ✓
```rust
// pt-project
pub enum ProjectError {
    InvalidStatusTransition { from: ProjectStatus, to: ProjectStatus },
    ZoneNotFound(ZoneId),
    DuplicateZone(ZoneId),
    GeoJsonConversion(String),
    InvalidTierCount(usize),
}
```
- Pro: Typed errors, matchable, descriptive
- Con: More types to maintain

### Option B: String errors / anyhow
- Pro: Less code
- Con: Not matchable, bad for library crates

**Selected**: Option A. These are library crates — typed errors let consumers
decide how to handle failures.

## Decision 8: Status Transitions

Spec lifecycle: draft → quoted → approved → complete.

Implement as a method that validates transitions:
```rust
impl Project {
    pub fn transition_to(&mut self, status: ProjectStatus) -> Result<(), ProjectError>
}
```

Valid transitions:
- Draft → Quoted (quotes generated)
- Quoted → Approved (client approved a tier)
- Approved → Complete (work done)
- Any → Draft (reset/revert)

Invalid: Draft → Complete, Draft → Approved, etc.

## Chosen Architecture

### pt-materials (no dependencies on pt-project or pt-geo)
```
src/
├── lib.rs        — re-exports, crate doc
├── types.rs      — MaterialId, Material, MaterialCategory, Unit, ExtrusionBehavior
└── builder.rs    — MaterialBuilder
```

### pt-project (depends on geo, geojson, pt-materials for MaterialId)
```
src/
├── lib.rs        — re-exports, crate doc
├── types.rs      — ZoneId, Zone, ZoneType, Tier, TierLevel, MaterialAssignment, ProjectStatus
├── project.rs    — Project struct, zone CRUD, status transitions, tier management
├── geojson.rs    — to_geojson / from_geojson conversion
└── error.rs      — ProjectError enum
```

Wait — should pt-project depend on pt-materials for MaterialId? This creates a
dependency: pt-project → pt-materials. That's correct for the domain (a MaterialAssignment
references a MaterialId), and pt-quote → both is fine. The alternative (pt-project
defines its own MaterialId) would mean two MaterialId types that must stay in sync.

**Decision**: pt-project depends on pt-materials. MaterialAssignment uses
`pt_materials::MaterialId`. This is the honest dependency.

## Testing Strategy

### pt-materials tests (~8 tests)
- Serde round-trip: Material → JSON → Material (all fields preserved)
- Builder: required fields only → valid Material with defaults
- Builder: all fields specified → correct values
- Builder: missing required fields → error (if we add validation)
- ExtrusionBehavior serde: each variant round-trips
- Unit enum: Display/FromStr if needed
- MaterialId: equality, hashing, display

### pt-project tests (~15 tests)
- Zone CRUD: add zone, remove zone, update zone, get by id
- Duplicate zone id → error
- Zone not found → error
- Status transitions: valid (draft→quoted) and invalid (draft→complete)
- Tier construction: exactly 3 tiers enforced
- MaterialAssignment: assign material to zone in tier
- GeoJSON round-trip: Project with zones → GeoJSON → Project, all fields preserved
- GeoJSON with multiple zones of different types
- Empty project (no zones) round-trip
- Geometry preservation: polygon coordinates survive GeoJSON round-trip

### Scenario impact
S.3.1 and S.3.2 remain NotImplemented (they need pt-quote). But after this ticket,
the scenario test code can construct Projects and Materials. The types are the
foundation; pt-quote (T-002-02) will turn scenarios green.
