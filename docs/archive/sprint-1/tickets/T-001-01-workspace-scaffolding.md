---
id: T-001-01
story: S-001
title: workspace-scaffolding
type: task
status: open
priority: critical
phase: done
depends_on: []
---

## Context

Bootstrap the Rust monorepo workspace. This is the first ticket — everything else depends on the directory structure and workspace Cargo.toml being in place. Sets up the skeleton that all crates and apps will live in.

## Acceptance Criteria

- Workspace Cargo.toml at repo root with workspace members for crates/*, apps/*
- Workspace-level dependency management (shared versions for geo, geojson, serde, uuid, chrono, rust_decimal)
- Directory structure created matching specification: crates/, apps/api/, apps/viewer/, web/, worker/, baml_src/, assets/textures/, assets/models/, migrations/, infra/
- Placeholder lib.rs in pt-geo, pt-project, pt-materials, pt-quote crate dirs (enough for `cargo check` to pass)
- .gitignore for Rust (target/), Node (node_modules/), and environment files
- README.md updated with project description and workspace layout
