# Audit: Document sdlc update as Update Mechanism

## Security Surface

This feature makes two changes:

1. **README.md** — adds 8 lines of documentation. No executable code, no configuration changes, no data model changes. Security surface: zero.

2. **`crates/sdlc-cli/src/cmd/init/mod.rs`** — changes one `println!` string from `"Next: sdlc feature create <slug> --title \"...\""` to `"Next: sdlc ui    # then visit /setup to define Vision and Architecture"`. This is a purely cosmetic change to terminal output. No logic, no data flow, no file writes, no network calls are affected.

## Findings

None. This change has no security surface. Both modifications are documentation and a single printed string.

## Verdict

No security concerns. Waiver-eligible but full audit written for completeness.
