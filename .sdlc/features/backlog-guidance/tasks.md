# Task Breakdown: Guidance and Agent Command Template Updates for Backlog

## Pre-existing Tasks (from feature creation)

These tasks already exist in the manifest. The task breakdown maps them to the implementation order.

| ID | Title | Notes |
|---|---|---|
| T1 | Add backlog commands to §6 CLI table in `.sdlc/guidance.md` | Requires CLI to exist first |
| T2 | Add §12 Session Close Protocol section to `.sdlc/guidance.md` | Independent of CLI |
| T3 | Update `GUIDANCE_MD_CONTENT` §6 table in `templates.rs` | Must match T1 content |
| T4 | Add 'Discovered out-of-scope concerns' instruction to `sdlc-run` template | Independent |
| T5 | Add 'Discovered out-of-scope concerns' instruction to `sdlc-next` template | Independent |
| T6 | Update capture timing in templates: IMMEDIATE capture language | T4+T5 can include this |
| T7 | Add title quality protocol to guidance and templates | Part of T2/T3, or T4/T5 |
| T8 | Add vocabulary section to guidance §12 | Part of T2/T3 |

## Implementation Order

### Phase 1: Rust CLI (prerequisite for §6 table accuracy)

**T0 (implicit)** — Implement `sdlc backlog` CLI subcommand:
- Create `crates/sdlc-cli/src/cmd/backlog.rs` with `add`, `list`, `show`, `park`
- Register in `cmd/mod.rs` and `main.rs`
- Run `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings`

This task is not in the pre-existing list but is required by T1/T3.

### Phase 2: Content updates (can run concurrently after Phase 1)

**T3 + T1** (merged): Update `GUIDANCE_MD_CONTENT` in `templates.rs` with §6 additions + §12 section, then overwrite `.sdlc/guidance.md`.

**T2 + T7 + T8** (merged into T2): §12 section includes vocabulary (T8), title quality (T7), and capture protocol — write them together.

**T4 + T6** (merged into T4): `sdlc-run` template update includes immediate-capture timing.

**T5 + T6** (merged into T5): `sdlc-next` template update includes immediate-capture timing.

## Execution Sequence

1. Implement `sdlc backlog` CLI (prerequisite)
2. T1 + T3: Update §6 table in templates.rs + guidance.md
3. T2 + T7 + T8: Add §12 section to templates.rs + guidance.md
4. T4 + T6: Update `sdlc-run` template with out-of-scope capture instruction
5. T5 + T6: Update `sdlc-next` template with out-of-scope capture instruction
6. Verify build + clippy pass

## Definition of Done

- `sdlc backlog add|list|show|park` all work
- `sdlc backlog --help` shows all subcommands
- `guidance.md` §6 table has backlog rows
- `guidance.md` §12 exists with vocabulary, title protocol, capture protocol
- `GUIDANCE_MD_CONTENT` in `templates.rs` matches `guidance.md`
- `sdlc-run` template instructs immediate capture with well-formed example
- `sdlc-next` template instructs immediate capture with well-formed example
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` passes
