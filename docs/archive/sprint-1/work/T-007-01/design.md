# T-007-01 Design: Canvas Polygon Drawing

## Decision: Raw Canvas API + Svelte 5 Runes

Use the HTML5 Canvas API directly, managed by Svelte 5's reactive primitives.
No third-party canvas libraries.

---

## Options Evaluated

### Option A: Fabric.js

Fabric provides a full object model (shapes, groups, selection, transforms) on top of Canvas.
- **Pro**: Built-in selection, dragging, object serialization
- **Con**: ~300KB bundle, jQuery-era API style, fights Svelte's reactivity (Fabric maintains
  its own object graph — Svelte state and Fabric state would diverge and need manual sync)
- **Con**: Overkill — we need polygon drawing, not a general-purpose canvas editor
- **Rejected**: Too heavy, wrong abstraction level, reactivity mismatch

### Option B: Konva.js + svelte-konva

Konva is lighter than Fabric, and svelte-konva provides Svelte component wrappers.
- **Pro**: Declarative API closer to Svelte's model
- **Con**: svelte-konva targets Svelte 4 — Svelte 5 runes compatibility is uncertain
- **Con**: Still adds ~150KB for functionality we don't fully need
- **Rejected**: Svelte 5 compatibility risk, unnecessary abstraction

### Option C: SVG-based drawing

Render polygons as SVG `<polygon>` elements with Svelte `{#each}` blocks.
- **Pro**: DOM-native events (click, drag), CSS styling, Svelte reactivity is natural
- **Pro**: Accessible — SVG elements can have ARIA attributes
- **Con**: Performance with many polygons (not a concern at realistic zone counts <50)
- **Con**: Less control over rendering (no pixel-level effects, harder custom cursors)
- **Viable alternative**: Would work, but Canvas gives better UX for drawing interactions

### Option D: Raw Canvas API (chosen)

Direct Canvas 2D context calls, with Svelte runes managing all state.
- **Pro**: Zero dependencies, zero bundle impact
- **Pro**: Full control over rendering, hit-testing, cursors
- **Pro**: State lives entirely in Svelte runes — canvas is a pure render target
- **Pro**: Matches the project's zero-production-deps approach
- **Con**: Manual hit-testing (point-in-polygon, point-near-edge, point-near-vertex)
- **Con**: Manual redraw management (mitigated by `$effect` triggering redraws)
- **Chosen**: Best fit for focused polygon tool with minimal dependencies

---

## Architecture

### State Machine

The component operates in three modes:

```
IDLE  ──(click canvas)──▶  DRAWING  ──(double-click / close)──▶  IDLE
  │                            │
  │                            ├──(click)── add vertex
  │                            ├──(mousemove)── preview edge
  │                            └──(Escape)── cancel
  │
  └──(click zone)──▶  SELECTED  ──(click empty)──▶  IDLE
                          │
                          ├──(drag handle)── move vertex
                          ├──(Delete/Backspace)── delete zone
                          └──(type/label change)── update zone
```

### Data Flow

```
ZoneEditor (canvas + toolbar)
  │
  ├─ $state: zones[]           ← array of EditorZone objects
  ├─ $state: mode              ← 'idle' | 'drawing' | 'selected'
  ├─ $state: activeZoneType    ← current zone type for new zones
  ├─ $state: drawingVertices   ← vertices being placed during drawing
  ├─ $state: selectedZoneId    ← which zone is selected for editing
  ├─ $state: dragState         ← vertex drag tracking
  │
  ├─ $effect: redraw canvas when any state changes
  │
  └─ bindable prop: zones      ← parent reads the zone array
```

The parent component binds to `zones` to receive the zone data.
No events needed — Svelte 5 `$bindable()` makes the parent reactive to changes.

### Zone Data Shape (Frontend)

```typescript
interface EditorZone {
  id: string;                           // UUID
  vertices: { x: number; y: number }[]; // ordered polygon vertices
  zoneType: ZoneType;                   // 'bed' | 'patio' | 'path' | 'lawn' | 'wall' | 'edging'
  label: string;                        // empty string if no label
}
```

This maps directly to the backend `Zone` struct:
- `id` → `ZoneId(Uuid)`
- `vertices` → `Polygon<f64>` exterior ring coordinates
- `zoneType` → `ZoneType` enum (snake_case serialization matches)
- `label` → `Option<String>` (empty string → None)

### Zone Type Color Scheme

Each zone type gets a distinct fill color (with alpha) for visual differentiation:

| Type   | Fill (30% alpha)         | Stroke           | Rationale |
|--------|--------------------------|-------------------|-----------|
| Bed    | `rgba(139, 69, 19, 0.3)` | `#8B4513`        | Earth/soil brown |
| Patio  | `rgba(128, 128, 128, 0.3)` | `#808080`      | Stone gray |
| Path   | `rgba(210, 180, 140, 0.3)` | `#D2B48C`      | Sand/tan |
| Lawn   | `rgba(34, 139, 34, 0.3)` | `#228B22`        | Grass green |
| Wall   | `rgba(105, 105, 105, 0.3)` | `#696969`      | Dark gray |
| Edging | `rgba(160, 82, 45, 0.3)` | `#A0522D`        | Sienna border |

### Hit-Testing Strategy

Canvas doesn't provide built-in element selection. Manual hit-testing required:

1. **Vertex hit**: Check if click is within 8px of any vertex handle → start drag
2. **Polygon hit**: Point-in-polygon test (ray casting algorithm) → select zone
3. **Close-polygon hit**: During drawing, check if click is within 12px of first vertex → close

Ray casting is simple and fast for convex and concave polygons:
```
Cast horizontal ray from point, count intersections with polygon edges.
Odd count = inside, even count = outside.
```

### Rendering Order

1. Grid background (light gray, 20px spacing — placeholder for plan view)
2. Completed zones (filled polygons, bottom to top by creation order)
3. Selected zone highlight (thicker stroke, visible handles)
4. Drawing-in-progress polygon (dashed edges, vertex dots)
5. Cursor preview edge (from last vertex to mouse position)

---

## Rejected Alternatives

### Event-driven architecture (CustomEvent dispatch)

Could emit `zone:created`, `zone:updated`, `zone:deleted` events instead of bindable prop.
Rejected: Svelte 5's `$bindable()` is simpler and more idiomatic. Events add ceremony
without benefit when the parent just needs the current zone array.

### Separate toolbar component

Could extract the zone type selector and label input into a separate component.
Rejected: The toolbar is tightly coupled to the canvas state (active zone type, selected
zone). Splitting adds prop-drilling without meaningful reuse. Keep it in one component,
extract later if needed.

### GeoJSON as internal format

Could store zones as GeoJSON internally for backend compatibility.
Rejected: GeoJSON is a serialization format, not an editing format. The simple
`{ x, y }[]` array is easier to manipulate during drawing and editing. Conversion
to GeoJSON happens at the boundary (when emitting to parent / sending to API in T-007-02).

---

## Scope Boundaries

**In scope (this ticket)**:
- Polygon drawing with click-to-place vertices
- Double-click or click-near-first-vertex to close polygon
- Zone type selector toolbar
- Label input for selected zone
- Visual feedback (handles, preview edge, fill by type)
- Edit mode (drag vertices, delete zone)
- Multiple zones on one canvas
- Emit zone data to parent via bindable prop

**Out of scope (future tickets)**:
- API integration (T-007-02)
- Snap-to-grid / edge snapping
- Undo/redo
- Touch events / iPad optimization
- Plan view image background
- Boolean zone operations (subtract, split)
- Real-unit measurements display
