# Design — T-001-01: Workspace Scaffolding

## Decision 1: Workspace Member Strategy

### Option A: Glob-based members (`crates/*`, `apps/*`)

Cargo workspace supports glob patterns: `members = ["crates/*", "apps/*"]`. This auto-discovers any subdirectory with a Cargo.toml. New crates are picked up automatically.

**Pros**: No workspace Cargo.toml edits when adding crates. Matches the ticket's language ("workspace members for crates/*, apps/*").

**Cons**: Every subdirectory under crates/ or apps/ must have a valid Cargo.toml — can't have non-crate directories there. Empty directories without Cargo.toml are silently ignored (this is fine).

### Option B: Explicit member paths

List each crate explicitly: `members = ["crates/pt-geo", "crates/pt-project", ...]`.

**Pros**: Precise control. No surprises from accidental directories.

**Cons**: Must edit workspace Cargo.toml for every new crate. More maintenance.

### Decision: Option A (glob-based)

The ticket text explicitly says "workspace members for crates/*, apps/*". Glob-based discovery is the idiomatic Cargo approach for monorepos. New tickets (T-001-02, T-002-01, etc.) can add crates without touching the workspace root.

## Decision 2: Which Crate Directories to Create Now

### Option A: All 14 crates + 2 apps from the spec

Create every crate directory mentioned in the specification, each with stub Cargo.toml + lib.rs.

**Pros**: Full structure in place immediately.

**Cons**: Violates the ticket scope. Acceptance criteria explicitly lists only 4 crates (pt-geo, pt-project, pt-materials, pt-quote). Other crates are work for their own tickets.

### Option B: Only the 4 crates specified in acceptance criteria

Create pt-geo, pt-project, pt-materials, pt-quote with placeholder Cargo.toml + lib.rs. Leave apps/ empty (no workspace member Cargo.toml files for api/ or viewer/ yet).

**Pros**: Matches acceptance criteria exactly. Keeps scope tight. Other tickets will populate their own crates.

**Cons**: The `apps/*` glob in workspace members won't match anything yet. That's fine — Cargo handles empty globs gracefully.

### Decision: Option B (4 crates only)

The acceptance criteria are explicit. Creating more would be scope creep. The apps/ member glob is forward-looking and harmless when empty.

## Decision 3: Workspace Dependency Versions

The acceptance criteria list 6 shared dependencies: geo, geojson, serde, uuid, chrono, rust_decimal. Use `[workspace.dependencies]` to pin versions at the workspace level.

### Version Strategy

Use the latest stable versions compatible with each other. Key compatibility concern: `geo` and `geojson` must be compatible versions since pt-geo will use both.

| Dependency | Version | Notes |
|-----------|---------|-------|
| geo | 0.29 | Latest stable, geo-types 0.7.x |
| geojson | 0.24 | Compatible with geo 0.29 |
| serde | 1 | Stable, with derive feature |
| serde_json | 1 | Not in AC but useful with serde; include for completeness |
| uuid | 1 | With v4 and serde features |
| chrono | 0.4 | With serde feature |
| rust_decimal | 1 | With serde-with-str feature |

### Feature Flags

Declare commonly-needed features at the workspace level:
- `serde = { version = "1", features = ["derive"] }`
- `uuid = { version = "1", features = ["v4", "serde"] }`
- `chrono = { version = "0.4", features = ["serde"] }`
- `rust_decimal = { version = "1", features = ["serde-with-str"] }`

Individual crates inherit these and can add more features if needed.

## Decision 4: Placeholder lib.rs Content

### Option A: Empty file

Just create empty `lib.rs` files.

**Pros**: Minimal. Compiles fine.

**Cons**: No indication of what the crate is for.

### Option B: Single doc comment

A one-line `//! Crate description` doc comment.

**Pros**: `cargo doc` works. Gives context. Still minimal.

**Cons**: Slightly more than "placeholder."

### Decision: Option B (single doc comment)

One line of documentation per crate is appropriate scaffolding, not scope creep. It makes `cargo doc` useful from the start.

## Decision 5: .gitignore Coverage

The acceptance criteria require: Rust (target/), Node (node_modules/), environment files.

Contents:
```
# Rust
/target/
**/*.rs.bk

# Node
node_modules/
.svelte-kit/

# Environment
.env
.env.*
!.env.example

# OS
.DS_Store

# IDE
.idea/
*.swp
*.swo
```

Single .gitignore at repo root. No per-directory .gitignore files needed.

## Decision 6: README.md Content

The acceptance criteria say "project description and workspace layout." Update the existing README.md with:

1. Project name and one-line description
2. What Plantastic is (brief — the spec has details)
3. Workspace layout diagram (matching the spec's monorepo structure, simplified)
4. Getting started (cargo check, prerequisites)

Keep it concise — this is a workspace README, not product documentation.

## Decision 7: Empty Directory Tracking

Git doesn't track empty directories. The ticket requires these directories to exist: web/, worker/, baml_src/, assets/textures/, assets/models/, migrations/, infra/.

### Option A: .gitkeep files

Standard convention: place an empty `.gitkeep` file in each directory.

### Option B: Skip them

Let future tickets create the directories when they add content.

### Decision: Option A (.gitkeep)

The acceptance criteria explicitly require the directory structure to be "created." .gitkeep is the standard way to commit empty directories.

## Decision 8: Workspace Package Metadata

Use `[workspace.package]` to share common metadata:

```toml
[workspace.package]
edition = "2021"
license = "MIT"
rust-version = "1.75"
```

Individual crate Cargo.toml files inherit via `edition.workspace = true`, etc. This keeps metadata consistent and reduces boilerplate across 14+ future crates.

## Rejected Alternatives

1. **Using `cargo init` per crate**: Generates too much boilerplate (main.rs, default tests). Manual Cargo.toml + lib.rs is cleaner for library crates.
2. **Adding dev-dependencies to workspace**: Not needed yet. No tests to run. Future tickets will add testing deps.
3. **Creating a workspace-level build.rs**: No build-time code generation needed at this stage.
4. **Adding CI/CD configuration**: Out of scope. No `.github/workflows/` mentioned in acceptance criteria.
