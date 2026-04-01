---
id: S-001
epic: E-001
title: Workspace & Geometry
status: open
priority: critical
---

# S-001: Workspace & Geometry

## Purpose

Bootstrap the Rust monorepo and deliver the first domain crate. pt-geo is the foundation — every spatial operation in the system (area calculation for quotes, polygon projection for 3D scenes, boolean ops for zone editing) depends on it.

## Scope

- Cargo workspace with workspace-level dependency management
- Directory structure matching the specification (crates/, apps/, web/, worker/, baml_src/, assets/, migrations/, infra/)
- pt-geo crate: polygon area (sq ft), perimeter (linear ft), volume from area × depth (cu ft → cu yd), boolean operations (union, subtract), polygon simplification
- Thin wrapper around the `geo` crate. Pure functions, no I/O, extensively tested.

## Tickets

- T-001-01: Cargo workspace scaffolding
- T-001-02: pt-geo crate
