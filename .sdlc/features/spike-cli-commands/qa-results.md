# QA Results: Spike CLI — list, show, promote subcommands

## Test Execution

- Binary: `target/debug/ponder` (built from this branch, 2026-03-04)
- All tests run in isolated `mktemp` directories

## Build Checks

| Check | Result |
|-------|--------|
| `SDLC_NO_NPM=1 cargo build --all` | PASS |
| `cargo clippy --all -- -D warnings` | PASS (0 warnings) |
| `SDLC_NO_NPM=1 cargo test --all` | PASS (875 tests, 0 failures) |

## CLI Smoke Tests

| Test | Command | Expected | Result |
|------|---------|----------|--------|
| 1 | `spike list` (no spikes) | "No spikes." | PASS |
| 2 | `spike list --json` (no spikes) | `[]` | PASS |
| 3 | `spike list` (3 spikes) | Table with SLUG, VERDICT, DATE, TITLE | PASS |
| 4 | `spike list --json` | JSON array with all 6 fields | PASS |
| 5 | `spike show adopt-spike` | Findings + ADOPT hint | PASS |
| 6 | `spike show adapt-spike` | Findings, no hint (not promoted yet) | PASS |
| 7 | `spike show reject-spike` | Findings + REJECT hint with knowledge_slug | PASS |
| 8 | `spike show nonexistent` | Error exit 1 | PASS |
| 9 | `spike show adopt-spike --json` | JSON with findings_content field | PASS |
| 10 | `spike promote adapt-spike` | Ponder created, next-step hint printed | PASS |
| 11 | `spike show adapt-spike` (post-promote) | "Ponder: already promoted → 'adapt-spike'" | PASS |
| 12 | `spike promote adopt-spike --as my-custom-ponder` | Ponder 'my-custom-ponder' created | PASS |
| 13 | `spike promote adapt-spike --json` (already promoted) | Error exit 1 (expected) | PASS |

### Test 13 Note

Promoting a spike whose ponder already exists returns an error from core
(`PonderExists`). This is correct behavior — promoting twice is ambiguous
(different `--as` slug would create a second ponder). The error message is
clear: "ponder entry already exists: adapt-spike". No action required.

## Edge Cases

| Scenario | Result |
|----------|--------|
| REJECT spike auto-files to knowledge on `list` | PASS — knowledge_slug populated in JSON |
| date-descending sort in `list` | PASS — adopt (2026-03-04) first, reject (2026-01-01) last |
| `--as` slug override on promote | PASS — ponder 'my-custom-ponder' created |
| JSON `findings_content` field in `show` | PASS — full raw markdown included |

## Verdict

APPROVED — all acceptance criteria met. Feature is ready for merge.
