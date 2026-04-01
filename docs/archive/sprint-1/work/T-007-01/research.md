# T-007-01 Research: Canvas Polygon Drawing

## Scope

Frontend-only Svelte component for drawing, editing, and managing landscape zone polygons
on a canvas. No backend dependency — API wiring is T-007-02.

---

## Existing Frontend State

### SvelteKit Project (`web/`)

- **Framework**: SvelteKit 2.50 + Svelte 5.54 (runes mode enabled)
- **Styling**: Tailwind CSS 4.2 with brand theme (sage green `#2d6a4f`)
- **Build**: Vite 7.3, Cloudflare adapter
- **Formatting**: Prettier (tabs, single quotes, 100 width)
- **No canvas/drawing libraries installed** — zero production dependencies in package.json

### Editor Route

`web/src/routes/(app)/project/[id]/editor/+page.svelte` — placeholder "Coming soon."
This is where the polygon drawing component will be mounted.

### Project Store (`web/src/lib/stores/project.svelte.ts`)

Current Zone type is minimal and misaligned with the backend:
```typescript
interface Zone {
  id: string;
  name: string;   // backend uses `label: Option<String>`
  area: number;    // backend computes area from geometry
}
```

Missing: `geometry` (coordinate array), `zone_type` (enum), and the full polygon model.
The store will need an updated Zone type that matches what this component emits.

### Mock API (`web/src/lib/api/mock.ts`)

Mock zones are flat objects with `id`, `name`, `area` — no geometry data.
Mock data will need updating when T-007-02 wires the API, but this ticket
doesn't need to touch the mock layer since the component is self-contained.

### Existing Components

Three components in `web/src/lib/components/`:
- `Header.svelte` — top bar with tenant/user info
- `Sidebar.svelte` — main nav
- `TabNav.svelte` — project-level tab nav (Editor tab already wired)

No canvas or drawing components exist yet.

---

## Backend Data Model (compatibility target)

### Zone Types (`crates/pt-project/src/types.rs`)

```rust
enum ZoneType { Bed, Patio, Path, Lawn, Wall, Edging }

struct Zone {
    id: ZoneId,                    // UUID
    geometry: Polygon<f64>,        // geo crate — exterior ring + optional holes
    zone_type: ZoneType,
    label: Option<String>,
}
```

Serialization: `ZoneType` uses `#[serde(rename_all = "snake_case")]`.
Geometry serializes as GeoJSON polygon (via `serde_helpers::geojson_polygon`).

### Coordinate System

All coordinates are in **linear feet** (not lat/lon). Origin is arbitrary per project.
The canvas will work in a local coordinate space (pixels) that maps to feet via a
scale factor set when the plan view image is loaded (future ticket).

### Geometry Operations (`crates/pt-geo/`)

Backend computes area (sq ft), perimeter (linear ft), volume (cu ft/cu yd) from polygons.
The frontend component should compute display-only measurements in pixels/local units.
Authoritative measurements come from the backend via pt-geo — the frontend values are
for visual feedback only.

---

## Acceptance Criteria Analysis

| Criterion | Complexity | Notes |
|-----------|-----------|-------|
| Click to place vertices, double-click/close to finish | Medium | Core drawing state machine |
| Zone type selector (6 types) | Low | Dropdown/toolbar, maps to ZoneType enum |
| Label text input | Low | Optional string field |
| Visual feedback: handles, edge preview, fill by type | Medium | Canvas rendering with zone-type colors |
| Edit mode: drag vertices, delete zone | Medium | Hit-testing, drag state, selection |
| Multiple zones on one canvas | Medium | Z-ordering, selection management |
| Emits zone data to parent | Low | Svelte event dispatch / bindable prop |
| No API calls | Constraint | Pure UI component |

---

## Technology Landscape

### Canvas API vs SVG

**Canvas API (HTML5 `<canvas>`)**
- Pros: Fast rendering for many polygons, pixel-level control, efficient hit-testing
- Cons: No DOM elements (manual hit-testing), manual event handling, no built-in accessibility

**SVG**
- Pros: DOM-based (easy event handling per element), built-in transforms, CSS styling
- Cons: Performance degrades with many elements, harder to do custom rendering

**Assessment**: For a polygon drawing tool with drag handles and real-time preview,
Canvas API is the standard choice. SVG would work for small polygon counts but Canvas
scales better and gives more control over the drawing experience.

### Third-Party Libraries

**Fabric.js** — Full canvas abstraction with object model, selection, transforms.
Heavy (~300KB), opinionated API, may conflict with Svelte's reactivity model.

**Konva.js** — Similar to Fabric, lighter. Has a Svelte wrapper (`svelte-konva`).
Still adds significant bundle weight for what is essentially polygon drawing.

**Paper.js** — Vector graphics on canvas. Good for paths but complex API.

**Raw Canvas API** — Zero dependency. Full control. More code to write but the
polygon drawing primitives (moveTo, lineTo, closePath, fill, stroke) are simple.

**Assessment**: The acceptance criteria describe a focused polygon tool, not a general
canvas editor. The raw Canvas API plus Svelte reactivity is sufficient and avoids
dependency bloat. The project currently has zero production dependencies — keeping it
that way is a strength.

---

## Patterns and Constraints

1. **Svelte 5 Runes**: Component must use `$state`, `$derived`, `$effect` — not legacy stores.
2. **Tailwind CSS**: Styling should use Tailwind utilities where possible, inline styles for canvas.
3. **Component Location**: New components go in `web/src/lib/components/`.
4. **TypeScript**: Strict mode, all types explicit.
5. **No production deps**: Preferred — use raw Canvas API.
6. **Coordinate space**: Pixels on canvas, mapped to feet later. For now, 1 pixel = 1 unit.
7. **Zone colors**: Need a color map for 6 zone types. Should use the brand palette as base.

---

## Risks and Open Questions

1. **Touch support**: iPad Safari is a target (spec mentions crew field use). Touch events
   for polygon drawing need consideration but may be a follow-up ticket.
2. **Undo/redo**: Not in acceptance criteria but users will expect it. Can be added later
   with a command stack pattern.
3. **Snap-to-grid**: Spec mentions snap-to-grid and edge snapping. Not in this ticket's
   acceptance criteria — likely a future enhancement.
4. **Plan view background**: This ticket uses a placeholder background. Real plan view PNG
   comes from scan processing (different epic).
5. **Coordinate mapping**: 1:1 pixel-to-unit for now. Real mapping needs scan metadata.
