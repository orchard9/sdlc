---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Design complete — redb + composite key + game loop tick, Phase 1 scoped to scheduled actions only"
  next: "Build Phase 1: Action struct + redb wrapper in sdlc-core, sdlc orchestrate command in sdlc-cli, integration test"
  commit: "Phase 1 integration test passes: two scheduled actions fire within a 500ms tick"
---

## Session 1

Jordan opened with a clear mental model: the orchestrator is a game loop. Actions
have two trigger states — scheduled (timestamp) and webhook (stored raw). One tick
runs at a time, processes everything due, sleeps the remainder of the tick interval.

Start simple: get the tick rate working, then expand.

### Brief Analysis

The design is elegantly constrained:
- No distributed coordination — one orchestrator per project
- No queue infrastructure — the DB is the queue
- No transformation on ingress (webhooks) — store raw, process on tick
- The tick loop doesn't know what actions do — it just finds, marks, dispatches, marks

The hard part is the DB key design for efficient "get all actions due now" queries.

### DB Choice

? Open: sled vs redb vs rusqlite for the embedded DB

⚑ Decided: **redb** — pure Rust, ACID, range-scannable, actively maintained.
Rationale: rusqlite has C FFI (cross-compile friction), sled has maintenance
concerns. redb gives typed tables with ordered keys, exactly what timestamp-keyed
action range scans need.

### Key Design Insight

**Priya Nair · Distributed Systems**

The composite key `timestamp_ms (u64 be) ++ uuid (16 bytes)` = 24-byte array is
the critical pattern. Big-endian byte ordering means timestamp bytes sort correctly.
A single range scan `..=due_upper_bound()` returns all due actions in chronological
order. This is the only DB pattern needed for Phase 1.

One addition from Priya: on startup, sweep for `Running` actions older than 2×
tick_rate and mark them `Failed { reason: "recovered from restart" }`. Prevents
silent loss on restart.

### Correctness Property

⚑ Decided: Mark action `Running` in the DB (inside a write transaction) BEFORE
executing. This is the only correctness property that matters: no action fires twice
on restart.

### Where It Lives

**Marcus Webb · Enterprise Platform**

⚑ Decided: `sdlc orchestrate` CLI command (foreground daemon). Same pattern as
existing CLI commands. DB at `.sdlc/orchestrator.db` — add to .gitignore, it's
operational state not audit state. Git tracks `.sdlc/` YAML; redb file is disposable.

Location:
- `crates/sdlc-core/src/orchestrator/` — Action struct, ActionDb, redb wrapper
- `crates/sdlc-cli/src/cmd/orchestrate.rs` — tick loop, CLI entry point

### Phase 1 Scope

**Dana Cho · Product Skeptic**

Phase 1 = tick rate works, scheduled actions dispatch, integration test passes.
Webhooks are Phase 2. Management API is Phase 3. Web UI is Phase 3.

The gate: integration test — schedule two actions 100ms apart, tick rate 500ms,
verify both fired. If that passes, the model is sound.

### Open Questions

? Open: What does an action "execute" in Phase 1? Options:
  - Shell command (most generic, matches existing gate model)
  - `sdlc next --for <slug>` + spawn Claude agent (the full vision)
  - Configurable per action (cleanest but more scope)
  Recommendation: shell command for Phase 1, configurable handler later.

? Open: How are actions created in Phase 1 (no management API yet)? Options:
  - CLI: `sdlc orchestrate add <slug> --at <timestamp>`
  - Hardcoded in test only
  Recommendation: minimal `sdlc orchestrate add` subcommand, enough to test.
