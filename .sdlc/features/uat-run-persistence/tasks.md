# Tasks: uat-run-persistence

## Task List

- [ ] Add path helpers to `paths.rs`: `milestone_uat_runs_dir`, `uat_run_dir`, `uat_run_manifest`
- [ ] Add `UatVerdict` enum and `UatRun` struct to `milestone.rs`
- [ ] Implement `save_uat_run`, `list_uat_runs`, `latest_uat_run` in `milestone.rs`
- [ ] Add integration tests: round_trip, list_sorted_newest_first, latest_returns_none
- [ ] Verify all tests pass with `SDLC_NO_NPM=1 cargo test --all`
- [ ] Verify zero clippy warnings with `cargo clippy --all -- -D warnings`
