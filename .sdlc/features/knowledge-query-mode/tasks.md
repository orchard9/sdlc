# Tasks: knowledge-query-mode

## T1 — Add `CitedEntry` struct and SSE variants to `state.rs`

**File:** `crates/sdlc-server/src/state.rs`

Add above the `SseMessage` enum:

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CitedEntry {
    pub slug: String,
    pub code: String,
    pub title: String,
}
```

Add to `SseMessage`:

```rust
KnowledgeQueryStarted { question: String },
KnowledgeQueryCompleted {
    answer: String,
    cited_entries: Vec<CitedEntry>,
    gap_detected: bool,
    gap_suggestion: Option<String>,
},
```

---

## T2 — Serialize new SSE variants in `events.rs`

**File:** `crates/sdlc-server/src/routes/events.rs`

Extend the `SseMessage` → JSON conversion (the `match` in `sse_events` or its helper) to handle the two new variants. Follow the exact pattern of existing variants.

`KnowledgeQueryStarted`:
```json
{"type": "KnowledgeQueryStarted", "question": "..."}
```

`KnowledgeQueryCompleted`:
```json
{
  "type": "KnowledgeQueryCompleted",
  "answer": "...",
  "cited_entries": [...],
  "gap_detected": false,
  "gap_suggestion": null
}
```

---

## T3 — Add `ask_knowledge` handler to `routes/knowledge.rs`

**File:** `crates/sdlc-server/src/routes/knowledge.rs`

Add at the bottom of the file:

```rust
#[derive(serde::Deserialize)]
pub struct AskKnowledgeBody {
    pub question: String,
}

pub async fn ask_knowledge(
    State(app): State<AppState>,
    Json(body): Json<AskKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> { ... }
```

Implementation steps inside the handler:
1. Reject empty `question` with `AppError::bad_request`.
2. Load catalog + entries in `spawn_blocking`.
3. Build prompt via `build_ask_prompt(catalog, entries, question)`.
4. Build `QueryOptions` via `knowledge_ask_query_options(root)` (private helper in the same file).
5. Call `spawn_agent_run` with:
   - key: `"knowledge:ask"`
   - run_type: `"knowledge"`
   - label: `"Knowledge query"`
   - completion_event: `Some(SseMessage::KnowledgeQueryCompleted { ... })` — populated by parsing agent output after streaming completes.

**Note:** `spawn_agent_run` returns on duplicate with `409` automatically — no extra check needed.

Private helpers in same file:
- `fn build_ask_prompt(catalog, entries, question) -> String` — formats catalog tree + entry table + question + instructions.
- `fn knowledge_ask_query_options(root) -> QueryOptions` — extends `sdlc_guideline_query_options` but clears `mcp_servers` and sdlc tools, keeping only `WebSearch` and `WebFetch`.
- `fn extract_citations(text, entries) -> Vec<CitedEntry>` — regex `\[(\d+(?:\.\d+)?)\s+([\w-]+)\]` over answer text, resolved against entry list.
- `fn extract_gap(text) -> (bool, Option<String>)` — looks for sentinel ` ```json\n{"gap": true, ...}\n``` ` block and parses it.

---

## T4 — Register route in `lib.rs`

**File:** `crates/sdlc-server/src/lib.rs`

Add before the `"/api/knowledge"` catch-all route:

```rust
.route(
    "/api/knowledge/ask",
    post(routes::knowledge::ask_knowledge),
)
```

---

## T5 — Add `Ask` subcommand to CLI

**File:** `crates/sdlc-cli/src/cmd/knowledge.rs`

Add variant to `KnowledgeSubcommand`:

```rust
/// Ask the knowledge librarian a question
Ask {
    /// Question (free-form natural language)
    question: String,
},
```

Add match arm in `run()`:

```rust
KnowledgeSubcommand::Ask { question } => ask(root, &question, json),
```

Implement `fn ask(root: &Path, question: &str, json: bool) -> anyhow::Result<()>`:

1. Load server port from `sdlc_core::config::Config::load(root)` or default `7842`.
2. `POST http://localhost:{port}/api/knowledge/ask {"question": question}` via `ureq`.
3. Handle `409` → print busy message and return early.
4. Handle non-200/202 → return error.
5. Parse `run_id` from response body.
6. Subscribe to `GET http://localhost:{port}/api/events` (SSE) via `ureq` in blocking mode.
7. Read lines: look for `data:` prefix containing `"type":"KnowledgeQueryCompleted"`.
8. On match: parse answer, cited_entries, gap_detected, gap_suggestion.
9. Print formatted output (or JSON if `--json`).
10. If server not reachable: print helpful message `"sdlc server not running — start with \`sdlc ui\`"`.

---

## T6 — Unit tests for prompt builder and parsers

**File:** `crates/sdlc-server/src/routes/knowledge.rs` (test module at bottom)

Tests:
- `build_ask_prompt` includes catalog tree, entry titles, and question in output.
- `extract_citations` correctly parses `[100.20 slug-name]` and skips unknown slugs.
- `extract_gap` returns `(true, Some("topic"))` given a well-formed sentinel block.
- `extract_gap` returns `(false, None)` given text with no sentinel block.

---

## T7 — Integration tests in server test suite

**File:** `crates/sdlc-server/tests/integration.rs`

Tests:
- `POST /api/knowledge/ask` with `{"question": ""}` → `400`.
- `POST /api/knowledge/ask` with `{"question": "hello"}` against a temp dir with no knowledge entries → `202`, body contains `run_id`.

---

## T8 — Update CLAUDE.md knowledge CLI command reference

**File:** `crates/sdlc-cli/src/cmd/init/mod.rs` or `templates.rs`

Update the `GUIDANCE_MD_CONTENT` command reference table entry for `sdlc knowledge` to include `ask '<question>'` in the description so agents know the command exists.
