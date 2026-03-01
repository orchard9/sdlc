# QA Plan: orchestrator-action-model

## Gate: cargo test

```bash
SDLC_NO_NPM=1 cargo test -p sdlc-core -- orchestrator
```

All 5 unit tests must pass.

## Gate: cargo clippy

```bash
cargo clippy -p sdlc-core -- -D warnings
```

Zero warnings.

## Correctness checks

- `range_due` with an empty DB returns an empty Vec (no panic)
- `set_status` on a nonexistent UUID returns `Err(OrchestratorDb(...))`
- `insert` called twice with the same action id does not panic (upsert or error)
- `startup_recovery` called on an empty DB returns `Ok(0)`
- `list_all` returns actions sorted by `created_at` descending

## Key property verification

The composite key must sort by timestamp before UUID. Verify with:
- Insert two actions where action_a has a later timestamp but a lower UUID byte sequence
- Confirm `range_due(between)` returns only action_b (the earlier timestamp)

## Build integration

`SDLC_NO_NPM=1 cargo build --all` succeeds with redb added to workspace.
No unused import warnings, no dead code warnings.
