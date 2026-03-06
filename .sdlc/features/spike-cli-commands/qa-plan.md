# QA Plan: Spike CLI — list, show, promote subcommands

## Automated checks

1. `SDLC_NO_NPM=1 cargo test --all` — all unit tests pass
2. `cargo clippy --all -- -D warnings` — no warnings
3. `cargo build --all` — compiles cleanly

## CLI smoke tests (manual / CI)

Set up a temp `.sdlc/spikes/` directory with fixture findings.md files for each verdict.

### list
- `sdlc spike list` → table with SLUG | VERDICT | DATE | TITLE columns
- `sdlc spike list` with no spikes dir → "No spikes."
- `sdlc spike list --json` → valid JSON array with correct fields

### show
- `sdlc spike show <adopt-slug>` → shows findings + ADOPT hint line
- `sdlc spike show <reject-slug>` → shows findings + REJECT hint with knowledge_slug
- `sdlc spike show <adapt-slug>` (promoted) → shows "Ponder: already promoted → '<ponder_slug>'"
- `sdlc spike show <nonexistent>` → exits non-zero with error message
- `sdlc spike show <slug> --json` → includes `findings_content` field

### promote
- `sdlc spike promote <adapt-slug>` → creates ponder, prints slug + next-step hint
- `sdlc spike promote <slug> --as custom-name` → uses override slug
- `sdlc spike promote <nonexistent>` → exits non-zero
- `sdlc spike promote <slug> --json` → `{ spike_slug, ponder_slug }`

## Edge cases

- Spike directory exists but findings.md is absent → `show` prints "No findings." gracefully
- Promote called twice on same slug → idempotent (ponder already exists); core handles this
- REJECT spike in list → auto-files to knowledge base (core side effect); no crash
