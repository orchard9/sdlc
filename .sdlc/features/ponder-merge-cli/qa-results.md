# QA Results: sdlc ponder merge — CLI command and core data model

## QA-1: Successful merge end-to-end — PASS
- JSON output: `sessions_copied: 1, artifacts_copied: 1, team_members_copied: 1`
- Source: `status: "parked"`, `merged_into: "qa-tgt"`
- Target: `merged_from: ["qa-src"]`, sessions bumped to 1, tags merged ("ux"), team member copied
- Target dir contains `notes.md` artifact
- Target session contains merge header comment `<!-- merged from: qa-src, original session: 1 -->`

## QA-2: Merged entries hidden from default list — PASS
- Default `list --json`: only shows `qa-tgt`, no `qa-src`
- `list --all --json`: shows both entries, `qa-src` has `merged_into` field

## QA-3: Show redirect banner — PASS
- `show qa-src` outputs: "This entry was merged into 'qa-tgt'. Use `sdlc ponder show qa-tgt` instead."

## QA-4: Reject merge of committed source — PASS
- Error: "cannot merge committed entry 'qa-committed'"

## QA-5: Reject merge into committed target — PASS
- Error: "cannot merge into committed entry 'qa-committed-tgt'"

## QA-6: Reject self-merge — PASS
- Error: "cannot merge an entry into itself"

## QA-7: Artifact collision prefix — PASS
- Target dir contains both `notes.md` (original) and `col-src--notes.md` (from source)

## QA-8: Serde backward compatibility — PASS
- Existing project ponder entries (without merged_into/merged_from) load and list cleanly

## QA-9: Build and test suite — PASS
- `SDLC_NO_NPM=1 cargo build --all` — clean
- `SDLC_NO_NPM=1 cargo test --all` — all pass (452 tests)
- `cargo clippy --all -- -D warnings` — zero warnings

## Summary: 9/9 PASS
