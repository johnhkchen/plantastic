# T-007-01 Plan: Canvas Polygon Drawing

## Implementation Steps

### Step 1: Type definitions and color mapping

Create `types.ts` and `colors.ts` — the data foundation everything else imports.

Files:
- Create `web/src/lib/components/zone-editor/types.ts`
- Create `web/src/lib/components/zone-editor/colors.ts`

Verify: TypeScript compiles, types are importable.

### Step 2: Hit-testing utilities

Create `hit-test.ts` — pure geometry functions for canvas interaction.

Files:
- Create `web/src/lib/components/zone-editor/hit-test.ts`

Verify: Functions are pure and importable. Can verify point-in-polygon logic mentally
(ray casting is well-established). Formal tests would be ideal but this ticket is
frontend-only and the project doesn't have a frontend test runner configured yet.

### Step 3: Canvas renderer

Create `renderer.ts` — all canvas drawing functions.

Files:
- Create `web/src/lib/components/zone-editor/renderer.ts`

Verify: Functions accept ctx + data, return void. No side effects beyond canvas pixels.

### Step 4: Main ZoneEditor component

Create `ZoneEditor.svelte` — the state machine with canvas event handling.

Files:
- Create `web/src/lib/components/zone-editor/ZoneEditor.svelte`

This is the largest step. Key behaviors to implement:
1. Canvas setup with resize handling
2. IDLE mode: click starts drawing, click existing zone selects it
3. DRAWING mode: click adds vertex, double-click/close finishes, Escape cancels
4. SELECTED mode: drag handles to move vertices, Delete removes zone
5. Toolbar: zone type buttons, label input, delete button
6. Status bar: mode indicator, helpful instructions
7. `$effect` for reactive canvas redraw
8. `$bindable` zones prop for parent communication

Verify: Mount in browser, draw a polygon, select it, edit it, delete it.

### Step 5: Update store types and mount component

Update the project store Zone interface and mount ZoneEditor on the editor page.

Files:
- Modify `web/src/lib/stores/project.svelte.ts` — update Zone interface
- Modify `web/src/routes/(app)/project/[id]/editor/+page.svelte` — mount ZoneEditor
- Modify `web/src/lib/api/mock.ts` — update mock zone data shape

Verify: Editor page shows the canvas. Full drawing flow works end to end.

### Step 6: Polish and verify

- Test all acceptance criteria manually
- Check TypeScript compilation (`npm run check` in web/)
- Check formatting (`npm run lint` in web/)
- Verify multiple zones, zone type switching, label editing
- Verify keyboard shortcuts (Escape to cancel, Delete to remove)

Verify: `cd web && npm run check` passes. All acceptance criteria met.

---

## Testing Strategy

This is a frontend-only UI component. The project's test infrastructure is Rust-focused
(cargo test, pt-scenarios). There is no frontend test runner (no vitest, no playwright).

**What can be verified automatically:**
- TypeScript compilation: `npm run check` in `web/`
- Lint/format: `npm run lint` in `web/`
- Rust workspace: `just check` should remain green (no Rust changes)

**What requires manual verification:**
- Canvas drawing interactions (click, double-click, drag)
- Visual rendering (zone fills, handles, preview edges)
- State machine transitions (idle → drawing → idle, idle → selected → idle)
- Keyboard handling (Escape, Delete/Backspace)

**Pure function testability:**
The `hit-test.ts` functions are pure and deterministic — they're testable if/when
a frontend test runner is added. Same for `renderer.ts` (with a mock canvas context).
Structuring these as separate modules is intentional preparation for testability.

---

## Commit Strategy

1. After Step 1-3: Commit utility modules (types, colors, hit-test, renderer)
2. After Step 4-5: Commit component + integration (ZoneEditor, page mount, store update)
3. After Step 6: Commit any polish fixes

---

## Risk Mitigation

**Risk: Canvas sizing/DPI issues**
Canvas must account for `devicePixelRatio` for sharp rendering on Retina displays.
Handle in canvas setup: set canvas width/height attributes to `clientWidth * dpr`,
then scale the context.

**Risk: Double-click also fires two click events**
Use a click delay timer or track click count. Simpler: check vertex count in the
double-click handler — if the polygon has fewer than 3 vertices, don't close it.
The two preceding clicks will have added vertices, and the dblclick handler removes
the extra vertex added by the second click before closing.

**Risk: State machine bugs**
Keep mode transitions explicit. Each event handler checks `mode` first and only
acts on valid transitions. Invalid state combinations should be impossible by
construction.
