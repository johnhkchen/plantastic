---
id: E-001
title: Core Domain & Data Layer
status: open
sprint: 1
---

# E-001: Core Domain & Data Layer

## Goal

Establish the Rust workspace, foundational domain crates, and database layer that everything else builds on. At the end of this epic, the project has a working Cargo workspace with pt-geo, pt-project, pt-materials, and pt-quote crates — all with tests — backed by a PostGIS schema and a repository layer that can persist and retrieve projects.

This is the narrow critical path. No other epic can ship working features without the types and persistence this epic provides.

## Stories

- **S-001**: Workspace & Geometry — Cargo workspace scaffolding + pt-geo crate
- **S-002**: Domain Models & Quote Engine — pt-project, pt-materials, pt-quote
- **S-003**: Database & Repository — PostGIS migrations + sqlx repository layer

## Success Criteria

- `cargo build --workspace` compiles cleanly
- `cargo test --workspace` passes with geometry, domain model, and quote engine coverage
- PostgreSQL schema applies cleanly via migrations
- Repository layer round-trips a Project (with zones, materials, tier assignments) through the database
