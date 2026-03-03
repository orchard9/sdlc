# Tasks: sdlc ui --run-actions

## Task Breakdown

The feature has pre-existing tasks (T1–T5) from the feature manifest. This document maps them to the design and adds verification tasks.

---

### T1: Remove `no_orchestrate` field from `Commands::Ui` in `main.rs`

**File:** `crates/sdlc-cli/src/main.rs`

- Remove `/// Skip starting the orchestrator daemon` doc comment from the `Ui` variant.
- Remove `#[arg(long)] no_orchestrate: bool` field.
- Update the destructuring in the `match cli.command` dispatch arm to remove `no_orchestrate`.
- Update the `cmd::ui::run(...)` call to remove the `no_orchestrate` argument.

**Done when:** `cargo build --all` succeeds with T2 applied simultaneously.

---

### T2: Add `run_actions: bool` (default false) to `sdlc ui` command

**Files:** `crates/sdlc-cli/src/main.rs`, `crates/sdlc-cli/src/cmd/ui.rs`

In `main.rs` `Commands::Ui`:
- Add `/// Start the orchestrator daemon and execute scheduled actions` doc comment.
- Add `#[arg(long)] run_actions: bool` field (clap defaults to `false` for `bool` flags).

In `cmd/ui.rs` `UiSubcommand::Start`:
- Replace `no_orchestrate: bool` with `run_actions: bool` (same doc comment as above).

Update all function signatures:
- `run(... run_actions: bool)` in `ui.rs`
- `run_start(... run_actions: bool)` in `ui.rs`
- Dispatch arm in `run()` destructuring `UiSubcommand::Start { ..., run_actions: ra }`.

**Done when:** `sdlc ui --help` shows `--run-actions`, does not show `--no-orchestrate`.

---

### T3: Flip orchestrator spawn logic — only start when `run_actions == true`

**File:** `crates/sdlc-cli/src/cmd/ui.rs`

Change:
```rust
if !no_orchestrate {
```
to:
```rust
if run_actions {
```

All other orchestrator spawn code is unchanged.

**Done when:** Running `sdlc ui` (without `--run-actions`) does not spawn the orchestrator thread; running `sdlc ui --run-actions` does.

---

### T4: Update `DEVELOPER.md` to replace `--no-orchestrate` with `--run-actions`

**File:** `DEVELOPER.md`

- Find all mentions of `--no-orchestrate` and replace with `--run-actions`.
- Update prose: change descriptions from "skip the orchestrator" framing to "enable the orchestrator / action execution" framing.
- Ensure the quick-start example in DEVELOPER.md (if any) shows `sdlc ui --run-actions` for the action-execution workflow.

**Done when:** No mention of `--no-orchestrate` remains in `DEVELOPER.md`.

---

### T5: [user-gap] Evaluate config-file project-wide opt-out escape hatch

**File:** Backlog or `.sdlc/features/`

- Determine whether a `.sdlc/config.yaml` setting (e.g., `orchestrator.auto_run: false`) is needed for teams that want to enforce "never auto-run" even when `--run-actions` is passed.
- If trivial (1–2 lines in `run_start`): implement now.
- If non-trivial: create a backlog item documenting the gap and close T5.

**Done when:** A decision is recorded — either implemented or backlogged with rationale.

---

## Verification Tasks

### TV1: Build verification

Run:
```bash
cargo build --all
```
Expected: zero errors, zero new warnings.

### TV2: Test suite

Run:
```bash
SDLC_NO_NPM=1 cargo test --all
```
Expected: all tests pass.

### TV3: Clippy

Run:
```bash
cargo clippy --all -- -D warnings
```
Expected: zero warnings treated as errors.

### TV4: CLI smoke test

Run:
```bash
./target/debug/sdlc ui --help
./target/debug/sdlc ui start --help
```
Expected:
- `--run-actions` appears in both help outputs.
- `--no-orchestrate` does NOT appear.
