# Design — T-009-02: Quote Comparison Three-Star

## Problem

The landscaper needs to show a client three price tiers (Good / Better / Best) side by side so the client can compare materials, quantities, and totals at a glance. Currently the quote page is a stub, and the QuoteSummary component only shows one tier at a time.

## Options Evaluated

### Option A: Reuse QuoteSummary × 3

Place three `QuoteSummary.svelte` instances side by side, each with a different tier's quote. Minimal new code.

**Pros**: Reuses existing component, fast to build.
**Cons**: QuoteSummary is designed for a sidebar (narrow, no header emphasis, no tier labels). No visual emphasis on price differences. No way to highlight which tier is cheapest/most expensive. Doesn't show material name differences across tiers for the same zone.

### Option B: New QuoteComparison component with aligned rows

Build a dedicated `QuoteComparison.svelte` component that renders a table-like layout: rows are zones, columns are tiers. Each cell shows the material name, quantity, and line total for that zone in that tier. Grand totals at the bottom.

**Pros**: Purpose-built for comparison. Zones align across tiers so the client sees exactly what differs. Price differences are visually obvious (same row, different amounts). Can highlight cheapest/most expensive per zone.
**Cons**: More code than Option A. Need to handle cases where a zone has an assignment in one tier but not another.

### Option C: Card-based layout (one card per tier)

Three vertical cards, each with tier name, total, and a list of line items. Similar to a pricing page pattern.

**Pros**: Familiar "pricing page" pattern. Clean mobile layout (stack vertically).
**Cons**: Harder to compare individual zone costs across tiers. No row alignment.

## Decision: Option B — Zone-aligned comparison table

**Rationale**: The core value is letting the client see what they're getting in each tier for each zone. A zone-aligned layout makes price differences immediately visible: "Your patio costs $720 in Good, $1,530 in Better, $2,700 in Best" — all on the same row. This is the layout that saves the most presentation time.

The component will have:
1. **Header row**: Good / Better / Best column headers with grand total per tier
2. **Zone rows**: One row per zone, three cells showing material + quantity + line_total
3. **Footer row**: Subtotal per tier, tax if applicable, grand total per tier
4. **Visual emphasis**: Color-coded totals (green for cheapest, neutral for mid, premium badge for best). Bold totals. Price difference indicators.

## Rejected

- **Option A**: Doesn't achieve the "comparison" UX — it's three isolated views
- **Option C**: Doesn't align zones, harder for client to compare specific items

## Data Flow

```
Page loads → fetchQuote(id, 'good'), fetchQuote(id, 'better'), fetchQuote(id, 'best') in parallel
           → 3 Quote objects
           → QuoteComparison component renders aligned grid
```

No new API endpoints needed. Three parallel GET requests.

## Empty States

- **No assignments in any tier**: "Assign materials in the Materials tab to generate quotes"
- **Some tiers empty**: Show the tier column with $0.00 and "No assignments"
- **Zone in one tier but not another**: Empty cell with dash or "—"

## Scenario Upgrade

S.3.1 and S.3.2 currently return `ScenarioOutcome::Pass(Integration::OneStar)`. After this ticket delivers the comparison UI:
- Upgrade to `ScenarioOutcome::Pass(Integration::ThreeStar)`
- The scenario assertions remain identical (computation correctness)
- The integration level reflects: computation works + API exists + UI exists

Effective minutes change:
- S.3.1: 25 × (3/5) = 15 min (was 5)
- S.3.2: 15 × (3/5) = 9 min (was 3)
- Quoting area: ~24 min effective (was ~8)
