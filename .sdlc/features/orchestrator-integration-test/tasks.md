# Tasks: Orchestrator Integration Test

## T1 — Extract `run_one_tick` from `run_daemon` in `orchestrate.rs`

**File**: `crates/sdlc-cli/src/cmd/orchestrate.rs`

Extract the inner body of the daemon loop into a `pub fn run_one_tick(root: &Path, db: &ActionDb) -> Result<()>` function. Update `run_daemon` to call `run_one_tick`. This makes the tick logic directly testable without spawning threads.

**Done when**: `run_one_tick` is `pub`, `run_daemon` delegates to it, existing behavior is unchanged.

---

## T2 — Write integration test: happy path (two actions complete)

**File**: `crates/sdlc-cli/tests/integration.rs`

Add test `orchestrator_two_actions_complete_in_one_tick`:
- Create `TempDir`.
- Write `.sdlc/tools/stub-tool/tool.ts` with content `console.log(JSON.stringify({ok:true}));`.
- Guard: skip if `detect_runtime()` is `None`.
- Open `ActionDb` at `root/.sdlc/orchestrator.db`.
- Insert two `Pending` actions with `tool_name="stub-tool"`, `next_tick_at=Utc::now() + 100ms` and `+200ms`.
- `std::thread::sleep(Duration::from_millis(300))`.
- Call `sdlc_cli::cmd::orchestrate::run_one_tick(&root, &db)`.
- Assert `db.list_all()` has exactly 2 actions, both `ActionStatus::Completed`.

**Done when**: Test compiles, passes in environments with a JS runtime, and is skipped cleanly without one.

---

## T3 — Write integration test: startup recovery

**File**: `crates/sdlc-cli/tests/integration.rs`

Add test `orchestrator_startup_recovery_marks_stale_running_as_failed`:
- Create `TempDir`.
- Open `ActionDb`.
- Construct `Action` directly: `label="stale"`, `tool_name="noop"`, `status=ActionStatus::Running`, `updated_at=Utc::now() - 10 minutes`.
- Call `db.insert(&action)`.
- Call `db.startup_recovery(Duration::from_secs(120))`.
- Assert return value is `1`.
- Assert `db.list_all()[0].status` matches `ActionStatus::Failed { reason }` where `reason.contains("recovered")`.

**Done when**: Test compiles and passes with `SDLC_NO_NPM=1 cargo test --all`.

---

## T4 — Make `run_one_tick` accessible from integration tests

**File**: `crates/sdlc-cli/src/cmd/orchestrate.rs` and/or `crates/sdlc-cli/src/cmd/mod.rs`

Ensure `run_one_tick` is accessible from `crates/sdlc-cli/tests/integration.rs`. Integration tests in the same crate can access `pub` items in `src/`. Verify the module path is correct (e.g. `sdlc_cli::cmd::orchestrate::run_one_tick` if `sdlc-cli` exposes a lib target, or use the binary's internal module structure).

If `sdlc-cli` is binary-only (no `lib.rs`), add a `lib.rs` that re-exports the orchestrate module, or move `run_one_tick` to `sdlc-core` as a standalone tick helper. Prefer the minimal change.

**Done when**: Integration test can call `run_one_tick` without a `use` error.

---

## T5 — Run gate: `SDLC_NO_NPM=1 cargo test --all` passes

Run `SDLC_NO_NPM=1 cargo test --all` and confirm:
- All existing tests still pass.
- The two new tests appear in output.
- No compilation errors.
- Clippy clean: `cargo clippy --all -- -D warnings`.

**Done when**: Exit code 0 for both commands.
