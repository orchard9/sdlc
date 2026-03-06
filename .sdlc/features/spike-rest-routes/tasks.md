# Tasks: Spike REST Routes

## Task List

1. **Create `crates/sdlc-server/src/routes/spikes.rs`**
   - Implement `list_spikes` handler (GET /api/spikes)
   - Implement `get_spike` handler (GET /api/spikes/:slug) with 404 on NotFound
   - Implement `promote_spike` handler (POST /api/spikes/:slug/promote) with 422 on non-ADAPT verdict
   - All handlers use `spawn_blocking` over `sdlc_core::spikes` functions

2. **Register module and routes**
   - Add `pub mod spikes;` to `crates/sdlc-server/src/routes/mod.rs`
   - Add three route registrations to `build_router_from_state` in `crates/sdlc-server/src/lib.rs`

3. **Write integration tests**
   - Test GET /api/spikes returns empty array when no spikes dir exists
   - Test GET /api/spikes/:slug returns 404 for unknown slug
   - Test POST /api/spikes/:slug/promote returns 422 for ADOPT/REJECT verdicts
   - Test POST /api/spikes/:slug/promote returns ponder_slug for ADAPT verdict
   - Tests use `build_router_for_test` and `TempDir`

4. **Build and verify**
   - `SDLC_NO_NPM=1 cargo test --all` passes
   - `cargo clippy --all -- -D warnings` passes
