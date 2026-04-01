# T-022-01 Design: Polish Enum + Dashboard

## Decision

**Single approach — direct enum extension.** This ticket is mechanical: add a
`Polish` enum mirroring `Integration`, extend the `Pass` tuple, update the
formula and display. No architectural options to evaluate — the ticket is
prescriptive about the shape of the change.

## Polish Enum Design

Mirror `Integration` exactly:

```rust
#[derive(Debug, Clone, Copy)]
pub enum Polish {
    OneStar,   // Bare computation, no UX consideration
    TwoStar,   // Basic UI exists, functional but rough
    ThreeStar,  // Decent UX, handles common paths
    FourStar,   // Polished UX, error handling, edge cases
    FiveStar,   // Production-quality UX, delightful
}
```

Same methods: `stars()`, `weight()`, `label()`. The `weight()` method is
kept for symmetry but the actual formula in `effective_minutes()` uses
`stars()` directly.

## Formula

Current: `raw × (integration.stars() / 5)`
New:     `raw × (integration.stars() + polish.stars()) / 10`

Max possible: `(5 + 5) / 10 = 1.0×` — same ceiling as before.
Min possible: `(1 + 1) / 10 = 0.2×` — same floor as current OneStar.

The two dimensions are additive and equally weighted. This means:
- Integration alone can get you to 0.5× max
- Polish alone can get you to 0.5× max
- Both at max = 1.0× (production-ready)

## Display Format

### status_label

Current: `PASS ★★★☆☆`
New:     `PASS ★★★☆☆ / ★☆☆☆☆`

The `/` separator is clean and readable at a glance. The ticket suggests
`int` / `pol` suffixes but the dual star display is self-explanatory when
the dashboard header explains the format.

### Dashboard header

Add a formula explanation line:
```
  Formula: effective = raw × (int★ + pol★) / 10
```

And a "polish debt" summary showing the gap:
```
  Polish debt:     X.X min recoverable by polish alone
```

Polish debt = sum over passing scenarios of:
`raw × (int.stars() + 5) / 10 - raw × (int.stars() + pol.stars()) / 10`
= `raw × (5 - pol.stars()) / 10`

This shows how much effective time could be gained purely through UX polish
without any new integration work.

### Dashboard legend

Add a one-line legend below the header explaining the dual rating:
```
  Ratings: integration★ / polish★ (each 1-5, weighted equally)
```

## All scenarios start at Polish::OneStar

Per the ticket: all 10 passing scenario `test_fn` returns get
`Polish::OneStar`. This is the honest starting point — the computation
works but UX polish hasn't been evaluated. Future tickets will bump polish
levels as UX work lands.

## Rejected alternatives

None — the ticket is prescriptive. The only design latitude is in display
format, where I chose the clean `★★★☆☆ / ★☆☆☆☆` over verbose labels like
`int:★★★☆☆ pol:★☆☆☆☆` because the header explains the format once and
keeping each row shorter improves readability.
