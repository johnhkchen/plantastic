---
id: E-010
title: Polish Dimension & Dashboard Evolution
status: open
priority: high
sprint: 2
---

## Context

Sprint 1 delivered integration depth (computation → API → UI) as the sole quality dimension. But a scenario at ★★★ integration with no loading states, no error handling, and raw unstyled HTML delivers less real value than the star rating implies. A landscaper encountering a blank screen during a slow API call doesn't think "the computation works."

This epic adds **polish** as a second dimension to the value delivery dashboard. Integration measures *reachability* (can a user reach this capability?). Polish measures *usability* (does it feel like a real product when they get there?).

## Formula

```
effective_minutes = raw_minutes × (integration + polish) / 10
```

Both dimensions contribute equally. A 25-minute capability at ★★★ integration + ★☆☆☆☆ polish = 25 × 4/10 = 10.0 effective minutes. The same capability at ★★★ + ★★★ = 25 × 6/10 = 15.0.

### Impact on current numbers

All systems start at 1★ polish. The dashboard total will drop from 58.0 to ~44.5 effective minutes. This is intentional — it makes the integration debt visible. Polish work directly recovers those minutes without any new computation.

## Polish Levels

| Rating | Meaning | Example |
|--------|---------|---------|
| ★☆☆☆☆ | Bare minimum. No loading states, no error messages, default browser styling, no empty states. "It works if you know exactly what to do." | Raw HTML form, no feedback on submit, page blank while loading |
| ★★☆☆☆ | Basic UX. Loading indicators, error messages shown, empty states with prompts. "A user won't get stuck, but it's ugly." | Spinner on load, "No materials yet — add one" empty state, form validation |
| ★★★☆☆ | Designed. Consistent styling, responsive layout, sensible defaults, keyboard navigation. "Looks like a real app." | Design tokens, mobile-friendly, tab order, consistent spacing |
| ★★★★☆ | Demo-ready. Animations, tenant branding hooks, mobile-optimized, accessibility basics (WCAG AA). "Would impress in a sales demo." | Smooth transitions, tenant logo/colors, touch targets, aria labels |
| ★★★★★ | Production-grade. Performance-optimized, error recovery, offline handling, tested on real devices with real data. "A landscaper uses it daily." | Optimistic UI, retry on failure, works on iPad in the field |

## Stories

- S-022: Polish Rating Framework — add the dimension to the harness
- S-023: Baseline Polish Audit — set honest initial ratings

## Success Criteria

- Dashboard displays both integration and polish per scenario
- Formula uses `(integration + polish) / 10` weighting
- All 8 passing scenarios have honest polish ratings
- `just check` passes with updated harness
