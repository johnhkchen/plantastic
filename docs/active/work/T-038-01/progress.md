# T-038-01 Progress: Fix PDF Assertion

## Completed

- [x] Replace broken `String::from_utf8_lossy` + `contains("1530")` assertion
      with minimum PDF size check (>10 KB) in `quoting.rs:907-917`
- [x] Updated comment block explaining rationale (Typst glyph encoding,
      quote math verified by S.3.1/S.3.2)
- [x] `cargo fmt --check` — passes
- [x] `cargo clippy --workspace --all-targets` — passes
- [x] `cargo run -p pt-scenarios` — no regressions, S.3.3 shows BLOCKED
      (expected without DATABASE_URL)

## Deviations from plan

None. Single-block replacement as planned.

## Notes

- `just test` fails on pre-existing timeout in `pt-scan` Powell Market
  integration tests (SIGKILL after 60s) — unrelated to this ticket.
  Those tests were already failing before this change.
