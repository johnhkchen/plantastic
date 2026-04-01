---
id: T-037-03
story: S-037
title: scan-glb-viewer-test
type: task
status: open
priority: high
phase: done
depends_on: [T-037-02, T-032-03]
---

## Context

After T-032-03 produces a terrain GLB from the Powell & Market scan, this test verifies that the real scan output loads in the Bevy viewer. This is the end-to-end proof: PLY → pt-scan → GLB → browser.

## Acceptance Criteria

- `web/e2e/scan-viewer.spec.ts`:
  1. Reference the terrain GLB produced by T-032-03 (committed as a small test fixture or served from disk)
  2. Navigate to viewer page
  3. Intercept scene API call, return the scan-produced GLB path
  4. Wait for `ready` postMessage
  5. Assert scene loaded (no error)
  6. Take screenshot — should show the Powell & Market ground plane (brick path terrain)
  7. Optional: test orbit interaction (simulate mouse drag, verify camera moved)
- If the full 20M-point terrain GLB is too large for CI, produce a downsampled version (< 5MB)
- Test can run independently of the Rust backend (GLB served as static fixture)

## Implementation Notes

- The scan GLB is vertex-colored (brick texture from RGB points) — it should look like a real place, not a gray blob
- Consider adding a `web/static/viewer/assets/models/powell-market.glb` fixture (downsampled)
- This screenshot is demo material: "we scanned Powell & Market and rendered it in the browser"
- The orbit test proves interactivity: the camera actually orbits the terrain, not a frozen image
- Keep the fixture GLB small (target < 2MB) — decimate aggressively for the test, keep the full version for demos
