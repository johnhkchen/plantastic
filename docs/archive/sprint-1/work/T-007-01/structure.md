# T-007-01 Structure: Canvas Polygon Drawing

## File Changes Overview

```
web/src/lib/
├── components/
│   └── zone-editor/
│       ├── ZoneEditor.svelte       # NEW — main component (canvas + toolbar)
│       ├── types.ts                # NEW — EditorZone, ZoneType, state types
│       ├── colors.ts               # NEW — zone type → color mapping
│       ├── hit-test.ts             # NEW — point-in-polygon, vertex proximity
│       └── renderer.ts            # NEW — canvas drawing functions
├── stores/
│   └── project.svelte.ts          # MODIFY — update Zone interface to include geometry
web/src/routes/(app)/project/[id]/
│   └── editor/
│       └── +page.svelte           # MODIFY — mount ZoneEditor component
```

Total: 5 new files, 2 modified files.

---

## New Files

### `web/src/lib/components/zone-editor/types.ts`

Type definitions shared across the zone editor module.

```typescript
// Zone type enum matching backend ZoneType (snake_case)
type ZoneType = 'bed' | 'patio' | 'path' | 'lawn' | 'wall' | 'edging';

// A vertex in canvas coordinates
interface Point { x: number; y: number; }

// A zone as stored during editing
interface EditorZone {
  id: string;
  vertices: Point[];
  zoneType: ZoneType;
  label: string;
}

// Component interaction modes
type EditorMode = 'idle' | 'drawing' | 'selected';

// Drag state for vertex editing
interface DragState {
  zoneId: string;
  vertexIndex: number;
  startPos: Point;
}
```

Exports: `ZoneType`, `Point`, `EditorZone`, `EditorMode`, `DragState`, `ZONE_TYPES` (array).

### `web/src/lib/components/zone-editor/colors.ts`

Maps zone types to fill and stroke colors. Pure data, no logic.

```typescript
interface ZoneColors { fill: string; stroke: string; }
const ZONE_COLOR_MAP: Record<ZoneType, ZoneColors> = { ... };
function getZoneColors(type: ZoneType): ZoneColors;
```

Exports: `ZONE_COLOR_MAP`, `getZoneColors`.

### `web/src/lib/components/zone-editor/hit-test.ts`

Geometry utilities for canvas interaction. Pure functions, no state.

```typescript
// Ray-casting point-in-polygon test
function isPointInPolygon(point: Point, vertices: Point[]): boolean;

// Distance from point to nearest vertex, returns index + distance
function nearestVertex(point: Point, vertices: Point[]): { index: number; distance: number };

// Check if point is close enough to "close" a polygon (near first vertex)
function isNearFirstVertex(point: Point, vertices: Point[], threshold: number): boolean;

// Find which zone (if any) contains a point — checks in reverse order (top-most first)
function findZoneAtPoint(point: Point, zones: EditorZone[]): EditorZone | null;
```

Exports: all four functions.

### `web/src/lib/components/zone-editor/renderer.ts`

Canvas drawing functions. Takes a CanvasRenderingContext2D and draws shapes.
Pure rendering — reads state, writes pixels. No state mutation.

```typescript
// Draw the background grid
function drawGrid(ctx: C2D, width: number, height: number): void;

// Draw a completed zone polygon with fill and stroke
function drawZone(ctx: C2D, zone: EditorZone, isSelected: boolean): void;

// Draw vertex handles for a zone
function drawHandles(ctx: C2D, vertices: Point[]): void;

// Draw the in-progress polygon being drawn
function drawDrawingPolygon(ctx: C2D, vertices: Point[], mousePos: Point | null, zoneType: ZoneType): void;

// Full redraw: clear canvas, draw grid, draw all zones, draw active drawing
function redraw(ctx: C2D, state: RenderState): void;
```

`RenderState` bundles the data needed for a full redraw:
```typescript
interface RenderState {
  zones: EditorZone[];
  selectedZoneId: string | null;
  drawingVertices: Point[];
  mousePos: Point | null;
  activeZoneType: ZoneType;
  mode: EditorMode;
}
```

### `web/src/lib/components/zone-editor/ZoneEditor.svelte`

The main component. ~250 lines. Structure:

```svelte
<script lang="ts">
  // Props
  let { zones = $bindable<EditorZone[]>([]) } = $props();

  // Internal state (runes)
  let mode = $state<EditorMode>('idle');
  let activeZoneType = $state<ZoneType>('bed');
  let drawingVertices = $state<Point[]>([]);
  let selectedZoneId = $state<string | null>(null);
  let dragState = $state<DragState | null>(null);
  let mousePos = $state<Point | null>(null);
  let canvas: HTMLCanvasElement;

  // Derived
  let selectedZone = $derived(zones.find(z => z.id === selectedZoneId) ?? null);
  let ctx = $derived(canvas?.getContext('2d'));

  // Effects
  $effect(() => { /* redraw canvas when any render-relevant state changes */ });

  // Event handlers
  function handleCanvasClick(e: MouseEvent) { ... }
  function handleCanvasDoubleClick(e: MouseEvent) { ... }
  function handleCanvasMouseMove(e: MouseEvent) { ... }
  function handleCanvasMouseDown(e: MouseEvent) { ... }
  function handleCanvasMouseUp(e: MouseEvent) { ... }
  function handleKeyDown(e: KeyboardEvent) { ... }

  // Zone operations
  function finishDrawing() { ... }
  function cancelDrawing() { ... }
  function deleteSelectedZone() { ... }
  function getCanvasPoint(e: MouseEvent): Point { ... }
</script>

<!-- Toolbar: zone type selector + label input -->
<div class="toolbar">
  <!-- Zone type buttons -->
  <!-- Label input (visible when zone selected) -->
  <!-- Delete button (visible when zone selected) -->
</div>

<!-- Canvas -->
<canvas
  bind:this={canvas}
  on:click={handleCanvasClick}
  on:dblclick={handleCanvasDoubleClick}
  on:mousemove={handleCanvasMouseMove}
  on:mousedown={handleCanvasMouseDown}
  on:mouseup={handleCanvasMouseUp}
  on:contextmenu|preventDefault
  class="..."
/>

<!-- Status bar: mode indicator, zone count, instructions -->
<div class="status-bar">...</div>
```

---

## Modified Files

### `web/src/lib/stores/project.svelte.ts`

Update the `Zone` interface to carry geometry data compatible with the editor:

```typescript
// Before
interface Zone { id: string; name: string; area: number; }

// After
interface Zone {
  id: string;
  vertices: { x: number; y: number }[];
  zoneType: 'bed' | 'patio' | 'path' | 'lawn' | 'wall' | 'edging';
  label: string;
  area: number;  // computed, for display
}
```

The `name` field becomes `label` to match the backend. `vertices` and `zoneType` added.
Update the mock zones data reference in mock.ts accordingly.

### `web/src/routes/(app)/project/[id]/editor/+page.svelte`

Replace placeholder with actual editor:

```svelte
<script lang="ts">
  import ZoneEditor from '$lib/components/zone-editor/ZoneEditor.svelte';
  import type { EditorZone } from '$lib/components/zone-editor/types';

  let zones = $state<EditorZone[]>([]);
</script>

<div class="h-full flex flex-col">
  <h2 class="text-lg font-semibold text-gray-900 px-4 py-3">Zone Editor</h2>
  <ZoneEditor bind:zones />
</div>
```

---

## Module Boundaries

The `zone-editor/` directory is a self-contained module. Its public interface is:

- `ZoneEditor.svelte` — the component to mount
- `types.ts` — type exports (`EditorZone`, `ZoneType`, `Point`) for consumers

Internal modules (`colors.ts`, `hit-test.ts`, `renderer.ts`) are implementation details.
They're separate files for testability and readability, not for external consumption.

---

## Dependency Graph

```
ZoneEditor.svelte
  ├── types.ts       (shared types)
  ├── colors.ts      (zone type colors)
  ├── hit-test.ts    (geometry utilities)
  └── renderer.ts    (canvas drawing)
        ├── types.ts
        └── colors.ts
```

No circular dependencies. All arrows point inward (types.ts is the leaf).

---

## Sizing Estimates

| File | Lines (est.) | Complexity |
|------|-------------|------------|
| types.ts | ~40 | Low — type definitions only |
| colors.ts | ~30 | Low — data mapping |
| hit-test.ts | ~60 | Medium — geometry math |
| renderer.ts | ~120 | Medium — canvas draw calls |
| ZoneEditor.svelte | ~250 | High — state machine + event handling |
| editor/+page.svelte | ~15 | Low — mount component |
| project.svelte.ts | ~10 changed | Low — interface update |
| **Total** | **~525** | |
