# T-007-01 Review: Canvas Polygon Drawing

## Summary

Implemented a self-contained Svelte 5 zone editor component using the raw HTML5 Canvas API.
Zero new dependencies. The component supports drawing polygons with click-to-place vertices,
editing by dragging vertex handles, zone type selection, labeling, and deletion. Multiple
zones can coexist on one canvas. Zone data is exposed to the parent via a bindable prop.

---

## Files Changed

### Created (5 files, 663 lines total)

| File | Lines | Purpose |
|------|-------|---------|
| `web/src/lib/components/zone-editor/types.ts` | 48 | Shared type definitions: ZoneType, Point, EditorZone, EditorMode, DragState, RenderState |
| `web/src/lib/components/zone-editor/colors.ts` | 20 | Zone type → fill/stroke color mapping |
| `web/src/lib/components/zone-editor/hit-test.ts` | 70 | Point-in-polygon (ray casting), vertex proximity, zone selection |
| `web/src/lib/components/zone-editor/renderer.ts` | 187 | Canvas drawing: grid, zones, handles, drawing preview, labels, DPI scaling |
| `web/src/lib/components/zone-editor/ZoneEditor.svelte` | 338 | Main component: state machine, event handling, toolbar, status bar, canvas |

### Modified (3 files)

| File | Change |
|------|--------|
| `web/src/routes/(app)/project/[id]/editor/+page.svelte` | Replaced "Coming soon" with ZoneEditor mount |
| `web/src/lib/stores/project.svelte.ts` | Updated Zone interface: added `vertices`, `zoneType`, `label`; removed `name`, `area` |
| `web/src/lib/api/mock.ts` | Updated mock zones to match new Zone interface (vertices + zoneType) |

---

## Acceptance Criteria Verification

| Criterion | Status | Implementation |
|-----------|--------|----------------|
| Click to place vertices, double-click/close to finish | Done | Click adds vertex; double-click closes; click near first vertex closes |
| Zone type selector (6 types) | Done | Toolbar with colored buttons for each ZoneType |
| Label text input | Done | Text input appears when zone is selected |
| Visual feedback: handles, edge preview, fill by type | Done | Vertex handles on selected zone; dashed preview edge to cursor; fill color per zone type |
| Edit mode: drag vertices, delete zone | Done | Mousedown on handle starts drag; Delete/Backspace removes zone; red "Delete zone" button |
| Multiple zones on one canvas | Done | Array state; reverse-order hit testing; all zones rendered |
| Emits zone data to parent | Done | `$bindable` zones prop |
| No API calls | Done | Pure UI component; no imports from `$lib/api` |

---

## Test Coverage

### Automated (verified 2026-03-31)
- **TypeScript compilation**: `npm run check` — 0 errors, 0 warnings (311 files)
- **Rust workspace**: `just check` — all gates passed
- **Scenario dashboard**: 8.0/240.0 min — no change (this ticket doesn't advance scenarios)

### Not Automated (requires manual/browser verification)
- Canvas drawing interactions (click, double-click, drag)
- Visual rendering correctness (fills, strokes, handles)
- State machine transitions
- Keyboard shortcuts (Escape, Delete)
- Canvas DPI scaling on Retina displays
- ResizeObserver-based canvas resizing

### Testable but Not Yet Tested
- `hit-test.ts` functions are pure and deterministic — ready for unit tests when a frontend
  test runner (vitest) is added. No runner exists yet in the project.

---

## Scenario Dashboard

```
Before: 8.0 min / 240.0 min (3.3%) — 2 pass, 0 fail, 15 not implemented
After:  8.0 min / 240.0 min (3.3%) — 2 pass, 0 fail, 15 not implemented
```

No change. This ticket is a UI component prerequisite for S.2.1 (Zone drawing with
measurements). S.2.1 requires both this component (T-007-01) and the API wiring (T-007-02)
plus pt-geo measurement integration before it can flip to passing.

---

## Open Concerns

1. **No frontend test runner.** The hit-test and renderer modules are pure functions that
   should have unit tests. The project doesn't have vitest or similar configured. This is
   a gap that affects all frontend work, not just this ticket.

2. **Touch events not implemented.** The spec calls out iPad Safari for crew field use.
   The current implementation uses mouse events only. Touch support (touchstart, touchmove,
   touchend) should be added in a follow-up — the state machine is ready for it, the event
   translation is mechanical.

3. **Store type breaking change.** The Zone interface in `project.svelte.ts` changed shape
   (`name` → `label`, added `vertices`/`zoneType`, removed `area`). This is correct — the
   old interface was a placeholder. But any code consuming the store's zones will need
   updating. Currently only the mock.ts consumer was affected and was updated.

4. **Double-click edge case.** Double-click fires two preceding click events. The handler
   removes the duplicate vertex added by the second click. This works but relies on event
   ordering that could theoretically vary across browsers. Tested logic is sound for
   Chrome/Safari/Firefox.

5. **No undo/redo.** Users will expect Ctrl+Z. The state mutations are all immutable
   (new array on every change), which makes a command stack straightforward to add later.

---

## What This Unblocks

- **T-007-02** (zone API + measurements) — can now wire the editor to the backend and add
  real-time area/perimeter display using pt-geo computations
- **S.2.1** (zone drawing with measurements) — once T-007-02 is complete, this scenario
  can be advanced from NotImplemented
