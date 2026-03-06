# Design: Fix Human UAT PassWithTasks Skips Release

## Change

Single-line fix in `crates/sdlc-server/src/routes/runs.rs`.

### Current (line 1235)
```rust
if verdict == UatVerdict::Pass {
```

### Proposed
```rust
if verdict == UatVerdict::Pass || verdict == UatVerdict::PassWithTasks {
```

Alternatively, using `matches!`:
```rust
if matches!(verdict, UatVerdict::Pass | UatVerdict::PassWithTasks) {
```

## Rationale

`PassWithTasks` is semantically a pass — the UAT succeeded, but follow-up tasks were logged. The milestone should release in both cases. The `matches!` form is preferred as it's more idiomatic and easier to extend if new passing variants are added.

## Test Strategy

Add an integration test in `crates/sdlc-server/tests/integration.rs` that:
1. Creates a milestone in Verifying status
2. POSTs to `/api/milestone/{slug}/uat/human` with `verdict: "pass_with_tasks"` and notes
3. Asserts the milestone now has `released_at` set

## Files Changed

| File | Change |
|------|--------|
| `crates/sdlc-server/src/routes/runs.rs` | Fix release guard conditional |
| `crates/sdlc-server/tests/integration.rs` | Add PassWithTasks release test |
