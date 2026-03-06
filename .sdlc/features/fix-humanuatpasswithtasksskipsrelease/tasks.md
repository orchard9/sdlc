# Tasks: Fix Human UAT PassWithTasks Skips Release

## Tasks

- [ ] **T1**: Fix release guard in `submit_milestone_uat_human` — change `verdict == UatVerdict::Pass` to `matches!(verdict, UatVerdict::Pass | UatVerdict::PassWithTasks)` at `crates/sdlc-server/src/routes/runs.rs:1235`
- [ ] **T2**: Add integration test `human_uat_pass_with_tasks_releases_milestone` in `crates/sdlc-server/tests/integration.rs` — submit PassWithTasks verdict, assert `released_at` is set
- [ ] **T3**: Run `cargo clippy --all -- -D warnings` and `SDLC_NO_NPM=1 cargo test --all` to verify no regressions
