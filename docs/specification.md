# Plantastic — Product & Technical Specification

## The Problem

A landscaping company's revenue is bottlenecked by the design-to-proposal cycle. Every potential job requires a site visit, manual measurements, mental inventory of what grows in that specific spot, hand-calculated material quantities, spreadsheet pricing across option tiers, and then somehow communicating the vision to a homeowner who can't visualize a garden from a line-item spreadsheet.

That cycle takes the most experienced person on the team 3-5 hours per prospect. Most of that time isn't design work — it's data gathering and arithmetic. The expert's taste and plant knowledge get buried under tape measures and takeoff sheets. The admin can't run proposals independently, so the company's throughput is capped by one person's calendar. Prospects go cold. Lower-value jobs get declined because the proposal cost doesn't justify the margin. Bigger jobs close at the cheapest tier because the client can't see what "better" looks like.

The company that turns a site visit into three polished, visual, accurately-priced proposals in 30 minutes instead of 3 days wins more contracts at higher tiers.

-----

## What Plantastic Is

A B2B platform for landscaping companies and garden planners. Clients scan their yard with an iPhone, the platform pre-populates site data from satellite and municipal sources, landscapers draw zones and assign materials across three tiers, and the system generates branded 3D previews, accurate quotes, and crew-ready exports.

The scan provides close-enough geometry. The 3D scene is the product. Everyone — client, landscaper, crew — points at the same thing and agrees on what gets built.

**Domain:** get-plantastic.com

-----

## Stakeholders

### Landscaping Company (Tenant)

The B2B customer. Each company is a tenant on the platform with their own branding, material catalog, pricing, and client list. Plantastic is invisible to the homeowner — they see their landscaper's brand.

### Landscaper / Designer

The domain expert. Spends their time on design decisions, not measurement and calculation. Assigns materials and plants per zone across three tiers (good / better / best). Reviews and adjusts AI-generated recommendations. Their expertise is encoded into the catalog and project, not locked in their head.

### Admin

Runs the business. Handles client communication, generates proposals, manages the project pipeline. With Plantastic, the admin can run a consultation from site visit through signed proposal without needing the expert present for every step.

### Crew

Builds from the plan. Opens the approved project on a tablet on site. Sees the same 3D view the client approved, with material callouts, dimensions, supplier SKUs, and install depths. No re-interpretation of a paper sketch.

### Homeowner / Client

The end user of the landscaper's service. Scans their yard, draws rough zones with intent ("patio here", "flower bed"), views three options side by side in 3D, approves a tier. Feels confident about what they're buying before writing a check.

-----

## Proven Foundations

Plantastic draws on validated work from multiple prototypes:

**Solar-sim** (SvelteKit prototype) proved that solar radiance calculation, shade modeling (terrain + buildings + trees), climate integration (frost dates, Sunset zones, microclimate), and scoring-based plant recommendations work at horticultural accuracy. Five-minute sampling intervals yield sub-1% error. Shade timing (morning vs afternoon) meaningfully affects plant advice.

**Satellite/LiDAR prototype** (Rust) proved that pre-populating site data from satellite LiDAR, SF gov parcel data, and Meta's canopy height dataset is viable. Also proved that Rust is necessary for production-grade compute performance on spatial data, and that larger municipal plant databases can be ingested and scored. Revealed the resolution floor: satellite data gets to ~1 foot but no further. Under-canopy and ground-level detail requires iPhone LiDAR on site.

**HMW Workshop** (Go + BAML + SvelteKit) proved the production deployment pattern: Lambda scale-to-zero with low cold starts, SSE streaming for AI responses, BAML for type-safe LLM orchestration, SvelteKit on Cloudflare Pages, CF Worker proxy for edge concerns. This architecture pattern carries forward, with Rust replacing Go as the Lambda runtime.

-----

## System Components

### 1. Scan Processing Pipeline

**PLY in → terrain mesh (glTF) + plan view (PNG) + metadata (JSON)**

Receives iPhone LiDAR scans captured via SiteScape. Processes the point cloud into usable artifacts: a decimated terrain mesh for 3D rendering, a top-down orthographic image for the 2D editor, and metadata (bounding box, elevation range, point count).

Processing steps: statistical outlier removal, RANSAC ground plane fitting, voxel downsampling, Delaunay triangulation, mesh decimation (~50k triangles), orthographic projection, glTF binary export.

Implemented in Rust. This is one-shot compute per project — heavier than most operations but infrequent. Runs as an async job triggered on upload.

Replaces: manual site measurement with tape measure, graph paper, return visits.

### 2. Satellite Pre-population

**Address → lot boundary, existing trees, sun exposure baseline**

When a landscaper creates a new project from an address, the platform pre-fills a project skeleton before anyone visits the site. Lot boundary from municipal parcel data. Existing trees detected from Meta canopy height data with estimated height and spread. Baseline sun exposure grid computed from Rust solar radiance engine accounting for terrain and detected structures.

This gives the landscaper a head start. They arrive at the site visit already knowing the sun profile and major features, and the client's on-site scan refines rather than starts from scratch.

Data sources are cacheable per address. Solar radiance computation is CPU-bound but proven fast in Rust (~2ms for a full year at a single point, parallelizable across a grid).

Replaces: the cold start on every new project where the landscaper knows nothing until they visit.

### 3. Plant Intelligence

**Platform-level plant database + multi-factor scoring + Plant.id integration**

The plant knowledge layer. A large curated database sourced from municipal databases and horticultural references, scored against site conditions using a proven weighted algorithm: light requirements (50%), season length (30%), climate zone (20%), with soil compatibility as an additional factor.

Sunset Western Garden zones are the authority for the Bay Area market. The scoring system produces nuanced results (excellent / good / marginal / not recommended) rather than binary yes/no filtering.

Plant.id integration: photos taken during the site visit identify existing plants by species. These get mapped into the platform database and placed in the project model with their observed location, giving the landscaper an inventory of what's already growing.

AI-powered contextual recommendations via BAML explain why a plant works in a specific zone ("afternoon shade from the existing oak prevents lettuce from bolting in summer heat"). This is the expertise-encoding that lets the admin make suggestions without the expert.

Replaces: the landscaper's mental catalog, reference book lookups, and the "I think that'll grow there" guesswork.

### 4. Project Model (the core)

**GeoJSON-based project: scan + zones + material assignments + tiers + status**

The single source of truth. Every other component reads from or writes to this model. A project contains:

- **Scan reference**: pointer to the PLY and derived terrain/PNG artifacts
- **Pre-populated baseline**: lot boundary, detected trees, sun exposure grid
- **Zones**: typed polygons (bed, patio, path, lawn, wall, edging) with optional labels
- **Three tiers**: good / better / best, each with material and plant assignments per zone
- **Status lifecycle**: draft → quoted → approved → complete

```
Project
├── ScanRef (PLY + terrain.glb + planview.png + metadata.json)
├── Baseline (lot polygon, trees[], sun exposure grid)
├── Zone[] (id, geometry, zone_type, label)
├── Tier[3]
│   └── MaterialAssignment[] (zone_id, material_id, overrides)
└── Status
```

GeoJSON is the canonical serialization format. Stored in PostgreSQL with PostGIS for spatial queries. This is the data model that makes the plan the product — not a report generated at the end, but the central artifact from the first zone drawn to the crew building it.

### 5. 2D Zone Editor

**Polygon drawing over the plan view image**

The primary design interface. The landscaper or client draws zones on top of the orthographic plan view PNG. Each zone is a typed polygon (bed, patio, path, lawn, wall, edging) with an optional label capturing intent.

As zones are drawn, computed measurements update in real time: area (sq ft), perimeter (linear ft), volume at depth (cu yd). Boolean operations let zones interact (subtract a patio from a lawn, split a bed with a path).

This is a SvelteKit component — canvas-based polygon drawing with snap-to-grid and edge snapping. The geometry writes back to the GeoJSON project model.

Replaces: sketching on printed satellite photos, verbal descriptions of "about this big", multiple revision rounds to agree on boundaries.

### 6. Material Catalog

**Per-tenant catalog layered on a platform starter set**

Each landscaping company curates their own materials: preferred suppliers, negotiated pricing, proven products. This is a competitive differentiator — the catalog encodes business relationships and regional expertise.

A material includes: name, category (hardscape / softscape / edging / fill), unit (sq ft / cu yd / linear ft / each), price per unit, default install depth, PBR texture set for 3D rendering, product photo for quotes, supplier SKU, and extrusion behavior (sits on top / fills / builds up).

The platform provides a starter catalog with common materials and default textures. Supplier partnerships can provide pre-built catalogs. Landscapers add, remove, adjust pricing, and upload their own product photos.

Replaces: spreadsheet price lists, supplier catalog binders, inconsistent quoting across projects.

### 7. Quote Engine

**Zone geometry + material assignment → quantities → prices × 3 tiers**

Pure computation. For each tier, the engine walks every zone, computes the quantity needed (area × 1 for surface materials, area × depth for fill, perimeter × 1 for edging), multiplies by the material's unit price, and produces line items.

Output: per-zone line items with material name, quantity, unit, unit price, and line total. Subtotal, tax, and total per tier. Three tiers side by side for comparison.

This is the single biggest time saver. Manual quantity takeoff is hours of spreadsheet work per project. The quote engine makes it instant and accurate, recalculating on every material change.

Replaces: manual takeoff sheets, calculator math, spreadsheet formulas, the "let me get back to you with numbers" delay.

### 8. 3D Scene Generator

**Project model → glTF scene per tier**

The visualization engine. Takes the project's terrain mesh, zone geometries, and material assignments for a given tier, and produces a single glTF binary that represents the finished design.

Process:
1. Load terrain mesh as the base layer
2. Project 2D zone polygons onto the 3D terrain surface
3. Extrude each zone according to its material's behavior (pavers sit on top, mulch fills flush, walls build up)
4. Generate UV coordinates for tiling PBR textures
5. Apply material texture sets
6. Merge into a single glTF binary

This is the most complex computation in the system. Terrain-conforming polygon projection and UV generation are hard geometry problems. One scene per tier, three per project, regenerated on material changes, cached in object storage.

Replaces: "trust me, it'll look great" and the imagination gap that makes clients pick the cheapest option.

### 9. 3D Viewer

**Bevy compiled to WASM, embedded in the SvelteKit frontend**

Loads a generated glTF scene and provides interactive exploration:
- Orbit camera (drag to rotate, scroll to zoom)
- Tap a zone to see material info, dimensions, and product photo
- Toggle between three tier scenes
- Sunlight direction slider (see shadows at different times of day)
- Measurement tool (tap two points, get distance)

Compiled to WASM, embedded in the web app via iframe. Must run on iPad Safari for crew field use in bright sunlight (high contrast, large tap targets).

The viewer is how the client experiences the design. It's how the landscaper presents the proposal. It's how the crew references the plan on site. Same artifact, three audiences.

### 10. PDF Generator

**Branded quote document**

The formal proposal. Carries the landscaping company's brand — logo, colors, contact info — not Plantastic's. Contains:

- Landscaper's branding and contact
- Plan view image with zone overlays
- Three-tier comparison table
- Per-tier line item breakdown
- Material product photos
- Growing notes and planting calendar (for plant-heavy zones)
- Terms and signature line

Generated in Rust (typst). On demand, cached in object storage. This is what gets emailed, printed, and signed.

Replaces: Word documents with pasted-in photos, handwritten quotes, inconsistent formatting that undermines professional credibility.

### 11. DXF Export

**CAD file from approved tier**

For crews that want precise field reference in a CAD tool. Each zone type maps to a DXF layer. Polygons become LWPOLYLINE entities. Dimensions and area annotations included. Material names in TEXT entities.

Generated from the approved tier's project data via ixmilia/dxf (pure Rust). Lightweight computation.

### 12. Tenant System

**Multi-tenant platform with branded output**

Each landscaping company is a tenant:
- **Owns**: branding (logo, colors, contact info), material catalog with custom pricing, client list, all projects
- **Shares**: platform plant intelligence, solar/climate data, satellite pre-population, base PBR texture library, compute infrastructure

Tenant isolation at the database level. All client-facing output (PDFs, viewer links, emails) carries the tenant's brand. Plantastic is the platform, not the face.

### 13. AI Layer

**BAML-driven LLM features integrated into the Rust backend**

Type-safe structured LLM orchestration using BAML, compiled into the same Rust binary as the API. Capabilities:

- **Plant recommendations**: Given a zone's sun exposure, climate, soil, and the landscaper's catalog, suggest plants with contextual reasoning
- **Design assistance**: "This north-facing bed gets 3 hours of morning sun — here are shade-tolerant ground covers that pair well with the hostas the client already has"
- **Natural language intent parsing**: Client labels a zone "herb garden" → system suggests zone type, dimensions, and plant list
- **Quote narrative**: Generate a plain-English summary of what each tier includes and why it costs what it does

Streaming responses via SSE to the frontend (proven pattern from HMW Workshop). AI features enhance but never gate the core workflow — the system works without them, they make it faster and smarter.

-----

## Infrastructure

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| API + Compute | Rust (Axum) on AWS Lambda | Spatial/3D compute needs compiled performance. Scale-to-zero. Single language with domain crates. |
| AI Interface | Rust + BAML (same Lambda binary) | Type-safe LLM orchestration. No separate AI service. Structured streaming. |
| Frontend | SvelteKit on Cloudflare Pages | Proven DX from prior projects. Fast iteration. Global CDN. Free tier at prototype scale. |
| Edge Proxy | Cloudflare Worker | CORS, rate limiting, auth, Turnstile bot protection, SSE passthrough. Proven pattern from HMW. |
| Database | PostgreSQL + PostGIS | Spatial queries on zone geometry. GeoJSON native. Tenant isolation via row-level security or schema-per-tenant. |
| Object Storage | S3 / Minio | Terrain meshes, PBR textures, generated glTF/PDF/DXF artifacts, PLY uploads. |
| 3D Viewer | Rust + Bevy → WASM | Real PBR rendering in browser. No JS 3D library matches Bevy's rendering quality for this use case. |
| IaC | SST | Lambda + secrets + storage defined in code. Proven from HMW. |
| Secrets | Doppler (dev) + AWS SSM (prod) | Centralized, CLI-managed. Proven pattern. |
| Scan Capture | SiteScape (3rd party, V1) | Free iPhone LiDAR app. Good PLY export. Avoids building a native app in V1. |
| Plant ID | Plant.id (3rd party, V1) | Species identification from photos. API integration, not in-house ML. |

### Lambda Architecture

The Rust backend compiles to a single binary deployed on Lambda (provided.al2023, arm64). The binary includes:

- Axum HTTP router (API endpoints)
- All domain crates (gw-geo, gw-materials, gw-quote, gw-project)
- BAML runtime (AI interface)
- Lambda streaming adapter (RESPONSE_STREAM mode for SSE)

Heavier operations (scan processing, 3D scene generation) may need dedicated Lambda functions with higher memory/timeout configurations, or an async job queue pattern where the API enqueues work and the client polls or receives a webhook on completion.

### Data Flow

```
iPhone (SiteScape)                 Address Input
       │                                │
       │ PLY upload                     │ geocode + satellite lookup
       ▼                                ▼
   Scan Processing              Pre-population Pipeline
   (PLY → glTF + PNG)          (lot, trees, sun baseline)
       │                                │
       └──────────┬─────────────────────┘
                  ▼
          Project Model (PostGIS)
          ┌─────────────────────┐
          │ GeoJSON + scan ref  │
          │ + baseline + zones  │
          └─────────┬───────────┘
                    │
       ┌────────────┼────────────┐
       ▼            ▼            ▼
  Zone Editor   Plant.id     AI Layer
  (draw/edit)   (identify)   (recommend)
       │            │            │
       └────────────┼────────────┘
                    │
          Material Assignment × 3 Tiers
                    │
       ┌────────────┼────────────┬──────────┐
       ▼            ▼            ▼          ▼
   Quote Engine  3D Generator  PDF Gen   DXF Export
       │            │            │          │
       ▼            ▼            ▼          ▼
   Line Items    glTF scenes  Branded    CAD file
   × 3 tiers     × 3 tiers    PDF
       │            │            │          │
       └────────────┼────────────┼──────────┘
                    ▼
              Client Reviews
              Approves a Tier
                    │
                    ▼
              Crew Builds It
```

-----

## Monorepo Structure

```
plantastic/
├── Cargo.toml                        # workspace root
├── README.md
├── docs/
│   ├── specification.md              # this document
│   ├── knowledge/                    # research, workflow definitions
│   └── active/                       # tickets, stories, work artifacts
│
├── crates/
│   ├── pt-geo/                       # geometry & spatial math
│   │                                 #   polygon area, perimeter, volume
│   │                                 #   boolean ops (union, subtract)
│   │                                 #   simplification, projection
│   │                                 #   pure functions, no I/O
│   │
│   ├── pt-solar/                     # solar radiance engine
│   │                                 #   sun position calculation
│   │                                 #   shadow modeling (terrain, trees)
│   │                                 #   effective sun hours grid
│   │                                 #   light category classification
│   │                                 #   pure computation, no I/O
│   │
│   ├── pt-climate/                   # climate data models
│   │                                 #   frost dates, growing season
│   │                                 #   Sunset Western Garden zones
│   │                                 #   microclimate classification
│   │                                 #   hardiness zones
│   │
│   ├── pt-plants/                    # plant intelligence
│   │                                 #   plant database model
│   │                                 #   multi-factor scoring engine
│   │                                 #   recommendation generation
│   │                                 #   Plant.id API client
│   │
│   ├── pt-project/                   # project domain model
│   │                                 #   Project, Zone, Tier, MaterialAssignment
│   │                                 #   GeoJSON serialization
│   │                                 #   status lifecycle
│   │                                 #   the source of truth
│   │
│   ├── pt-materials/                 # material catalog domain
│   │                                 #   Material, category, unit, pricing
│   │                                 #   extrusion behavior
│   │                                 #   texture/photo references
│   │                                 #   tenant catalog layering
│   │
│   ├── pt-quote/                     # quote engine
│   │                                 #   quantity takeoff (geo + materials)
│   │                                 #   line items, subtotals, tiers
│   │                                 #   pure computation, no I/O
│   │
│   ├── pt-scan/                      # scan processing pipeline
│   │                                 #   PLY parsing
│   │                                 #   point cloud filtering
│   │                                 #   ground plane fitting (RANSAC)
│   │                                 #   mesh generation (Delaunay)
│   │                                 #   mesh decimation
│   │                                 #   orthographic projection → PNG
│   │                                 #   glTF export
│   │
│   ├── pt-scene/                     # 3D scene generator
│   │                                 #   terrain mesh loading
│   │                                 #   2D → 3D polygon projection
│   │                                 #   extrusion by material behavior
│   │                                 #   UV generation for texture tiling
│   │                                 #   PBR material application
│   │                                 #   glTF binary assembly
│   │
│   ├── pt-pdf/                       # PDF generation (typst)
│   │                                 #   branded quote document
│   │                                 #   plan view with overlays
│   │                                 #   tier comparison tables
│   │                                 #   material photos
│   │                                 #   planting calendar
│   │
│   ├── pt-dxf/                       # DXF export (ixmilia/dxf)
│   │                                 #   zones → LWPOLYLINE per layer
│   │                                 #   dimensions, annotations
│   │                                 #   material labels
│   │
│   ├── pt-satellite/                 # satellite pre-population
│   │                                 #   Meta canopy height data
│   │                                 #   municipal parcel data
│   │                                 #   tree detection
│   │                                 #   baseline project generation
│   │
│   └── pt-tenant/                    # tenant/brand domain
│                                     #   company profile, branding
│                                     #   catalog ownership
│                                     #   client management
│                                     #   auth context
│
├── apps/
│   ├── api/                          # Axum backend (Lambda target)
│   │   ├── src/
│   │   │   ├── main.rs              # router + Lambda runtime detection
│   │   │   ├── routes/              # REST + SSE endpoints
│   │   │   ├── auth/                # tenant auth, session management
│   │   │   ├── jobs/                # async job queue (scan, scene gen)
│   │   │   └── streaming/           # SSE adapter for BAML AI responses
│   │   └── Cargo.toml
│   │
│   └── viewer/                       # Bevy 3D viewer (WASM target)
│       ├── src/
│       │   ├── main.rs              # Bevy app entry
│       │   ├── camera.rs            # orbit camera controls
│       │   ├── scene.rs             # glTF scene loading
│       │   ├── interaction.rs       # tap-to-inspect, measurement
│       │   ├── tiers.rs             # tier toggle
│       │   └── lighting.rs          # sunlight direction control
│       └── Cargo.toml
│
├── web/                              # SvelteKit frontend (CF Pages)
│   ├── src/
│   │   ├── routes/
│   │   │   ├── +page.svelte         # landing / marketing
│   │   │   ├── dashboard/           # landscaper project list
│   │   │   ├── project/[id]/        # project workspace
│   │   │   │   ├── editor/          # 2D zone drawing
│   │   │   │   ├── materials/       # material assignment per tier
│   │   │   │   ├── quote/           # quote review + comparison
│   │   │   │   ├── viewer/          # embedded 3D viewer
│   │   │   │   └── export/          # PDF + DXF downloads
│   │   │   ├── catalog/             # material catalog management
│   │   │   ├── settings/            # tenant branding, account
│   │   │   └── client/[token]/      # client-facing view (branded)
│   │   └── lib/
│   │       ├── api/                 # API client, SSE streaming
│   │       ├── stores/              # Svelte state (project, session)
│   │       ├── components/          # shared UI components
│   │       └── utils/               # geo helpers, formatting
│   ├── svelte.config.js
│   ├── tailwind.config.js
│   └── package.json
│
├── worker/                           # Cloudflare Worker proxy
│   ├── src/
│   │   └── index.ts                 # CORS, rate limiting, auth, SSE passthrough
│   └── wrangler.toml
│
├── baml_src/                         # BAML definitions (AI layer)
│   ├── clients.baml                  # LLM client configs
│   ├── types.baml                    # structured types
│   ├── recommend.baml                # plant recommendation function
│   ├── describe.baml                 # zone/quote narrative generation
│   └── intent.baml                   # natural language → structured zone intent
│
├── assets/
│   ├── textures/                     # default PBR texture sets
│   └── models/                       # plant models, furniture (future)
│
├── migrations/                       # PostgreSQL/PostGIS schema
│   ├── 001_tenants.sql
│   ├── 002_projects.sql
│   ├── 003_materials.sql
│   └── 004_plants.sql
│
└── infra/
    ├── sst.config.ts                 # SST: Lambda functions, S3, secrets
    └── scripts/
        ├── deploy.sh                 # full-stack deploy orchestration
        └── verify-deploy.sh          # post-deploy verification
```

-----

## Crate Dependency Graph

```
pt-geo ←──────────── pt-quote ←── apps/api
  ↑                    ↑
  ├── pt-project ──────┤
  │     ↑              │
  │     ├── pt-scan    │
  │     ├── pt-scene ──┘
  │     └── pt-satellite
  │
  ├── pt-solar ←── pt-plants
  │                  ↑
  ├── pt-climate ────┘
  │
  ├── pt-materials ←── pt-quote
  │                 ←── pt-scene
  │                 ←── pt-pdf
  │
  ├── pt-pdf
  ├── pt-dxf
  └── pt-tenant ←── apps/api
```

Pure computation crates (pt-geo, pt-solar, pt-quote) have no I/O dependencies. Domain model crates (pt-project, pt-materials, pt-tenant) define types and serialization. I/O lives at the edges: apps/api for HTTP, pt-scan for file processing, pt-satellite for external data fetches.

-----

## API Surface

```
# Tenant
POST   /auth/login
POST   /auth/register
GET    /tenant/profile
PATCH  /tenant/profile                   # branding, contact info

# Projects
POST   /projects                         # create from address (triggers pre-population)
GET    /projects                          # list for tenant
GET    /projects/:id
DELETE /projects/:id

# Scan
POST   /projects/:id/scan               # upload PLY (triggers async processing)
GET    /projects/:id/scan/status         # job status

# Zones
GET    /projects/:id/zones
PUT    /projects/:id/zones               # bulk update (from 2D editor)
POST   /projects/:id/zones               # add zone
PATCH  /projects/:id/zones/:zid
DELETE /projects/:id/zones/:zid

# Materials
GET    /materials                         # tenant's catalog
POST   /materials
PATCH  /materials/:id
DELETE /materials/:id

# Tiers & Assignments
GET    /projects/:id/tiers
PUT    /projects/:id/tiers/:tier         # set material assignments for a tier

# Outputs
GET    /projects/:id/quote/:tier         # computed quote
GET    /projects/:id/scene/:tier         # generated glTF (triggers gen if stale)
GET    /projects/:id/pdf                 # branded quote PDF
GET    /projects/:id/dxf                 # DXF export (approved tier)

# Plants
GET    /plants/search                    # search platform plant database
GET    /plants/recommend                 # AI-powered recommendations for a zone
POST   /plants/identify                  # Plant.id integration (photo upload)

# AI (SSE streaming)
POST   /ai/recommend                     # plant recommendations with reasoning
POST   /ai/describe                      # narrative for zone or quote
POST   /ai/intent                        # parse natural language zone description

# Client-facing (branded, token-auth)
GET    /c/:token                         # client view of project
POST   /c/:token/approve/:tier           # client approves a tier
```

-----

## Data Models

### PostgreSQL Schema (abbreviated)

```sql
-- Tenant
CREATE TABLE tenants (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    domain       TEXT,                    -- custom domain (future)
    logo_url    TEXT,
    brand_color TEXT,
    contact     JSONB NOT NULL,          -- {email, phone, address}
    created_at  TIMESTAMPTZ DEFAULT now()
);

-- Project
CREATE TABLE projects (
    id          UUID PRIMARY KEY,
    tenant_id   UUID REFERENCES tenants(id),
    client_name TEXT,
    client_email TEXT,
    address     TEXT NOT NULL,
    location    GEOGRAPHY(POINT, 4326),  -- lat/lng
    scan_ref    JSONB,                   -- {ply_key, terrain_key, planview_key, metadata}
    baseline    JSONB,                   -- {lot_polygon, trees[], sun_grid_key}
    status      TEXT DEFAULT 'draft',    -- draft, quoted, approved, complete
    created_at  TIMESTAMPTZ DEFAULT now(),
    updated_at  TIMESTAMPTZ DEFAULT now()
);

-- Zone (spatial)
CREATE TABLE zones (
    id          UUID PRIMARY KEY,
    project_id  UUID REFERENCES projects(id) ON DELETE CASCADE,
    geometry    GEOMETRY(POLYGON, 4326),
    zone_type   TEXT NOT NULL,           -- bed, patio, path, lawn, wall, edging
    label       TEXT,
    sort_order  INT DEFAULT 0
);

-- Material (per tenant)
CREATE TABLE materials (
    id              UUID PRIMARY KEY,
    tenant_id       UUID REFERENCES tenants(id),
    name            TEXT NOT NULL,
    category        TEXT NOT NULL,        -- hardscape, softscape, edging, fill
    unit            TEXT NOT NULL,        -- sq_ft, cu_yd, linear_ft, each
    price_per_unit  DECIMAL(10,2),
    depth_inches    REAL,
    extrusion       JSONB NOT NULL,       -- {type, height?, flush?}
    texture_key     TEXT,                 -- S3 key for PBR texture set
    photo_key       TEXT,                 -- S3 key for product photo
    supplier_sku    TEXT
);

-- Tier assignment
CREATE TABLE tier_assignments (
    id          UUID PRIMARY KEY,
    project_id  UUID REFERENCES projects(id) ON DELETE CASCADE,
    tier        TEXT NOT NULL,            -- good, better, best
    zone_id     UUID REFERENCES zones(id) ON DELETE CASCADE,
    material_id UUID REFERENCES materials(id),
    overrides   JSONB,                   -- custom depth, pattern, etc.
    UNIQUE(project_id, tier, zone_id)
);

-- Plant database (platform-level)
CREATE TABLE plants (
    id              UUID PRIMARY KEY,
    common_name     TEXT NOT NULL,
    botanical_name  TEXT,
    min_sun_hours   REAL,
    ideal_sun_min   REAL,
    ideal_sun_max   REAL,
    frost_tolerance TEXT,                -- tender, semi-hardy, hardy
    sunset_zones    INT[],               -- Sunset Western Garden zone numbers
    days_to_maturity_min INT,
    days_to_maturity_max INT,
    mature_height_ft REAL,
    mature_spread_ft REAL,
    water_needs     TEXT,                -- low, moderate, high
    tags            TEXT[],              -- edible, native, pollinator, drought-tolerant
    photo_url       TEXT
);
```

-----

## What's Not in V1

- Native iOS scan app (use SiteScape)
- Plant placement as point entities in 3D (zones only, not individual plant positions)
- Irrigation design
- Lighting design
- Permit-grade drawings
- Scheduling and crew management
- Invoicing and payments
- Supplier ordering integration
- Multi-project portfolio analytics
- Client self-serve sign-up (landscaper invites client in V1)
- Custom domain per tenant
- Offline mode

Each extends naturally from the project model + tenant architecture.
