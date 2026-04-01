---
id: S-002
epic: E-001
title: Domain Models & Quote Engine
status: open
priority: critical
dependencies:
  - S-001
---

# S-002: Domain Models & Quote Engine

## Purpose

Define the core data types that the entire system operates on. The Project model (with Zones, Tiers, MaterialAssignments) is the single source of truth — every component reads from or writes to it. The quote engine is pure computation that turns geometry + materials into priced line items.

## Scope

- pt-project: Project, Zone, ZoneType, Tier, MaterialAssignment, ProjectStatus. GeoJSON serialization/deserialization.
- pt-materials: Material, MaterialCategory, Unit, ExtrusionBehavior, TextureRef. Catalog model with tenant ownership.
- pt-quote: Quote, LineItem. Takes a Project + Tier + material catalog, computes quantities from zone geometry (via pt-geo), multiplies by unit prices. Pure computation, no I/O.

## Tickets

- T-002-01: pt-project and pt-materials crates
- T-002-02: pt-quote crate
