# QA Plan: Guidance and Agent Command Template Updates for Backlog

## Test Strategy

This feature has two surfaces: Rust CLI and static content. Quality checks are:
1. `SDLC_NO_NPM=1 cargo test --all` — unit tests in `sdlc-core` cover the backlog data layer; CLI wiring is verified by smoke tests
2. `cargo clippy --all -- -D warnings` — lint correctness
3. Manual smoke tests for CLI output shape and error cases
4. Content review: verify guidance.md and templates have all required sections

---

## QA Scenarios

### CLI Correctness

| # | Scenario | Command | Expected |
|---|---|---|---|
| QA-1 | Help shows subcommands | `sdlc backlog --help` | Output includes: `add`, `list`, `show`, `park` |
| QA-2 | Add concern | `sdlc backlog add "AuthMiddleware in auth.rs: token race." --kind concern` | Prints "Added B1 [concern]" |
| QA-3 | Add with source-feature | `sdlc backlog add "X" --kind idea --source-feature my-feat` | B2 created with source_feature=my-feat |
| QA-4 | List all | `sdlc backlog list` | Shows B1 and B2 in table |
| QA-5 | List by status | `sdlc backlog list --status open` | Both items shown (both open) |
| QA-6 | Show item | `sdlc backlog show B1` | Shows title, kind=concern, status=open |
| QA-7 | Park item | `sdlc backlog park B1 "not urgent this sprint"` | Prints "Parked B1: not urgent this sprint" |
| QA-8 | List after park | `sdlc backlog list --status open` | Only B2 (B1 is parked) |
| QA-9 | JSON output | `sdlc backlog list --json` | Valid JSON array |
| QA-10 | Park requires reason | `sdlc backlog park B2 ""` | Error: park_reason must not be empty |
| QA-11 | Unknown ID | `sdlc backlog show B99` | Error: backlog item B99 not found |

### Build Gates

| # | Gate | Command | Expected |
|---|---|---|---|
| QA-12 | Tests pass | `SDLC_NO_NPM=1 cargo test --all` | All tests pass |
| QA-13 | No clippy warnings | `cargo clippy --all -- -D warnings` | Zero warnings |

### Content Checks

| # | Check | Method | Expected |
|---|---|---|---|
| QA-14 | §6 table has backlog rows | `grep "sdlc backlog" .sdlc/guidance.md` | 4 rows present |
| QA-15 | §12 exists | `grep "^## 12" .sdlc/guidance.md` | Section header found |
| QA-16 | §12 has vocabulary | `grep "Backlog item\|backlog item" .sdlc/guidance.md` | Present |
| QA-17 | §12 has title quality | `grep "complete sentence\|component reference" .sdlc/guidance.md` | Present |
| QA-18 | §12 has CRITICAL capture | `grep "moment of discovery\|IMMEDIATE\|CRITICAL" .sdlc/guidance.md` | Present |
| QA-19 | templates.rs §6 matches | `grep "sdlc backlog" crates/sdlc-cli/src/cmd/init/templates.rs` | 4 rows present |
| QA-20 | templates.rs §12 exists | `grep "Session Close Protocol" crates/sdlc-cli/src/cmd/init/templates.rs` | Present |
| QA-21 | sdlc-run has capture instruction | `grep "moment of discovery\|out-of-scope" crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs` | Present |
| QA-22 | sdlc-next has capture instruction | `grep "moment of discovery\|out-of-scope" crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` | Present |

---

## Pass/Fail Criteria

All 22 scenarios must pass. No exceptions.
