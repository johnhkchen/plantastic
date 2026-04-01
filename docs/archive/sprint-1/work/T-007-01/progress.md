# T-007-01 Progress: Canvas Polygon Drawing

## Completed Steps

### Step 1: Type definitions and color mapping
- Created `web/src/lib/components/zone-editor/types.ts`
  - `ZoneType`, `Point`, `EditorZone`, `EditorMode`, `DragState`, `RenderState`
  - `ZONE_TYPES` array, `ZONE_TYPE_LABELS` map
- Created `web/src/lib/components/zone-editor/colors.ts`
  - `ZONE_COLOR_MAP` with distinct colors per zone type
  - `getZoneColors()` accessor

### Step 2: Hit-testing utilities
- Created `web/src/lib/components/zone-editor/hit-test.ts`
  - `isPointInPolygon()` — ray-casting algorithm
  - `distance()`, `nearestVertex()`, `isNearFirstVertex()`
  - `findZoneAtPoint()` — reverse-order search for topmost zone

### Step 3: Canvas renderer
- Created `web/src/lib/components/zone-editor/renderer.ts`
  - `redraw()` — full canvas redraw with DPI scaling
  - Grid background, zone fill/stroke, vertex handles, drawing preview
  - Close-polygon visual indicator (ring around first vertex)
  - Label rendering at polygon centroid

### Step 4: Main ZoneEditor component
- Created `web/src/lib/components/zone-editor/ZoneEditor.svelte`
  - Three-mode state machine: idle → drawing → selected
  - Canvas event handling: click, dblclick, mousemove, mousedown, mouseup
  - Keyboard: Escape (cancel/deselect), Delete/Backspace (remove zone)
  - Toolbar: zone type buttons with color indicators, label input, delete button
  - Status bar with contextual instructions and zone count
  - `$bindable` zones prop for parent communication
  - ResizeObserver for responsive canvas sizing
  - DPI-aware rendering (devicePixelRatio)

### Step 5: Store types and page mount
- Updated `web/src/lib/stores/project.svelte.ts` — Zone interface now has `vertices`, `zoneType`, `label`
- Updated `web/src/routes/(app)/project/[id]/editor/+page.svelte` — mounts ZoneEditor
- Updated `web/src/lib/api/mock.ts` — mock zones match new interface

### Step 6: Verification
- `npm run check` in web/ — 0 errors, 0 warnings
- `just check` — all Rust tests pass, scenarios unchanged (8.0/240.0 min)
- No regressions

## Deviations from Plan

None. Implementation followed the plan as written.
