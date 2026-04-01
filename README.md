# Plantastic

A B2B platform for landscaping companies. Clients scan their yard with an iPhone, landscapers draw zones and assign materials across three tiers, and the system generates branded 3D previews, accurate quotes, and crew-ready exports.

## Workspace Layout

```
plantastic/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── pt-geo/             # Geometry & spatial math
│   ├── pt-project/         # Project domain model
│   ├── pt-materials/       # Material catalog domain
│   ├── pt-quote/           # Quote engine
│   ├── pt-solar/           # Solar radiance engine (future)
│   ├── pt-climate/         # Climate data models (future)
│   ├── pt-plants/          # Plant intelligence (future)
│   ├── pt-scan/            # Scan processing pipeline (future)
│   ├── pt-scene/           # 3D scene generator (future)
│   ├── pt-pdf/             # PDF generation (future)
│   ├── pt-dxf/             # DXF export (future)
│   ├── pt-satellite/       # Satellite pre-population (future)
│   └── pt-tenant/          # Tenant/brand domain (future)
├── apps/
│   ├── api/                # Axum backend (Lambda target)
│   └── viewer/             # Bevy 3D viewer (WASM target)
├── web/                    # SvelteKit frontend (CF Pages)
├── worker/                 # Cloudflare Worker proxy
├── baml_src/               # BAML definitions (AI layer)
├── assets/
│   ├── textures/           # Default PBR texture sets
│   └── models/             # Plant models, furniture
├── migrations/             # PostgreSQL/PostGIS schema
├── infra/                  # SST IaC + deploy scripts
└── docs/                   # Specification, tickets, work artifacts
```

## Getting Started

### Prerequisites

- Rust 1.75+ (`rustup update stable`)

### Build

```sh
cargo check    # Verify workspace compiles
cargo build    # Build all crates
cargo test     # Run all tests
```
