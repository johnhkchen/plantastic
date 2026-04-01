# T-027-02 Structure: Responsive Editor & Viewer

## Files Modified

### 1. `web/src/routes/(app)/+layout.svelte`
**Purpose:** Add sidebar toggle state, restructure for mobile overlay

Current structure:
```
div.flex.h-screen > Sidebar + div.flex-1 > Header + main
```

New structure:
```
div.flex.h-screen
  ├─ [mobile backdrop: fixed overlay, z-40, hidden on md:]
  ├─ Sidebar (receives `open` prop; mobile=fixed overlay z-50, desktop=static)
  ├─ div.flex-1 > Header (receives `onToggleSidebar` prop) + main
```

Changes:
- Add `let sidebarOpen = $state(false)` 
- Pass `sidebarOpen` to Sidebar, `onToggleSidebar` callback to Header
- Add backdrop div: `{#if sidebarOpen}<div class="fixed inset-0 z-40 bg-black/50 md:hidden" ...>`
- Close sidebar on nav click (callback prop to Sidebar)

### 2. `web/src/lib/components/Sidebar.svelte`
**Purpose:** Responsive sidebar — static on desktop, overlay on mobile

Changes:
- Accept props: `open: boolean`, `onClose: () => void`
- Aside classes: 
  - Base: `flex h-full w-56 flex-col border-r border-gray-200 bg-white`
  - Mobile: `fixed inset-y-0 left-0 z-50 transform transition-transform duration-200`
  - Mobile hidden: `-translate-x-full` (when !open)
  - Mobile visible: `translate-x-0` (when open)
  - Desktop: `md:static md:translate-x-0` (always visible, no transform)
- Nav links: add `min-h-[44px] flex items-center` for tap targets
- Add `onclick` on each nav link to call `onClose()` (closes sidebar on mobile nav)

### 3. `web/src/lib/components/Header.svelte`
**Purpose:** Add hamburger button for mobile sidebar toggle

Changes:
- Accept prop: `onToggleSidebar: () => void`
- Add hamburger button: `<button class="md:hidden ..." onclick={onToggleSidebar}>` with 3-line SVG icon
- Button positioned left of tenant name
- Button: `min-h-[44px] min-w-[44px]` for tap target

### 4. `web/src/routes/(app)/project/[id]/editor/+page.svelte`
**Purpose:** Responsive measurements panel

Changes:
- Content div: `flex min-h-0 flex-1` → `flex min-h-0 flex-1 flex-col md:flex-row`
- Canvas area: stays `min-w-0 flex-1`
- Measurements panel: `w-64 border-l` → `md:w-64 md:border-l md:border-t-0 border-t max-h-48 md:max-h-none overflow-y-auto`
- On mobile: panel renders below canvas with top border, max height, scrollable
- On desktop: panel renders right of canvas with left border (current behavior)

### 5. `web/src/lib/components/zone-editor/ZoneEditor.svelte`
**Purpose:** Pointer events for touch support

Changes:
- Canvas element: replace mouse event handlers with pointer events
  - `onclick` → keep (works for both)
  - `ondblclick` → keep
  - `onmousemove` → `onpointermove`
  - `onmousedown` → `onpointerdown`
  - `onmouseup` → `onpointerup`
- Add `touch-action: none` to canvas style (prevents browser gestures)
- `getCanvasPoint`: parameter type `MouseEvent` → `PointerEvent`
- Add `let lastPointerType = $state<string>('mouse')`
- In `handlePointerDown`: capture `e.pointerType`, call `canvasEl.setPointerCapture(e.pointerId)`
- In hit testing: use dynamic radii based on `lastPointerType`:
  - Touch: VERTEX_HIT_RADIUS=18, CLOSE_HIT_RADIUS=20
  - Mouse/pen: VERTEX_HIT_RADIUS=10, CLOSE_HIT_RADIUS=12
- Toolbar zone type buttons: add `min-h-[44px]` for tap targets

### 6. `web/src/routes/(app)/project/[id]/viewer/+page.svelte`
**Purpose:** Responsive controls, tap targets

Changes:
- Sunlight slider: `w-48` → `w-full max-w-48` (fills on mobile, caps on desktop)
- Tier toggle buttons: add `min-h-[44px]` for tap targets
- Time display: `w-20` stays (already small enough)
- Sunlight container: `flex items-center gap-3` → `flex flex-wrap items-center gap-3`

## Files NOT Modified

- `web/src/lib/components/viewer/Viewer.svelte` — iframe container is already `w-full aspect-video`, responsive
- `web/src/lib/components/zone-editor/hit-test.ts` — hit radii passed as parameters from ZoneEditor
- `web/src/lib/components/zone-editor/renderer.ts` — pure drawing, no input handling
- `web/src/lib/components/zone-editor/types.ts` — no type changes needed (PointerEvent extends MouseEvent)
- `web/src/app.css` — no global CSS changes needed
- `web/src/routes/(app)/project/[id]/+layout.svelte` — project layout unchanged

## Component Boundary Summary

```
+layout.svelte (app)
  ├─ sidebarOpen state
  ├─ backdrop (mobile only)
  ├─ Sidebar ← open, onClose props
  └─ Header ← onToggleSidebar prop
       └─ hamburger button (md:hidden)

editor/+page.svelte
  └─ flex-col md:flex-row layout switch
       ├─ ZoneEditor (pointer events, touch-action:none)
       └─ measurements panel (responsive positioning)

viewer/+page.svelte
  └─ Viewer (unchanged)
  └─ controls (responsive slider width, tap targets)
```
