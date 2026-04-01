# T-036-03 Design: Scan-to-Quote Demo

## Decision: New Example Binary in pt-scan

### Option A: Extend process_sample.rs

Add quote logic to the existing example. Rejected — process_sample.rs is already 350
lines and focused on scan processing. Mixing in quoting/classification/material logic
would make it unwieldy and harder to maintain independently.

### Option B: New example in pt-scan (chosen)

Create `crates/pt-scan/examples/scan_to_quote.rs` — a standalone example that imports
pt-scan, pt-features, pt-quote, pt-project, pt-materials. Add a `just scan-to-quote`
recipe in the Justfile.

**Why:** Single-file example, easy to read top-to-bottom, demonstrates the full product
loop. Each stage is clearly delineated. The Justfile recipe makes it the one-command
demo for investors.

### Option C: New binary crate (pt-demo)

Create a separate crate. Rejected — overkill for a single example script. Would add
workspace overhead for no reusable library code.

## Pipeline Architecture

```
PLY → process_scan_timed() → PointCloud
    → cluster_obstacles() → ClusterResult
    → extract_candidates() → Vec<FeatureCandidate>
    → MockFeatureClassifier::classify() → Vec<ClassifiedFeature>
    → measure_gaps() → Vec<Gap>
    → gap_to_zone() [new helper] → Zone
    → build_tiers() [new helper] → (Tier, Tier, Tier)
    → compute_quote() × 3 → (Quote, Quote, Quote)
    → print_summary() [formatting]
    → generate_terrain() → write GLB
```

## Gap-to-Zone Bridge

Create a `Polygon<f64>` rectangle from gap measurements:
- Center at gap midpoint (converted to feet)
- Width = `gap.clear_width_ft`, Length = `gap.clear_length_ft`
- Oriented along the line between the two feature centroids

This is a ~15-line function. It lives in the example file, not in a library crate,
because it's demo-specific geometry that wouldn't generalize well.

## Material Catalog (Hardcoded for Demo)

Three tiers with realistic SF Bay Area landscaping materials:

**Good — Low-Maintenance Succulents**
- Plants: Unit::Each at $12/ea, quantity derived from area ÷ spacing²
- Planting soil: Unit::CuYd at $45/cuyd, 6" depth

**Better — Ornamental Grasses**
- Plants: Unit::Each at $15/ea
- Planting soil: Unit::CuYd at $45/cuyd, 6" depth

**Best — Seasonal Color Display**
- Plants: Unit::Each at $8/ea (annuals, higher quantity)
- Planting soil: Unit::CuYd at $45/cuyd, 6" depth
- Steel edging: Unit::LinearFt at $3.25/ft

## Plant Quantity Computation

The ticket shows specific plant counts (27 echeveria, 20 carex, 55 annuals). These come
from area ÷ (spacing_ft²). For each tier:
- Compute plantable area from gap
- Divide by plant spacing to get count
- Create an `Each` material with that quantity via an override or direct assignment

Since pt-quote's `Each` unit always produces quantity=1, we need one MaterialAssignment
per plant in the count, OR use a price-override approach where unit_price = price × count.
Better approach: use SqFt material with price_per_sqft = (price_per_plant / spacing²).
This way pt-quote computes: area × (price/spacing²) = correct total.

Actually, simplest: compute plant count outside pt-quote (area / spacing²), create the
line items manually for plants, and use pt-quote for soil/edging. But the ticket says
numbers must come from pt-quote. So: use SqFt pricing where unit_price = price_per_plant /
(spacing_ft × spacing_ft). Then area × unit_price = total plant cost. This is clean.

## Output Format

Match the ticket's example output exactly:
```
POWELL & MARKET PLANTER — SCAN ANALYSIS
Site: 2 tree trunks, X.X ft gap, XX sqft plantable area

GOOD — Low-Maintenance Succulents
  27 × Echeveria 'Lola' (4" spacing)     $324.00
  2.5 cu yd planting soil                 $112.50
  Total: $436.50

BETTER — ...
BEST — ...
```

The plant line uses SqFt in pt-quote but displays as "N × Plant Name" in the output.
We format it: quantity_sqft / spacing² = plant_count, then display as count × name.

## Mock vs Live Flag

Default: mock (deterministic, fast). `--live` flag: use ClaudeCliClassifier for real
LLM classification. The quote computation is always deterministic regardless.

## Terrain GLB Output

Always write terrain GLB alongside the quote, per acceptance criteria ("Viewer-ready:
terrain GLB written for Bevy loading"). Use the same generate_terrain() as process_sample.

## Error Handling

The example uses `.expect()` for fatal errors — this is a demo script, not a library.
Each stage prints timing and progress to stdout.

## Testing Strategy

No separate unit tests for the example binary. The example is verified by:
1. `just scan-to-quote` runs without error
2. Output matches expected format
3. Dollar amounts are deterministic (same scan → same quote)
4. All underlying crate tests continue passing via `just test`

The scenario harness (S.3.1, S.3.2) already validates the compute_quote path.
