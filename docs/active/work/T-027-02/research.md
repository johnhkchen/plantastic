# T-027-02 Research: Responsive Editor & Viewer

## Scope

Three areas need responsive treatment: (1) zone editor canvas + measurements panel, (2) Bevy 3D viewer + controls, (3) app sidebar. All must work on tablet for crew handoff (S.4.1).

## Current State

### 1. Sidebar (`web/src/lib/components/Sidebar.svelte`)

- Fixed `w-56` (224px) aside, always visible
- Flex column: logo header (h-14) + nav links
- Nav items: `px-3 py-2 text-sm` — ~28px tall, below 44px tap target guideline
- No responsive classes, no media queries, no collapse/hamburger state
- Used in app layout (`web/src/routes/(app)/+layout.svelte`) as direct child of `div.flex.h-screen`

### 2. App Layout (`web/src/routes/(app)/+layout.svelte`)

- Structure: `div.flex.h-screen.overflow-hidden > Sidebar + div.flex.flex-1.flex-col > Header + main`
- Sidebar is always rendered, takes 224px
- Header: `h-14`, shows tenant name + avatar
- Main: `flex-1 overflow-y-auto p-6`
- No responsive classes anywhere

### 3. Zone Editor Page (`web/src/routes/(app)/project/[id]/editor/+page.svelte`)

- Layout: `div.flex.h-full.flex-col` with header bar, then `div.flex.min-h-0.flex-1`
- Canvas area: `div.min-w-0.flex-1` containing `<ZoneEditor />`
- Measurements panel: `div.w-64.overflow-y-auto.border-l` — fixed 256px right sidebar
- Panel shows zone list with area/perimeter measurements
- When no zones: shows EmptyState in same w-64 panel
- No responsive classes — panel never moves below canvas

### 4. ZoneEditor Component (`web/src/lib/components/zone-editor/ZoneEditor.svelte`)

- Canvas fills parent via ResizeObserver + DPI scaling (devicePixelRatio)
- **Mouse-only events**: `onclick`, `ondblclick`, `onmousemove`, `onmousedown`, `onmouseup`
- `getCanvasPoint(e: MouseEvent)` extracts coordinates from MouseEvent
- Drag state for vertex handles: mousedown → mousemove → mouseup pattern
- Hit testing: `findZoneAtPoint`, `isNearFirstVertex`, `nearestVertex` (in ./hit-test.ts)
- Hit radii: VERTEX_HIT_RADIUS=10, CLOSE_HIT_RADIUS=12 — may be too small for touch
- Toolbar: zone type buttons, label input, delete button — all small touch targets
- Status bar at bottom

### 5. Viewer Page (`web/src/routes/(app)/project/[id]/viewer/+page.svelte`)

- Layout: `div.space-y-4` — vertical stack
- Contains: h2 title, ErrorBanner, `<Viewer />`, tier toggle, sunlight slider, zone info
- Tier toggle: `inline-flex rounded-lg border` with `px-4 py-2` buttons (~40px tall)
- Sunlight slider: `w-48` fixed width, `h-2` track
- Controls are already below viewer (vertical layout), but slider width is fixed
- No responsive classes

### 6. Viewer Component (`web/src/lib/components/viewer/Viewer.svelte`)

- Container: `div.relative.aspect-video.w-full.overflow-hidden.rounded-lg.bg-gray-900`
- Iframe: `h-full w-full border-0`, src="/viewer/index.html"
- PostMessage API for commands: setTier, setLightAngle, loadScene
- Listens for messages: ready, zoneTapped, error, lightAngleChanged, tierChanged
- Touch orbit/pan/zoom handled inside Bevy iframe (T-014-01) — no work needed here

### 7. Header Component (`web/src/lib/components/Header.svelte`)

- Simple `header.flex.h-14` with tenant name + avatar
- No hamburger button currently — will need one for sidebar toggle on mobile

## Patterns from T-027-01

T-027-01 (responsive quote/catalog, now done) established these patterns:
- Tailwind responsive prefixes: `md:` (768px), `lg:` (1024px)
- CSS-first approach: `hidden md:grid`, `md:hidden` for layout switching
- Minimal JS: single `$state` for mobile tier selection
- Tap targets: `min-h-[44px]` on interactive elements
- No custom CSS — all Tailwind utilities

## CSS Infrastructure

- Tailwind v4 (`tailwindcss@4.2.2`) with default breakpoints (sm:640, md:768, lg:1024, xl:1280)
- Global CSS: `web/src/app.css` — `@import 'tailwindcss'` + `@theme` brand colors
- Svelte 5 runes: `$props()`, `$state()`, `$derived()`, `$effect()`
- No custom responsive utilities or CSS-in-JS

## Key Constraints

1. **Touch for tablet crew handoff (S.4.1)**: ZoneEditor must support pointer events, not just mouse
2. **Sidebar overlay**: AC says "overlay when open, not push-content" on mobile
3. **Canvas aspect ratio**: Currently fills parent (no fixed ratio). AC says "preserving aspect ratio" — but the canvas is a drawing surface, not an image. Filling available width IS the right behavior; aspect ratio is preserved by the ResizeObserver scaling.
4. **Bevy viewer touch**: Already handled by bevy_panorbit_camera in the iframe (T-014-01)
5. **Breakpoint**: AC specifies < 768px for sidebar collapse → matches `md:` breakpoint

## Dependencies

- T-027-01 (done): Established responsive patterns, now followed here
- T-014-01 (Bevy viewer): Touch orbit/pan/zoom — assumed complete in iframe
