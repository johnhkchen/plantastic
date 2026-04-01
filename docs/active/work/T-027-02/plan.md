# T-027-02 Plan: Responsive Editor & Viewer

## Step 1: Sidebar — hamburger + responsive collapse

**Files:** Sidebar.svelte, Header.svelte, (app)/+layout.svelte

1. Modify Sidebar.svelte: accept `open` and `onClose` props, add responsive classes (fixed overlay on mobile, static on desktop), increase nav tap targets to min-h-[44px]
2. Modify Header.svelte: accept `onToggleSidebar` prop, add hamburger button (md:hidden)
3. Modify +layout.svelte: add sidebarOpen state, backdrop overlay, wire props

**Verify:** Visually — sidebar hidden on <768px, hamburger appears, opens as overlay, closes on nav click/backdrop/Escape. Desktop: unchanged.

## Step 2: Zone editor — responsive layout

**Files:** editor/+page.svelte

1. Change content flex container to `flex-col md:flex-row`
2. Adjust measurements panel: remove fixed `w-64`, add `md:w-64` + responsive border/height classes
3. Adjust empty state panel similarly

**Verify:** On narrow viewport, measurements panel appears below canvas. On desktop, panel stays on the right. Canvas fills available space in both orientations.

## Step 3: Zone editor — pointer events

**Files:** ZoneEditor.svelte

1. Replace `onmousemove/onmousedown/onmouseup` with `onpointermove/onpointerdown/onpointerup`
2. Add `touch-action: none` style to canvas
3. Change `getCanvasPoint` to accept `PointerEvent`
4. Add `lastPointerType` state, set from `e.pointerType` in pointerdown
5. Use dynamic hit radii: touch=18/20, mouse=10/12
6. In `handlePointerDown`: call `canvasEl.setPointerCapture(e.pointerId)` for drag
7. Add `min-h-[44px]` to toolbar buttons

**Verify:** Drawing, vertex dragging, zone selection all work with mouse (regression check). Touch behavior correct — larger hit targets when touch detected.

## Step 4: Viewer — responsive controls + tap targets

**Files:** viewer/+page.svelte

1. Sunlight slider: `w-48` → `w-full max-w-48`
2. Tier toggle buttons: add `min-h-[44px]`
3. Sunlight container: add `flex-wrap`

**Verify:** On narrow viewport, slider fills width. Buttons are 44px tall. Controls wrap gracefully.

## Step 5: Quality gate

1. Run `just check` (fmt, lint, test, scenarios)
2. Fix any issues
3. Run scenario dashboard, note before/after

## Testing Strategy

This ticket is primarily CSS/UI work. No new Rust tests needed. Verification is visual + quality gate:
- `just fmt-check` — code formatted
- `just lint` — no warnings
- `just test` — existing tests pass (no regressions)
- `just scenarios` — dashboard value unchanged (this ticket doesn't add new scenarios — it's UX polish)

Pointer event change is the highest-risk area — need to verify mouse interaction still works by confirming existing test patterns. The `onclick`/`ondblclick` handlers are kept, which means the click→draw→close flow is unchanged.
