# Audit: knowledge-query-mode — sdlc knowledge ask

## Scope

Security, safety, and quality audit of the `sdlc knowledge ask` feature. Components reviewed:
- `crates/sdlc-server/src/state.rs` (new SSE variants)
- `crates/sdlc-server/src/routes/events.rs` (SSE serialization)
- `crates/sdlc-server/src/routes/knowledge.rs` (ask_knowledge handler)
- `crates/sdlc-cli/src/cmd/knowledge.rs` (Ask subcommand)
- `crates/sdlc-server/src/lib.rs` (route registration)

## Findings

### A1: Input validation — PASS

`ask_knowledge` rejects empty questions with `AppError::bad_request`. The question is passed to the agent prompt as the query body, not interpolated into shell commands. No SQL, shell, or path injection vectors exist.

**Action:** None required.

### A2: Agent prompt injection — LOW RISK / ACCEPT

The user question is interpolated directly into the agent prompt:
```rust
let prompt = format!("...## Question\n{question}\n\n## Instructions\n...");
```

A malicious question string could attempt to override instructions ("ignore previous instructions..."). However:
- The agent is `spawn_agent_run`, which is already used for all other knowledge/ponder/investigation operations with user-supplied content.
- The agent runs with `max_turns: 20` — a prompt injection attempt wastes turns, not system resources.
- The agent has no destructive tools (write, delete, shell): `allowed_tools` extends sdlc base options (read-only MCP tools) plus `WebSearch` and `WebFetch`.
- The server is local-only by default; tunnels require explicit token configuration.

**Action:** Accept. No change. Document in follow-on T12: add a maximum question length cap (e.g., 2000 chars) to prevent oversized prompts.

### A3: Concurrent run key collision — PASS

The run key is derived from the question: `"knowledge:ask:{key}"` where `key` is the first 40 alphanumeric-ish chars of the question. Two different questions can share the same key if they are long and differ only after char 40. In that case, the second is a false 409.

This is a cosmetic UX issue, not a security issue: the user sees "already running" when it isn't. The question key approach prevents true concurrent duplicate queries.

**Action:** Track as T13 in a follow-on: use a random suffix or a hash of the full question for the key to eliminate false collisions.

### A4: Route ordering in lib.rs — PASS

`/api/knowledge/ask` is registered before `/api/knowledge/{slug}` and `/api/knowledge` to prevent Axum wildcard shadowing. Order is verified in `crates/sdlc-server/src/lib.rs` lines 159–175.

**Action:** None.

### A5: SSE event data size — LOW RISK / ACCEPT

`KnowledgeQueryCompleted` can carry a large `answer` field (agent output is unbounded). For long answers (>10KB), this is a single large SSE message. Downstream consumers (browser `EventSource`, CLI) handle this without fragmentation issues.

**Action:** Accept. No current risk; add a cap if production usage warrants it.

### A6: No `unwrap()` in library code — PASS

`ask_knowledge` uses `?`, `map_err`, and `unwrap_or_default` exclusively. No `unwrap()` calls in the new handler code.

**Action:** None.

### A7: Atomic writes via io.rs — N/A

This feature does not write any files. The agent run is in-memory and the run record is persisted by `spawn_agent_run` via the existing `persist_run` mechanism (which uses standard `fs::write`, consistent with other run records).

**Action:** None.

### A8: Clippy and tests — PASS

`cargo clippy --all -- -D warnings` passes clean. `SDLC_NO_NPM=1 cargo test --all` passes 31 tests with 0 failures.

**Action:** None.

## Follow-on Tasks Added

| ID | Task |
|---|---|
| T12 | Add maximum question length cap (2000 chars) with 400 response |
| T13 | Use full-question hash (not truncated prefix) for run key to eliminate false 409s |

## Verdict

No blocking security findings. The feature follows established patterns, uses existing auth/validation infrastructure, and introduces no new attack surface beyond what `research_knowledge` and `maintain_knowledge` already expose. Approved.
