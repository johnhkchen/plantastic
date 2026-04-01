# T-009-01 Design: Material Assignment UI

## Decision: Build on the Materials Tab

### Option A: Integrate into Editor Page
- Pros: Direct access to canvas, `selectedZoneId` already exists, single-page workflow
- Cons: Editor page is already complex (350 lines), canvas takes most of the viewport, cramming a material picker + tier tabs + quote total into the sidebar makes it unusable on typical screens

### Option B: Build on the Materials Tab (Selected)
- Pros: Dedicated full-width layout, clean separation of concerns (draw zones vs assign materials), each tab has one job, room for zone list + material picker + quote summary side by side
- Cons: No canvas — need a zone list with click-to-select instead of click-on-canvas

### Option C: New Dedicated "Assign" Tab
- Pros: Clean slate
- Cons: Adds another nav item, fragments the workflow unnecessarily. The "Materials" tab is already the expected home for this.

**Decision**: Option B. The Materials tab is the natural home. Landscapers think in steps: draw zones first (Editor tab), then assign materials (Materials tab). The zone list with selection is actually better UX than requiring the canvas — it's more accessible on tablets and makes the zone→material mapping explicit.

## Layout Design

Three-column layout within the Materials tab:

```
┌─────────────┬──────────────────────────┬─────────────────┐
│ Zone List   │ Material Picker          │ Quote Summary   │
│             │                          │                 │
│ [zone 1] ◄──│ [tier tabs: G | B | B]   │ Subtotal: $X    │
│  zone 2     │                          │ Total: $X       │
│  zone 3     │ ┌─ Hardscape ──────────┐ │                 │
│             │ │ Flagstone    $8.50   │ │ Line Items:     │
│             │ │ Travertine   $12.00  │ │  zone1: $Y      │
│             │ └──────────────────────┘ │  zone2: $Z      │
│             │ ┌─ Fill ───────────────┐ │                 │
│             │ │ DG           $45.00  │ │                 │
│             │ └──────────────────────┘ │                 │
└─────────────┴──────────────────────────┴─────────────────┘
```

- **Zone List** (left, ~200px): All zones with type badge, area, current material assignment for active tier. Click to select. Selected zone highlighted.
- **Material Picker** (center, flex): Tier tabs at top. Materials grouped by category. Click material to assign to selected zone for active tier. Current assignment shown with checkmark.
- **Quote Summary** (right, ~240px): Live total for active tier. Line items showing zone-material-cost breakdown.

## Zone Selection Without Canvas

The zone list replaces canvas selection. Each zone row shows:
- Zone label (or type as fallback)
- Zone type badge (colored like the canvas)
- Area and perimeter
- Currently assigned material name for active tier (or "No material")

Click a zone to select it. Selected zone gets a left border accent + background highlight. No canvas needed.

## Tier Navigation

Three tabs at the top of the material picker: Good | Better | Best. Active tab is underlined. Switching tabs:
1. Loads assignments for that tier
2. Updates zone list to show per-tier material assignments
3. Fetches quote for that tier

All three tiers' assignments are loaded upfront (`GET /projects/:id/tiers`) to avoid tab-switch latency.

## Assignment Flow

1. User selects a zone (left panel)
2. User clicks a material in the picker (center panel)
3. Frontend immediately updates local state (optimistic)
4. Debounced save: after 800ms of no changes, `PUT /projects/:id/tiers/:tier` with full assignment list
5. After save completes, `GET /projects/:id/quote/:tier` to refresh quote total
6. Quote summary updates

Why debounce: landscapers assigning materials to multiple zones rapidly shouldn't trigger N API calls. 800ms is shorter than zone auto-save (1500ms) since assignment saves are lighter.

## Zone ID Stability

Zone IDs can change when the user edits zones on the Editor tab and the debounced bulk PUT fires. When the user navigates to the Materials tab:
1. Fetch fresh zones (`GET /projects/:id/zones`)
2. Fetch fresh tier assignments (`GET /projects/:id/tiers`)
3. Zone IDs in assignments will match current zone IDs (both from same DB state)

This is safe because we reload everything on tab navigation. The only risk is if the user has two browser tabs open — not a concern for V1.

## Mock API Extensions

Add to `mock.ts`:
- `GET /projects/:id/tiers` — return 3 tiers with empty assignments
- `PUT /projects/:id/tiers/:tier` — store assignments in memory
- `GET /projects/:id/quote/:tier` — compute mock quote from stored assignments + mock materials + mock zone areas

## State Management

Local component state in the Materials page, not the global store. Pattern matches the Editor page. State:
- `zones: ApiZone[]` — loaded on mount
- `materials: Material[]` — loaded on mount
- `assignments: Map<TierLevel, Assignment[]>` — loaded on mount, updated on assign
- `activeTier: 'good' | 'better' | 'best'` — tab selection
- `selectedZoneId: string | null` — zone selection
- `quote: Quote | null` — refreshed after assignment save

## Rejected Alternatives

### Two-panel (no quote summary)
Rejected: the quote total is the payoff for assigning materials. Seeing it update live is the core value.

### Modal material picker
Rejected: extra clicks per assignment. With 20 zones and 3 tiers, modals would be exhausting.

### Drag-and-drop assignment
Rejected: overengineered for V1, harder on tablets, not better than click-to-assign.

## Scenarios
- This ticket does not directly flip any scenario. It enables T-009-02 (three-tier comparison) which pushes S.3.1/S.3.2 to ThreeStar.
- No milestone to claim — this is UI plumbing, not foundational compute.
