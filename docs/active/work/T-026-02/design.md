# T-026-02 Design: Empty States

## Decision: Reusable `EmptyState.svelte` with props + slots

### Option A: Reusable component with icon/message/action props and slots
Create a single `EmptyState.svelte` component that accepts:
- `icon`: string (emoji or SVG markup) — rendered above the message
- `message`: string — the primary guidance text
- `submessage`: optional string — secondary text (e.g., "You'll see zones listed here once you draw them")
- Default slot for action content (buttons, links)

Each page imports `EmptyState` and passes page-specific content.

**Pros**: Consistent styling, single place to adjust layout/spacing, follows ErrorBanner/LoadingSkeleton pattern.
**Cons**: Slightly more abstraction than raw HTML.

### Option B: Inline styled divs per page (status quo pattern)
Keep the existing pattern: each page has its own `<div class="rounded-lg...">` with text.

**Pros**: No new component.
**Cons**: Inconsistent styling, duplicated markup, no icons, no CTAs. Violates the ticket requirement for a reusable component.

### Option C: Generic slot-only wrapper
A wrapper that only provides the container styling, with all content as slots.

**Pros**: Maximum flexibility.
**Cons**: Doesn't enforce consistent layout (icon → message → action). Each page would re-implement the internal structure.

### Decision: Option A

The ticket explicitly requires "EmptyState.svelte with icon, message, action slot." Option A matches exactly. The component provides structure (icon above message above action) while the slot allows each page to customize the CTA (button, link, or nothing).

## Component API

```svelte
<EmptyState icon="📋" message="No projects yet" submessage="Create your first project to get started">
  <button onclick={...}>New Project</button>
</EmptyState>
```

Props:
- `icon: string` — emoji or short text rendered large above the message
- `message: string` — primary text, medium weight
- `submessage?: string` — secondary text, lighter
- Default slot: action area (buttons, links). Optional — renders nothing if not provided.

Styling: Centered flex column, rounded border, generous padding (matches existing `p-12 text-center` pattern). Icon at 2-3rem. Message in `text-gray-600 font-medium`. Submessage in `text-gray-400 text-sm`. Action area with `mt-4`.

## Per-Page Empty States

### Dashboard (no projects)
- Icon: `📋` (clipboard)
- Message: "No projects yet"
- Submessage: "Create your first project to get started"
- Action: "New Project" button (triggers existing `showCreateModal = true`)

### Catalog (no materials)
- Icon: `🧱` (brick)
- Message: "No materials in your catalog"
- Submessage: "Add materials before assigning them to zones"
- Action: "Add Material" button (triggers existing `openCreateModal()`)
- Note: The "no filter results" empty state stays separate (it's not an onboarding moment)

### Editor (no zones)
- Icon: `✏️` (pencil)
- Message: "No zones yet"
- Submessage: "Click on the plan view to draw your first zone"
- Action: None (the canvas IS the action — the hint is sufficient)
- Placement: Overlay or panel where the zone list would be. The ZoneEditor canvas should still render so the user can start drawing.

### Quote (no assignments)
- Already handled in QuoteComparison.svelte (lines 92-99), but replace with EmptyState
- Icon: `💰` (money bag)
- Message: "No quotes to compare"
- Submessage: "Assign materials to zones to generate quotes"
- Action: Link to materials tab

### Materials (no zones or no materials)
- Two possible empty conditions:
  1. No zones → "Draw zones in the editor first" with link to editor
  2. No materials → "Add materials to your catalog first" with link to catalog
- Priority: check zones first (can't assign without zones), then materials
- Icon: `🔗` (link, representing assignments)
- Replaces three-column layout entirely when the prerequisite data is missing

### Viewer (no scan)
- The viewer currently always has a test scene. The "no scan" empty state will only trigger when we have real scan detection. For now, add the empty state but condition it on a `hasScan` signal that we can wire up later.
- Icon: `📱` (mobile phone — LiDAR scan)
- Message: "No 3D scan yet"
- Submessage: "Upload a LiDAR scan to see a 3D preview"
- For now: skip this one. The viewer always loads a test scene. Adding a dead code path for a future feature contradicts the "no speculative abstractions" rule. When pt-scan is implemented, the empty state gets added with the real condition.

## Scenario Polish Advancement

Empty states complete the "onboarding UX" dimension. With loading (T-026-01) + error (T-026-01) + empty (this ticket), the frontend covers all three data states. This justifies advancing affected scenarios from Polish::TwoStar to Polish::ThreeStar:
- S.2.1 (zone drawing) → editor gets empty state
- S.3.1 (quantity) → quote gets empty state via QuoteComparison
- S.3.2 (three-tier) → quote gets empty state via QuoteComparison

S.2.4 (3D preview) stays at TwoStar — no empty state added for viewer (test scene always present).
S.2.2 (material catalog) stays at FiveStar polish (pure computation scenario).

## What not to do
- Don't add animations or illustrations — emoji icons are sufficient for now
- Don't change the "no filter results" state in catalog — that's a different UX concern
- Don't add empty states for sub-components (ZoneList, MaterialPicker, QuoteSummary) — the page-level empty state replaces the entire layout when prerequisites are missing
