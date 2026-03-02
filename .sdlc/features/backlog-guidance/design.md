# Design: Guidance and Agent Command Template Updates for Backlog

## Summary

This feature has two change surfaces:
1. **Rust CLI** — add `sdlc backlog` subcommand wiring `sdlc-core::BacklogStore` into the CLI.
2. **Content updates** — update `GUIDANCE_MD_CONTENT`, `sdlc_run.rs`, and `sdlc_next.rs` templates.

The data layer (`sdlc-core/src/backlog.rs`) already exists and is tested. This design covers only the CLI surface and content.

---

## 1. Rust CLI: `sdlc backlog`

### Module: `crates/sdlc-cli/src/cmd/backlog.rs`

```
pub enum BacklogSubcommand {
    Add {
        title: String,
        kind: BacklogKindArg,
        source_feature: Option<String>,
        evidence: Option<String>,
        description: Option<String>,
    },
    List {
        status: Option<BacklogStatusArg>,
        source_feature: Option<String>,
    },
    Show { id: String },
    Park { id: String, reason: String },
}
```

`BacklogKindArg` and `BacklogStatusArg` are thin `clap`-compatible enums that convert to the `sdlc_core::backlog::{BacklogKind, BacklogStatus}` equivalents.

#### `add` output (human-readable)
```
Added B1 [concern]
  AuthMiddleware in auth.rs: token validation has a race condition under concurrent requests.
```

#### `list` output (table)
```
ID   Kind     Status  Title
B1   concern  open    AuthMiddleware in auth.rs: token validation has a race condition…
B2   idea     parked  RedbDatabase in orchestrator.rs: compaction not configured
```

#### `show` output (key-value block)
```
ID:             B1
Kind:           concern
Status:         open
Title:          AuthMiddleware in auth.rs: token validation has a race condition under concurrent requests.
Description:    —
Evidence:       crates/sdlc-server/src/auth.rs:42
Source feature: my-feature
Created:        2026-03-02 03:15:00 UTC
```

#### `park` output
```
Parked B1: revisit after v14
```

### Registration

**`crates/sdlc-cli/src/cmd/mod.rs`** — add `pub mod backlog;`

**`crates/sdlc-cli/src/main.rs`**:
- Import `BacklogSubcommand`
- Add `Commands::Backlog { subcommand: BacklogSubcommand }` variant
- Add dispatch arm `Commands::Backlog { subcommand } => cmd::backlog::run(&root, subcommand, cli.json)`

---

## 2. Content Updates

### 2a. `GUIDANCE_MD_CONTENT` in `templates.rs`

**§6 table additions** (append after the escalate rows):

```markdown
| Add backlog item | `sdlc backlog add "<title>" --kind <concern|idea|debt>` |
| List backlog | `sdlc backlog list [--status <open|parked|promoted>]` |
| Show backlog item | `sdlc backlog show <id>` |
| Park backlog item | `sdlc backlog park <id> "<reason>"` |
```

**§12 Session Close Protocol** (new section, appended after §11):

```markdown
## 12. Backlog and Session Close Protocol

### Vocabulary

| Term | Meaning |
|---|---|
| **Backlog item** | Cross-feature, dormant concern captured by agents at moment of discovery. Lives in `.sdlc/backlog.yaml`. IDs: B1, B2… |
| **Task** | In-feature, active work item. Lives in the feature manifest. IDs: T1, T2… Written by agents and humans. |

### CRITICAL: Capture at Moment of Discovery

Capture must happen at the **moment of discovery** — not deferred to session end.
If you defer and the run crashes, the concern is permanently lost.
Session-close review is **additive** — confirm capture, then stop.

```bash
sdlc backlog add \
  "AuthMiddleware in auth.rs: token validation has a race condition under concurrent requests." \
  --kind concern \
  --source-feature my-feature \
  --evidence "crates/sdlc-server/src/auth.rs:42"
```

### Title Quality Protocol

A backlog item title must be a **complete sentence** with a **component reference**:

Good: `AuthMiddleware in auth.rs: token validation has a race condition under concurrent requests.`
Bad:  `race condition in auth`
```

### 2b. `sdlc_run.rs` template

Add a new section after "### 3. Run the loop" (or as a subsection):

```markdown
### Discovered out-of-scope concerns

When you notice a concern, bug risk, technical debt, or idea that is **outside the current feature's scope**, capture it **immediately** — before starting the next task:

```bash
sdlc backlog add \
  "ComponentName in file.rs: <what the problem is and why it matters>." \
  --kind concern \
  --source-feature <current-slug>
```

**Do not defer to session end.** If the run crashes before you record it, the concern is lost.

A well-formed title is a complete sentence with a component reference:
- Good: `RedbDatabase in orchestrator.rs: compaction not configured, may grow unbounded.`
- Bad: `compaction issue`
```

### 2c. `sdlc_next.rs` template

Add the same "Discovered out-of-scope concerns" block after "### 4. Execute the directive" section, as a numbered sub-item or a callout before step 5.

---

## 3. File Change Map

| File | Change type | Change |
|---|---|---|
| `crates/sdlc-cli/src/cmd/backlog.rs` | New file | Full `sdlc backlog` CLI subcommand |
| `crates/sdlc-cli/src/cmd/mod.rs` | Edit | `pub mod backlog;` |
| `crates/sdlc-cli/src/main.rs` | Edit | Import, Commands enum variant, dispatch arm |
| `crates/sdlc-cli/src/cmd/init/templates.rs` | Edit | §6 table + §12 section in `GUIDANCE_MD_CONTENT` |
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs` | Edit | Add out-of-scope concerns instruction |
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` | Edit | Add out-of-scope concerns instruction |
| `.sdlc/guidance.md` | Overwrite | Apply updated `GUIDANCE_MD_CONTENT` (or run `sdlc update`) |

---

## 4. Error Handling

All errors from `BacklogStore` methods bubble as `anyhow::Error` via `?`. The CLI layer prints them via the standard `eprintln!("error: {e:#}")` pattern in `main.rs`.

- `BacklogItemNotFound(id)` → exit 1, "backlog item B99 not found"
- `InvalidTransition` → exit 1, prints the transition's `reason` field
- YAML parse/serialize errors → exit 1, anyhow context

---

## 5. No Tests Added

`sdlc-core/src/backlog.rs` has comprehensive unit tests. The CLI layer is thin wiring — no new unit tests are required. Integration tests rely on `cargo clippy` and `cargo test` passing.
