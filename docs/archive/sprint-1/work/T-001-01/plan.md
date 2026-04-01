# Plan — T-001-01: Workspace Scaffolding

## Step 1: Create .gitignore

Create `.gitignore` at repo root covering Rust (target/), Node (node_modules/), environment files (.env), OS files (.DS_Store), and IDE artifacts.

**Verify**: File exists, covers all required categories.

## Step 2: Create workspace Cargo.toml

Create root `Cargo.toml` with:
- `[workspace]` with `members = ["crates/*", "apps/*"]` and `resolver = "2"`
- `[workspace.package]` with shared edition, license, rust-version
- `[workspace.dependencies]` with geo, geojson, serde, serde_json, uuid, chrono, rust_decimal (pinned versions with feature flags)

**Verify**: Valid TOML syntax.

## Step 3: Create pt-geo crate skeleton

Create `crates/pt-geo/Cargo.toml` inheriting workspace metadata, depending on geo, geojson, serde. Create `crates/pt-geo/src/lib.rs` with doc comment.

**Verify**: `cargo check -p pt-geo` passes.

## Step 4: Create pt-project crate skeleton

Create `crates/pt-project/Cargo.toml` inheriting workspace metadata, depending on geo, geojson, serde, serde_json, uuid, chrono. Create `crates/pt-project/src/lib.rs` with doc comment.

**Verify**: `cargo check -p pt-project` passes.

## Step 5: Create pt-materials crate skeleton

Create `crates/pt-materials/Cargo.toml` inheriting workspace metadata, depending on serde, uuid, rust_decimal. Create `crates/pt-materials/src/lib.rs` with doc comment.

**Verify**: `cargo check -p pt-materials` passes.

## Step 6: Create pt-quote crate skeleton

Create `crates/pt-quote/Cargo.toml` inheriting workspace metadata, depending on serde, uuid, rust_decimal. Create `crates/pt-quote/src/lib.rs` with doc comment.

**Verify**: `cargo check -p pt-quote` passes.

## Step 7: Create empty directories with .gitkeep

Create the following directories, each containing a `.gitkeep` file:
- `apps/api/`
- `apps/viewer/`
- `web/`
- `worker/`
- `baml_src/`
- `assets/textures/`
- `assets/models/`
- `migrations/`
- `infra/`

**Verify**: All directories exist with .gitkeep files.

## Step 8: Update README.md

Replace the existing 2-line README.md with:
- Project name and description
- What Plantastic is (brief)
- Workspace layout (directory tree)
- Getting started section (prerequisites, cargo check)

**Verify**: Content is accurate and matches actual directory structure.

## Step 9: Full verification

Run `cargo check` at workspace root — all 4 crates must compile successfully. Verify directory structure matches acceptance criteria.

**Verify**: `cargo check` exits 0. All acceptance criteria met.

## Testing Strategy

This ticket is pure scaffolding — no logic to unit test. Verification is:

1. **`cargo check`** — compilation succeeds for all workspace members
2. **Directory existence** — all specified directories exist
3. **File content** — .gitignore covers required patterns, README has project description and layout

No integration tests. No test dependencies needed.

## Commit Strategy

Single atomic commit after all files are created and `cargo check` passes. The scaffolding is one logical unit — partial commits would leave a broken workspace.
