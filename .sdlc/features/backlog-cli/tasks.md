# Tasks: sdlc backlog CLI commands

## Task List

### T1 — Create `crates/sdlc-cli/src/cmd/backlog.rs` with `BacklogSubcommand` enum and `run()` dispatch

Create the file with the `BacklogSubcommand` enum (Add, List, Park, Promote, Show variants) and the `pub fn run(root: &Path, subcmd: BacklogSubcommand, json: bool) -> anyhow::Result<()>` dispatch function. Stub each arm with `todo!()` initially; filled in by T2–T6.

**Acceptance:** File compiles with `cargo check`.

---

### T2 — Implement `add` subcommand with auto-inference and required confirmation output

Implement the `add` arm:
- Join variadic title vec
- Auto-infer `source_feature` from `State::load` → `active_features.last()` when `--source-feature` omitted; print stderr warning if empty
- Call `BacklogStore::add`
- Print: `Backlog item {id} recorded: "{title}" [{source}]` — required exact format for agent auditability

**Acceptance:** `sdlc backlog add auth race condition --kind concern` prints the required confirmation string with ID and source.

---

### T3 — Implement `list` subcommand with status and source-feature filters

Implement the `list` arm:
- Default: open items only (`BacklogStatus::Open`)
- `--all`: no status filter
- `--status <s>`: parse to `BacklogStatus`; conflicts with `--all`
- `--source-feature <slug>`: pass to `BacklogStore::list`
- Print table with columns: `ID | KIND | STATUS | SOURCE | TITLE`

**Acceptance:** `sdlc backlog list` shows only open items; `sdlc backlog list --all` shows all; `sdlc backlog list --source-feature my-slug` filters correctly.

---

### T4 — Implement `park` subcommand (required `--reason`)

Implement the `park` arm:
- `--reason` is a required Clap argument (Vec<String>, joined with spaces)
- Call `BacklogStore::park(root, &id, reason)`
- Print: `Parked {id}: {reason}`
- Error if reason is empty/whitespace (enforced by core)

**Acceptance:** `sdlc backlog park B1 --reason "revisit after v14"` succeeds; `sdlc backlog park B1` without `--reason` exits with clap usage error.

---

### T5 — Implement `promote` subcommand with optional `--slug` and `--milestone`

Implement the `promote` arm:
- Load item via `BacklogStore::get`
- Derive feature slug from `--slug` or `slugify(&item.title)` helper (unit-tested inline)
- Create feature: `Feature::create_with_description`
- Update state: load → `add_active_feature` → save
- `BacklogStore::mark_promoted`
- If `--milestone`: load milestone → `add_feature` → save
- Print required output lines

**Acceptance:** `sdlc backlog promote B1` creates a feature and marks B1 as promoted; `--milestone` links the feature to the milestone.

---

### T6 — Implement `show` subcommand

Implement the `show` arm:
- Load item via `BacklogStore::get`
- Print all fields, skipping None fields in human mode
- JSON mode: `print_json(&item)`

**Acceptance:** `sdlc backlog show B1` prints all populated fields; `sdlc backlog show B99` exits with error.

---

### T7 — Wire up: add `pub mod backlog` to `cmd/mod.rs` and `Backlog` variant to `main.rs`

- Add `pub mod backlog;` to `crates/sdlc-cli/src/cmd/mod.rs`
- Add `use cmd::backlog::BacklogSubcommand;` to `main.rs`
- Add `Backlog { subcommand: BacklogSubcommand }` variant to `Commands` enum
- Add dispatch arm `Commands::Backlog { subcommand } => cmd::backlog::run(&root, subcommand, cli.json)`

**Acceptance:** `cargo build --all` succeeds; `sdlc backlog --help` prints subcommand list.

---

### T8 — Add integration tests for backlog CLI

Add tests in `crates/sdlc-cli/tests/integration.rs`:
- `backlog_add_prints_required_format` — verifies confirmation string contains ID and source brackets
- `backlog_list_defaults_to_open` — adds an item, parks it, verifies `list` without flags omits parked item
- `backlog_park_requires_reason` — verifies exit code non-zero when `--reason` omitted
- `backlog_promote_creates_feature` — verifies feature directory is created and B1 is marked promoted

**Acceptance:** `SDLC_NO_NPM=1 cargo test --all` passes.

---

## Pre-existing tasks from feature creation (absorbed into implementation above)

The following tasks were pre-seeded in the manifest and are covered by T1–T8 above:
- T1 (manifest) → covered by T1 above
- T2 (manifest) → covered by T2 above
- T3 (manifest) → covered by T3 above
- T4 (manifest) → covered by T4, T5, T6 above
- T5 (manifest) → covered by T7 above
- T6 (user-gap: auto-infer source_feature) → covered by T2 above
- T7 (user-gap: confirmation output format) → covered by T2 above
- T8 (user-gap: --milestone flag on promote) → covered by T5 above
- T9 (user-gap: --source-feature filter on list) → covered by T3 above
- T10 (user-gap: park requires --reason) → covered by T4 above
