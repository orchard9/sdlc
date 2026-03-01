# Tasks: orchestrator-tick-cli

## T1: Add orchestrator_db_path to paths.rs

Add `pub fn orchestrator_db_path(root: &Path) -> PathBuf` returning `root/.sdlc/orchestrator.db`.

## T2: Create cmd/orchestrate.rs

Full implementation: OrchestrateSubcommand enum, parse_at(), run().
- Daemon tick loop with std::thread::sleep
- add subcommand: parse_at, create Action, db.insert
- list subcommand: db.list_all, filter by status, print_table
- run_tool dispatch with per-action error handling (never crash daemon)
- rescheduling logic for recurrence

## T3: Register in cmd/mod.rs

Add `pub mod orchestrate;` to `crates/sdlc-cli/src/cmd/mod.rs`.

## T4: Wire into main.rs

- Import `OrchestrateSubcommand`
- Add `Orchestrate { tick_rate, db, subcommand }` to `Commands` enum
- Add dispatch arm in `match cli.command`

## T5: Add .sdlc/orchestrator.db to .gitignore

Append `.sdlc/orchestrator.db` to root `.gitignore`.

## T6: Build and smoke test

```bash
SDLC_NO_NPM=1 cargo build -p sdlc-cli 2>&1 | tail -5
cargo clippy -p sdlc-cli -- -D warnings 2>&1 | tail -5
./target/debug/sdlc orchestrate --help
./target/debug/sdlc orchestrate add test-action --tool quality-check --input '{}' --at now+5s
./target/debug/sdlc orchestrate list
```

All commands print expected output, no panics.
