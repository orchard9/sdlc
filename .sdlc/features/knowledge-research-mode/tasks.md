# Tasks: knowledge-research-mode

## T1 — Add `OriginKind::Research` variant to `sdlc-core`

**File:** `crates/sdlc-core/src/knowledge.rs`

Add `Research` to the `OriginKind` enum. Update `Display` and `FromStr`
implementations. Serialize as `"research"`. Add a unit test confirming the
round-trip `"research"` → `OriginKind::Research` → `"research"`.

---

## T2 — Add SSE variants `KnowledgeResearchStarted` and `KnowledgeResearchCompleted`

**File:** `crates/sdlc-server/src/state.rs`

Add to `SseMessage`:
```rust
KnowledgeResearchStarted { slug: String },
KnowledgeResearchCompleted { slug: String },
```

Extend `to_sse_event()` to serialize both under the `"knowledge"` event channel
as JSON objects with a `"type"` discriminator field.

---

## T3 — Implement `POST /api/knowledge/:slug/research` server handler

**File:** `crates/sdlc-server/src/routes/knowledge.rs`

Implement `research_knowledge`:
- Accept optional `{ "topic": "..." }` JSON body.
- Load or create the knowledge entry (set `origin = Research` when creating).
- Emit `KnowledgeResearchStarted` SSE before spawning the agent.
- Call `spawn_agent_run` with key `"knowledge:{slug}"`, max_turns 20, and the
  static research prompt.
- Return 409 if a run with that key is already active.
- Return 202 with `{ "slug": "...", "started": true }`.

Include `build_research_prompt(slug, title, topic, root)` as a private function
returning the full agent instruction string.

---

## T4 — Register the new route in `sdlc-server`

**File:** `crates/sdlc-server/src/lib.rs`

Add:
```rust
.route("/api/knowledge/:slug/research", post(knowledge::research_knowledge))
```

Ensure the route is placed after the existing knowledge routes in the router.

---

## T5 — Add `Research` subcommand to `sdlc knowledge` CLI

**File:** `crates/sdlc-cli/src/cmd/knowledge.rs`

Add variant to `KnowledgeSubcommand`:
```rust
Research {
    topic: String,
    #[arg(long)]
    code: Option<String>,
},
```

Implement handler `research(root, topic, code, json)`:
- Slugify the topic to derive the slug.
- Ensure entry exists locally (create if missing).
- Discover server port via `read_server_port(root)`.
- POST to `/api/knowledge/{slug}/research`.
- On 202: print confirmation or JSON.
- On error: return descriptive anyhow error.

Implement `read_server_port(root: &Path) -> anyhow::Result<u16>` following the
same pattern used in `crates/sdlc-cli/src/cmd/orchestrate.rs`.

---

## T6 — Integration test for research endpoint

**File:** `crates/sdlc-server/tests/integration.rs`

Add test `test_knowledge_research_endpoint`:
- Creates a test `AppState` with a temp dir.
- POSTs to `/api/knowledge/test-topic/research` with `{ "topic": "test" }`.
- Asserts 202 status.
- Asserts response body contains `"started": true`.
- (Agent run is not awaited — this is a fire-and-return test only.)

---

## T7 — Unit test: `OriginKind::Research` round-trip (sdlc-core)

Covered in T1. Listed separately so it tracks independently in CI.

---

## T8 — Verify `cargo test --all` and `cargo clippy --all -- -D warnings` pass

After all code changes, run both commands and fix any failures before marking
implementation complete.
