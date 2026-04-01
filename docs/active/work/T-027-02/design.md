# T-027-02 Design: Responsive Editor & Viewer

## 1. Sidebar Collapse

### Option A: Svelte state + Tailwind responsive hiding

- Add `sidebarOpen` state to app layout
- Desktop (≥768px): sidebar always visible via `hidden md:flex`, hamburger hidden via `md:hidden`
- Mobile (<768px): sidebar hidden by default, toggled by hamburger button in Header
- Overlay: `fixed inset-0 z-40` with backdrop, not push-content
- Close on nav click + backdrop click + Escape

**Pros:** Follows T-027-01 pattern (CSS-first, minimal JS). Standard mobile sidebar UX.
**Cons:** State lives in layout, needs to be passed to Header for hamburger button.

### Option B: CSS-only sidebar with checkbox hack

- Hidden checkbox + `peer` classes to toggle sidebar visibility

**Pros:** Zero JS.
**Cons:** Hacky, hard to close on nav click, not accessible, doesn't match codebase style.

### Option C: Svelte store for sidebar state

- Shared store so any component can toggle sidebar

**Pros:** Decoupled.
**Cons:** Overkill — only layout and header need it. Store adds indirection for two-component communication.

### Decision: Option A

Direct state in layout, passed to Header via prop. Simplest, follows established patterns. Sidebar gets a `mobile` variant that renders as fixed overlay with backdrop.

## 2. Zone Editor Responsive Layout

### Option A: Flex direction switch + pointer events

- Desktop: `flex-row` — canvas left, measurements panel right (current layout)
- Mobile (<768px): `flex-col` — canvas fills width, measurements panel below
- Replace mouse events with pointer events on canvas (`onpointerdown`, `onpointermove`, `onpointerup`)
- Add `touch-action: none` on canvas to prevent browser gestures

**Pros:** Clean CSS-only layout switch. Pointer events are the standard — work for mouse, touch, and pen. Single event handler set.
**Cons:** None significant.

### Option B: Hide measurements panel on mobile, show in drawer

**Pros:** More canvas space on mobile.
**Cons:** Overengineered — a simple stack is fine for tablet, which is the primary mobile target.

### Decision: Option A

Flex direction switch is the simplest approach. Pointer events replace mouse events 1:1 — `PointerEvent` extends `MouseEvent`, so `clientX`/`clientY` work identically. Hit radii should increase slightly for touch (VERTEX_HIT_RADIUS 10→18, CLOSE_HIT_RADIUS 12→20) but only when `pointerType === 'touch'`.

## 3. Viewer Controls Reflow

### Option A: Responsive flex-wrap

- Controls (tier toggle, sunlight slider) already below viewer in vertical stack
- Main issue: slider has fixed `w-48` — change to `w-full max-w-48` so it fills on mobile
- Tier buttons: add `min-h-[44px]` for tap targets
- No layout change needed — controls are already vertically stacked

**Pros:** Minimal change, already nearly correct.
**Cons:** None.

### Option B: Move controls into viewer component

**Pros:** Encapsulated.
**Cons:** Breaks current architecture (controls are in page, viewer is iframe wrapper). Unnecessary coupling.

### Decision: Option A

The viewer page layout is already vertical (space-y-4). Just fix the slider width and tap targets.

## 4. Touch Events in ZoneEditor

### Approach: Pointer Events API

Replace all `onmouse*` handlers with `onpointer*` equivalents:
- `onmousedown` → `onpointerdown` (+ `setPointerCapture` for drag)
- `onmousemove` → `onpointermove`
- `onmouseup` → `onpointerup`
- `onclick` → keep (fires after pointer up for both mouse and touch)
- `ondblclick` → keep (works on touch too, though less common)
- Add `touch-action: none` CSS on canvas to prevent scroll/zoom interference
- `getCanvasPoint` signature: `MouseEvent` → `PointerEvent` (compatible — both have clientX/Y)

Touch-aware hit testing:
- Detect `pointerType === 'touch'` on pointerdown
- Use larger hit radii for touch (18px vertex, 20px close) vs mouse (10px, 12px)
- Store last pointerType in component state

## 5. Tap Target Compliance

- Sidebar nav items: `py-2` (28px) → `py-3` (44px+) with `min-h-[44px]`
- Editor toolbar buttons: `py-1` → `min-h-[44px]`
- Viewer tier buttons: `py-2` → `min-h-[44px]`
- Slider: browser-default range input is fine — touch area extends beyond visible track

## Summary of Decisions

| Area | Approach | Complexity |
|------|----------|-----------|
| Sidebar | State in layout, hamburger in header, fixed overlay on mobile | Medium |
| Editor layout | flex-row → flex-col at md: breakpoint | Low |
| Editor touch | Pointer events replacing mouse events | Medium |
| Viewer controls | Fix slider width, add tap targets | Low |
| Tap targets | min-h-[44px] on all interactive elements | Low |
