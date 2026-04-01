# Research — T-001-01: Workspace Scaffolding

## Current State

The repository is a fresh git repo with a single commit ("Initial commit"). It contains:

- `README.md` — two-line placeholder ("# plantastic / Gardening Plan System")
- `LICENSE` — project license file
- `CLAUDE.md` — Lisa agent instructions (directory conventions, RDSPI workflow reference)
- `docs/` — specification, knowledge base, active tickets/stories/epics
- `.lisa.toml`, `.lisa-layout.kdl`, `.lisa/` — Lisa orchestrator config
- `.claude/` — Claude Code project config

No Rust code, no Cargo.toml, no .gitignore, no directory structure beyond docs/.

### Toolchain

- Rust 1.94.1 (stable, 2026-03-25)
- Cargo 1.94.1

## Specification Requirements

The specification (`docs/specification.md`) defines a comprehensive monorepo structure:

### Crate Directory (`crates/`)

14 domain crates, all prefixed `pt-`:

| Crate | Purpose | I/O? |
|-------|---------|------|
| pt-geo | Geometry & spatial math | No |
| pt-solar | Solar radiance engine | No |
| pt-climate | Climate data models | No |
| pt-plants | Plant intelligence + Plant.id client | Yes (API) |
| pt-project | Project domain model (GeoJSON) | No |
| pt-materials | Material catalog domain | No |
| pt-quote | Quote engine | No |
| pt-scan | Scan processing pipeline | Yes (files) |
| pt-scene | 3D scene generator | No |
| pt-pdf | PDF generation (typst) | Yes (files) |
| pt-dxf | DXF export | Yes (files) |
| pt-satellite | Satellite pre-population | Yes (API) |
| pt-tenant | Tenant/brand domain | No |

### App Directory (`apps/`)

| App | Purpose | Target |
|-----|---------|--------|
| api | Axum backend | AWS Lambda (arm64) |
| viewer | Bevy 3D viewer | WASM |

### Non-Rust Directories

| Directory | Purpose |
|-----------|---------|
| web/ | SvelteKit frontend (Cloudflare Pages) |
| worker/ | Cloudflare Worker proxy |
| baml_src/ | BAML definitions (AI layer) |
| assets/textures/ | Default PBR texture sets |
| assets/models/ | Plant models, furniture |
| migrations/ | PostgreSQL/PostGIS schema |
| infra/ | SST IaC + deploy scripts |

## Ticket Scope — What's Actually Required

The ticket acceptance criteria are deliberately narrow:

1. **Workspace Cargo.toml** — workspace members for `crates/*` and `apps/*`
2. **Workspace dependency management** — shared versions for: geo, geojson, serde, uuid, chrono, rust_decimal
3. **Directory structure** — crates/, apps/api/, apps/viewer/, web/, worker/, baml_src/, assets/textures/, assets/models/, migrations/, infra/
4. **Placeholder lib.rs** — only in pt-geo, pt-project, pt-materials, pt-quote (enough for `cargo check`)
5. **.gitignore** — Rust (target/), Node (node_modules/), environment files
6. **README.md** — project description and workspace layout

### Key Observation

Only 4 of the 14 crates need placeholders (the ones referenced by story S-001 and S-002 tickets). The remaining 10 crate directories are NOT required — the ticket says to create the `crates/` parent directory and only populate the 4 needed crates. The `apps/` directory needs `api/` and `viewer/` subdirectories but no Cargo.toml or source files for them yet (those are separate tickets).

Wait — re-reading the acceptance criteria: "Workspace Cargo.toml at repo root with workspace members for crates/*, apps/*". This implies glob-based workspace members, which means all crate directories listed as members need a Cargo.toml, or we use path-based globs that tolerate empty dirs. Cargo workspace `members = ["crates/*"]` requires each subdirectory under `crates/` to have a `Cargo.toml`. So we should only create crate dirs that have Cargo.toml files.

The 4 crates with placeholder lib.rs files need Cargo.toml files. The apps need at minimum stub Cargo.toml files if included in workspace members. But the ticket says "Placeholder lib.rs in pt-geo, pt-project, pt-materials, pt-quote" — the apps are not mentioned for placeholders.

### Resolution

Create only the 4 crate directories (pt-geo, pt-project, pt-materials, pt-quote) under `crates/`. Use `members = ["crates/*"]` which will pick up only directories with Cargo.toml. Do NOT create apps/api or apps/viewer as workspace members yet — those are separate tickets. Create the non-Rust directories (web/, worker/, etc.) as empty dirs with .gitkeep.

## Dependency Research

Shared workspace dependencies and their current crates.io versions:

| Crate | Purpose | Current Version |
|-------|---------|----------------|
| geo | Geospatial primitives & algorithms | 0.29.x |
| geojson | GeoJSON serialization | 0.24.x |
| serde | Serialization framework | 1.x |
| serde_json | JSON serialization | 1.x |
| uuid | UUID generation/parsing | 1.x |
| chrono | Date/time handling | 0.4.x |
| rust_decimal | Precise decimal arithmetic | 1.x |

These will be declared in `[workspace.dependencies]` with versions, and individual crates inherit via `dep.workspace = true`.

## Patterns and Conventions

### Cargo Workspace Conventions

- `resolver = "2"` is standard for new Rust 2021+ projects
- `[workspace.dependencies]` for shared version management (stabilized in Rust 1.64)
- `[workspace.package]` for shared metadata (edition, license, rust-version)
- Glob members (`"crates/*"`) auto-discover crates

### Existing Repo Conventions

- Crate prefix: `pt-` (for "plantastic")
- Docs follow Lisa RDSPI workflow in `docs/active/`
- README exists but is minimal

## Constraints and Risks

1. **No overengineering**: This is scaffolding. Don't add crates or dependencies not in the acceptance criteria.
2. **cargo check must pass**: The 4 placeholder crates need valid Cargo.toml + lib.rs pairs.
3. **Future tickets depend on this structure**: T-001-02 (pt-geo), T-002-01 (pt-project, pt-materials), T-002-02 (pt-quote) all assume this workspace exists.
4. **No code logic**: Placeholder lib.rs files should be empty or near-empty — just enough for compilation.

## Files to Create

- `/Cargo.toml` — workspace root
- `/crates/pt-geo/Cargo.toml` + `src/lib.rs`
- `/crates/pt-project/Cargo.toml` + `src/lib.rs`
- `/crates/pt-materials/Cargo.toml` + `src/lib.rs`
- `/crates/pt-quote/Cargo.toml` + `src/lib.rs`
- `/.gitignore`
- `/README.md` — update existing

## Directories to Create (empty, with .gitkeep)

- `apps/api/`
- `apps/viewer/`
- `web/`
- `worker/`
- `baml_src/`
- `assets/textures/`
- `assets/models/`
- `migrations/`
- `infra/`
