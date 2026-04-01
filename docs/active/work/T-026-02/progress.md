# T-026-02 Progress: Empty States

## Completed

- [x] Step 1: Created `EmptyState.svelte` — icon/message/submessage props + default slot for actions
- [x] Step 2: Updated Dashboard — replaced inline div with EmptyState + "New Project" CTA button
- [x] Step 3: Updated Catalog — replaced inline div with EmptyState + "Add Material" CTA button
- [x] Step 4: Updated Editor — added EmptyState in zone sidebar when zones.length === 0
- [x] Step 5: Updated QuoteComparison — replaced inline empty div with EmptyState + "Go to Materials" link
- [x] Step 6: Updated Materials — added prerequisite checks (no zones → editor link, no materials → catalog link)
- [x] Step 7: Advanced polish ratings — S.2.1: TwoStar→ThreeStar, S.3.1/S.3.2: TwoStar→ThreeStar
- [x] Step 8: `just check` passes — all 4 gates green

## Deviations from plan

- **Viewer empty state skipped**: The viewer always loads a test scene (test_scene.glb), so there's no "no scan" condition today. Adding a dead code path for a future feature would be speculative. When pt-scan is implemented and real scan detection exists, the empty state gets added with the real condition.

## Metrics

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Effective savings | 76.5 min | 82.5 min | +6.0 min |
| Percentage | 31.9% | 34.4% | +2.5pp |
| Polish debt | 33.0 min | 27.0 min | -6.0 min |
