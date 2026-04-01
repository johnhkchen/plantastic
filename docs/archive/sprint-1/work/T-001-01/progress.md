# Progress — T-001-01: Workspace Scaffolding

## Completed

### Step 1: .gitignore
Created `.gitignore` covering Rust (target/), Node (node_modules/), environment files (.env), OS (.DS_Store), and IDE artifacts.

### Step 2: Workspace Cargo.toml
Created root `Cargo.toml` with:
- `[workspace]` with `members = ["crates/*"]` and `resolver = "2"`
- `[workspace.package]` with edition = "2021", license = "MIT", rust-version = "1.75"
- `[workspace.dependencies]` with geo 0.29, geojson 0.24, serde 1, serde_json 1, uuid 1, chrono 0.4, rust_decimal 1

**Deviation from plan**: Used `members = ["crates/*"]` instead of `["crates/*", "apps/*"]`. Cargo errors when directories exist under a member glob but have no Cargo.toml. Since no app crates exist yet (apps/api and apps/viewer have .gitkeep only), including `apps/*` causes a build failure. The glob will be added when the first app ticket creates a Cargo.toml under apps/.

### Step 3: pt-geo crate
Created `crates/pt-geo/Cargo.toml` (deps: geo, geojson, serde) and `src/lib.rs` with doc comment.

### Step 4: pt-project crate
Created `crates/pt-project/Cargo.toml` (deps: geo, geojson, serde, serde_json, uuid, chrono) and `src/lib.rs` with doc comment.

### Step 5: pt-materials crate
Created `crates/pt-materials/Cargo.toml` (deps: serde, uuid, rust_decimal) and `src/lib.rs` with doc comment.

### Step 6: pt-quote crate
Created `crates/pt-quote/Cargo.toml` (deps: serde, uuid, rust_decimal) and `src/lib.rs` with doc comment.

### Step 7: Empty directories
Created .gitkeep files in: apps/api/, apps/viewer/, web/, worker/, baml_src/, assets/textures/, assets/models/, migrations/, infra/.

Note: web/ already had a SvelteKit project scaffolded (with node_modules, svelte.config.js, etc.) — the .gitkeep is redundant there but harmless.

### Step 8: README.md
Updated README.md with project description, full workspace layout diagram (showing current and future crates), and getting started section.

### Step 9: Verification
`cargo check` passes — all 4 crates compile successfully (5.61s, 156 dependencies resolved).

## Remaining

None — all steps complete.
