# Spec: Guidance and Agent Command Template Updates for Backlog

## Overview

The `sdlc backlog` CLI and data layer already exist (`crates/sdlc-core/src/backlog.rs`), but:
1. No `sdlc backlog` CLI subcommand is exposed — agents have no commands to reference.
2. The `.sdlc/guidance.md` §6 table does not list backlog commands.
3. Agent command templates (`sdlc-run`, `sdlc-next`) do not instruct agents to capture out-of-scope concerns immediately upon discovery.
4. Guidance has no vocabulary section distinguishing **backlog items** from **tasks**.
5. Guidance has no §12 session-close protocol.
6. Title quality requirements (complete sentence + component reference) are not documented.

This feature closes all six gaps by updating `GUIDANCE_MD_CONTENT` in `templates.rs`, and the `sdlc-run` / `sdlc-next` command templates in `commands/sdlc_run.rs` and `commands/sdlc_next.rs`. The backlog CLI subcommand itself (`sdlc backlog`) must also be implemented and registered so the guidance table can reference it accurately.

## Goals

1. **Add `sdlc backlog` CLI subcommand** — wires `sdlc-core::backlog::BacklogStore` into `sdlc-cli`.
2. **Update §6 guidance table** — include all backlog commands.
3. **Add §12 Session Close Protocol** — capture-on-discovery rule, title quality, vocabulary.
4. **Update `sdlc-run` template** — add immediate capture instruction with well-formed example.
5. **Update `sdlc-next` template** — same capture instruction.
6. **Update `GUIDANCE_MD_CONTENT` in `templates.rs`** — keep in sync with live `guidance.md`.

## Non-Goals

- Backlog UI in the web frontend (separate feature).
- REST API for backlog (separate feature).
- `sdlc backlog promote` (may be added later; out of scope here).

## Acceptance Criteria

1. `sdlc backlog --help` prints usage with subcommands: `add`, `list`, `show`, `park`.
2. `sdlc backlog add "AuthMiddleware in auth.rs: token race" --kind concern` creates B1.
3. `sdlc backlog list` shows all items; `--status open` filters.
4. `sdlc backlog show B1` shows full item detail.
5. `sdlc backlog park B1 "not urgent this sprint"` sets status to parked.
6. `GUIDANCE_MD_CONTENT` §6 table includes backlog rows.
7. `GUIDANCE_MD_CONTENT` §12 section exists with vocabulary, title protocol, session-close rule.
8. `sdlc-run` command template includes: _"When you discover an out-of-scope concern, capture it IMMEDIATELY with `sdlc backlog add` — before the next task. Do not defer to session end. If the run crashes, deferred items are lost."_
9. `sdlc-next` template includes the same instruction.
10. `cargo test --all` passes with `SDLC_NO_NPM=1`.
11. `cargo clippy --all -- -D warnings` passes.

## §12 Session Close Protocol — Content Specification

### Vocabulary

| Term | Meaning |
|---|---|
| **Backlog item** | Cross-feature, dormant concern captured by agents at moment of discovery. Lives in `.sdlc/backlog.yaml`. IDs: B1, B2… |
| **Task** | In-feature, active work item. Lives in the feature manifest. IDs: T1, T2… Written by agents and humans. |

Backlog items surface concerns that have no natural home in the current feature. Tasks track committed work inside a feature.

### Immediate Capture Protocol (CRITICAL)

Capture must happen at the **moment of discovery** — not deferred to session end.

> If you defer capture and the run crashes, the concern is permanently lost.

Session-close review is **additive** — confirm you captured everything you noticed, then stop.

```bash
# Capture at moment of discovery (not session end):
sdlc backlog add \
  "AuthMiddleware in auth.rs: token validation has a race condition under concurrent requests." \
  --kind concern \
  --source-feature my-feature \
  --evidence "crates/sdlc-server/src/auth.rs:42"
```

### Title Quality Protocol

A backlog item title must be:
- A **complete sentence** — not a fragment.
- Include a **component reference** — file name, module name, or type name.
- Describe **what the problem is**, not just that a problem exists.

**Good:**
```
AuthMiddleware in auth.rs: token validation has a race condition under concurrent requests.
RedbDatabase in orchestrator.rs: compaction not configured, may grow unbounded under heavy write load.
MilestoneUatRunner in runs.rs: server lifecycle not guarded — UAT kills the user's dev session.
```

**Bad:**
```
race condition in auth
token bug
database issue
```

## §6 Table Additions

Add to the CLI table in `guidance.md` §6 and `GUIDANCE_MD_CONTENT`:

| Action | Command |
|---|---|
| Add backlog item | `sdlc backlog add "<title>" --kind <concern\|idea\|debt>` |
| List backlog | `sdlc backlog list [--status <open\|parked\|promoted>]` |
| Show backlog item | `sdlc backlog show <id>` |
| Park backlog item | `sdlc backlog park <id> "<reason>"` |

## CLI Design: `sdlc backlog`

### `sdlc backlog add <title> --kind <kind> [--source-feature <slug>] [--evidence <ref>] [--description <text>]`

Creates a new backlog item. `<title>` is positional (quoted string). `--kind` is required: `concern`, `idea`, or `debt`.

### `sdlc backlog list [--status <status>] [--source-feature <slug>]`

Lists all backlog items. `--status` filters by lifecycle state. Human-readable table output; `--json` for JSON.

### `sdlc backlog show <id>`

Shows full item detail (title, kind, status, description, evidence, source feature, park reason, promoted_to, timestamps).

### `sdlc backlog park <id> <reason>`

Parks an item with a required reason. Both ID and reason are positional.

## Implementation Notes

- `crates/sdlc-cli/src/cmd/backlog.rs` — new module, wraps `sdlc_core::backlog::BacklogStore`.
- Register in `crates/sdlc-cli/src/cmd/mod.rs` and `crates/sdlc-cli/src/main.rs`.
- `GUIDANCE_MD_CONTENT` in `crates/sdlc-cli/src/cmd/init/templates.rs` is the source of truth for `guidance.md` — edit the constant, not the file on disk (which is re-generated by `sdlc init`/`sdlc update`).
- Live `.sdlc/guidance.md` must also be updated in this feature (it is managed content — overwrite is safe).
- `sdlc-run` template: `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs`
- `sdlc-next` template: `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`

## Tasks (pre-existing, mapped to spec)

| Task | Spec Coverage |
|---|---|
| T1 — Add backlog commands to §6 in `guidance.md` | §6 Table Additions |
| T2 — Add §12 Session Close Protocol to `guidance.md` | §12 Content Specification |
| T3 — Update `GUIDANCE_MD_CONTENT` §6 in `templates.rs` | §6 + §12 in templates |
| T4 — Add 'Discovered out-of-scope' instruction to `sdlc-run` | `sdlc-run` template update |
| T5 — Add 'Discovered out-of-scope' instruction to `sdlc-next` | `sdlc-next` template update |
| T6 — Update capture timing in templates: IMMEDIATE capture | Immediate Capture Protocol |
| T7 — Add title quality protocol to guidance and templates | Title Quality Protocol |
| T8 — Add vocabulary section to guidance §12 | Vocabulary |

Note: The `sdlc backlog` CLI subcommand is required for T1/T3 (the §6 table must reference real commands). It must be implemented before those tasks are complete.
