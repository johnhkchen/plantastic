# Structure — T-001-01: Workspace Scaffolding

## Files Created

### Workspace Root

**`Cargo.toml`** — Workspace definition

```
[workspace]
members = ["crates/*", "apps/*"]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT"
rust-version = "1.75"

[workspace.dependencies]
geo = "0.29"
geojson = "0.24"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1", features = ["serde-with-str"] }
```

**`.gitignore`** — Rust, Node, env, OS, IDE ignores

**`README.md`** — Updated with project description, workspace layout, and getting started

### Crate: pt-geo

**`crates/pt-geo/Cargo.toml`**
```
[package]
name = "pt-geo"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
geo.workspace = true
geojson.workspace = true
serde.workspace = true
```

**`crates/pt-geo/src/lib.rs`** — Doc comment placeholder

### Crate: pt-project

**`crates/pt-project/Cargo.toml`**
```
[package]
name = "pt-project"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
geo.workspace = true
geojson.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
```

**`crates/pt-project/src/lib.rs`** — Doc comment placeholder

### Crate: pt-materials

**`crates/pt-materials/Cargo.toml`**
```
[package]
name = "pt-materials"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
serde.workspace = true
uuid.workspace = true
rust_decimal.workspace = true
```

**`crates/pt-materials/src/lib.rs`** — Doc comment placeholder

### Crate: pt-quote

**`crates/pt-quote/Cargo.toml`**
```
[package]
name = "pt-quote"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
serde.workspace = true
uuid.workspace = true
rust_decimal.workspace = true
```

**`crates/pt-quote/src/lib.rs`** — Doc comment placeholder

## Empty Directories (with .gitkeep)

Each gets a single `.gitkeep` file:

```
apps/api/.gitkeep
apps/viewer/.gitkeep
web/.gitkeep
worker/.gitkeep
baml_src/.gitkeep
assets/textures/.gitkeep
assets/models/.gitkeep
migrations/.gitkeep
infra/.gitkeep
```

## Files Modified

**`README.md`** — Replaced 2-line placeholder with project description and workspace layout

## Files NOT Modified

- `CLAUDE.md` — No changes needed
- `LICENSE` — No changes needed
- `docs/**` — No changes to documentation structure
- `.lisa*` — No changes to Lisa config

## Module Boundaries

At this stage there are no module boundaries to define — each crate is a single empty lib.rs. The dependency graph between crates is:

```
pt-geo         (standalone: geo, geojson, serde)
pt-project     (standalone: geo, geojson, serde, serde_json, uuid, chrono)
pt-materials   (standalone: serde, uuid, rust_decimal)
pt-quote       (standalone: serde, uuid, rust_decimal)
```

No inter-crate dependencies yet. Those will be added by future tickets (e.g., T-001-02 will flesh out pt-geo, T-002-02 will make pt-quote depend on pt-geo and pt-materials).

## Dependency Notes

Each crate declares only the workspace dependencies it will eventually need based on the specification:

- **pt-geo**: geo (spatial primitives), geojson (serialization), serde
- **pt-project**: geo + geojson (zone geometry), serde + serde_json (project model serialization), uuid (entity IDs), chrono (timestamps)
- **pt-materials**: serde (serialization), uuid (entity IDs), rust_decimal (pricing)
- **pt-quote**: serde (serialization), uuid (entity IDs), rust_decimal (pricing math)

## Ordering

No ordering constraints — all files can be created in any order. The only requirement is that the workspace Cargo.toml exists before running `cargo check`, and that crate Cargo.toml files reference valid workspace dependencies.

## Verification

After all files are created:

```
cargo check          # must succeed — all 4 crates compile
cargo doc --no-deps  # optional — doc comments render
```
