# Code Review: Spike REST Routes

## Summary

Three REST endpoints were added to expose the spike data layer over HTTP.
The implementation follows established server patterns faithfully.

## Files Changed

- `crates/sdlc-server/src/routes/spikes.rs` (new, 445 lines incl. tests)
- `crates/sdlc-server/src/routes/mod.rs` (1 line added)
- `crates/sdlc-server/src/lib.rs` (7 lines added)

## Findings

### Correctness

- All three handlers (`list_spikes`, `get_spike`, `promote_spike`) correctly
  delegate to `sdlc_core::spikes` public API inside `spawn_blocking`.
- `get_spike` correctly maps `SdlcError::Io(NotFound)` to `AppError::not_found`,
  producing a 404 response.
- `promote_spike` correctly gates on `SpikeVerdict::Adapt` and returns 422 via
  `AppError::unprocessable_json` for ADOPT, REJECT, and null verdicts.
- Idempotency of `promote_to_ponder` is handled by the underlying data layer.

### Error Handling

- Uses existing `AppError` sentinel constructors (`not_found`, `unprocessable_json`)
  — no new error types introduced.
- `spawn_blocking` join errors are wrapped in `AppError(anyhow::anyhow!(...))`.

### Patterns

- Route registration in `lib.rs` follows the same style as investigations,
  knowledge, and roadmap routes.
- Module declaration in `routes/mod.rs` is alphabetically ordered.
- No `unwrap()` in production code paths.
- All file I/O occurs inside `spawn_blocking`, never blocking the async executor.

### Tests

- 10 unit tests covering all three endpoints and their error cases.
- Tests use `build_router_for_test` with `TempDir` — fully isolated.
- `promote_spike` tests cover all four verdict states (ADOPT, ADAPT, REJECT, null).
- All 10 tests pass; full test suite (`cargo test --all`) shows 0 failures.
- `cargo clippy --all -- -D warnings` passes with no warnings.

### Optional body in promote endpoint

- `promote_spike` accepts `Option<Json<PromoteBody>>` rather than `Json<PromoteBody>`.
  This allows clients to send an empty `POST` with no body (common for promotions
  where the default slug is acceptable) without a 422 from axum's body extractor.
  This is intentional and correct.

## Verdict: Approved

No blocking issues found. All spec requirements are met. Test coverage is thorough.
