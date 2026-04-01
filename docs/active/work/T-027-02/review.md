# T-027-02 Review: Responsive Editor & Viewer

## Summary

Made the zone editor, 3D viewer controls, and app sidebar responsive for tablet crew handoff (S.4.1). Replaced mouse-only events with pointer events in the zone editor for touch support. All changes are CSS-first using Tailwind responsive prefixes, following patterns established in T-027-01.

## Files Modified

| File | Change |
|------|--------|
| `web/src/lib/components/Sidebar.svelte` | Responsive sidebar: fixed overlay on mobile (<768px), static on desktop. Accepts `open`/`onClose` props. Nav items increased to 44px tap targets. |
| `web/src/lib/components/Header.svelte` | Added hamburger menu button (md:hidden) with `onToggleSidebar` prop. 44px tap target. |
| `web/src/routes/(app)/+layout.svelte` | Added `sidebarOpen` state, mobile backdrop overlay, Escape key handler. Wires sidebar/header props. |
| `web/src/routes/(app)/project/[id]/editor/+page.svelte` | Measurements panel: stacks below canvas on mobile (`flex-col md:flex-row`), max-h-48 on mobile, full-width on desktop. |
| `web/src/lib/components/zone-editor/ZoneEditor.svelte` | Pointer events replace mouse events. Touch-action:none on canvas. Dynamic hit radii (touch: 18/20px, mouse: 10/12px). Pointer capture for drag. Toolbar buttons 44px tap targets. |
| `web/src/routes/(app)/project/[id]/viewer/+page.svelte` | Slider: w-full max-w-48 (fills on mobile). Tier buttons: 44px tap targets. Container: flex-wrap. |

## Acceptance Criteria Coverage

- [x] Canvas scales to fill available width (preserving aspect ratio) — was already working via ResizeObserver, now also works in stacked layout
- [x] Measurements panel moves below canvas on narrow viewports — flex-col on mobile
- [x] Touch events work for vertex placement and dragging — pointer events with setPointerCapture
- [x] Iframe fills container responsively — was already w-full aspect-video (unchanged)
- [x] Controls reflow below viewer on narrow viewports — already vertical, now slider fills width with flex-wrap
- [x] Touch orbit/pan/zoom supported by bevy_panorbit_camera — delegated to iframe (T-014-01)
- [x] Sidebar collapses to hamburger on <768px — md:hidden hamburger, fixed overlay sidebar
- [x] Overlay when open, not push-content — fixed positioning with z-50, backdrop at z-40
- [x] `just check` passes — all gates green

## Scenario Dashboard

Before: 58/240 min effective savings (24.2%)
After: 58/240 min effective savings (24.2%)

No change expected — this ticket is UX polish, not new capability. No scenarios regressed.

## Test Coverage

This ticket is pure frontend CSS/UI work. No new Rust tests needed. Verification:
- `just fmt-check` — passes
- `just lint` — passes (Clippy strict, warnings-as-errors)
- `just test` — all workspace tests pass
- `just scenarios` — no regressions

The pointer event change is the highest-risk modification. `onclick` and `ondblclick` are kept unchanged (they fire for both mouse and touch), so the click→draw→close flow works identically. The pointer event handlers (`onpointerdown`, `onpointermove`, `onpointerup`) map 1:1 to their mouse equivalents since `PointerEvent` extends `MouseEvent`.

## Open Concerns

1. **Touch testing is visual-only.** No automated test verifies touch interaction on the canvas. This is inherent to the canvas-based editor — it would require a browser automation tool (Playwright with touch emulation). Not blocked, but noted.

2. **Double-click on touch.** `ondblclick` fires on touch devices but is awkward UX. The primary close-polygon flow (tap first vertex) works well on touch; double-click is a secondary path. Acceptable for now.

3. **Sidebar animation.** The `transition-transform duration-200` is CSS-only — no reduced-motion media query. Could add `motion-reduce:transition-none` in a future polish pass if needed.

4. **Escape key on layout.** Both the app layout and ZoneEditor listen for Escape. Layout closes sidebar, ZoneEditor cancels drawing/deselects. Both fire — sidebar closes AND drawing cancels. This is acceptable behavior (both are "cancel" semantics) but could be surprising. Low priority.
