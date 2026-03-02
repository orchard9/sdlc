# Design: knowledge-query-mode — sdlc knowledge ask

## Architecture Overview

The feature adds a read-side intelligence layer to the existing knowledge base. It follows the same pattern used by ponder chat and investigation chat: a CLI subcommand fires a POST to the local server, the server builds a prompt from local state and calls `spawn_agent_run`, the agent streams over SSE, and the CLI or UI consumes the events.

The design adds exactly three touch-points to the existing codebase:

1. **`crates/sdlc-server/src/state.rs`** — two new `SseMessage` variants + a `CitedEntry` struct.
2. **`crates/sdlc-server/src/routes/knowledge.rs`** — one new handler `ask_knowledge` for `POST /api/knowledge/ask`.
3. **`crates/sdlc-cli/src/cmd/knowledge.rs`** — one new `Ask` variant on `KnowledgeSubcommand` + its handler.

No new files are required. No new crate dependencies.

## Component Diagram

```
┌───────────────────────┐
│  CLI: sdlc knowledge  │
│        ask '<q>'      │
└──────────┬────────────┘
           │ POST /api/knowledge/ask {question}
           ▼
┌───────────────────────────────────────────────────────────┐
│  sdlc-server: routes/knowledge.rs :: ask_knowledge        │
│                                                           │
│  1. Validate question non-empty                           │
│  2. Load catalog + all entries (slug, code, title, summary)│
│  3. Build librarian prompt                                │
│  4. spawn_agent_run("knowledge:ask", prompt, opts, ...)   │
│     - run_type = "knowledge"                              │
│     - label    = "Knowledge query"                        │
│     - completion_event = KnowledgeQueryCompleted{...}     │
│  5. Return 202 {run_id}                                   │
└───────────────────────────────────────────────────────────┘
           │  spawn
           ▼
┌───────────────────────────────────────────────────────────┐
│  tokio task: claude_agent::query(prompt, opts)            │
│  - max_turns: 10                                          │
│  - allowed_tools: [WebSearch, WebFetch]                   │
│  - streams events to broadcast channel                    │
│  - on completion: parse agent output for gap JSON         │
│  - emit KnowledgeQueryCompleted via event_tx              │
└───────────────────────────────────────────────────────────┘
           │  SSE /api/events
           ▼
┌───────────────────────┐
│  CLI (waits for       │
│  KnowledgeQuery-      │
│  Completed event)     │
│  prints answer +      │
│  citations            │
└───────────────────────┘
```

## State.rs Changes

### New struct: `CitedEntry`

Added above the `SseMessage` enum — plain serializable data, no logic.

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CitedEntry {
    pub slug: String,
    pub code: String,
    pub title: String,
}
```

### New `SseMessage` variants

Added to the existing `SseMessage` enum:

```rust
/// A knowledge query agent run has started.
KnowledgeQueryStarted { question: String },
/// A knowledge query agent run completed.
KnowledgeQueryCompleted {
    answer: String,
    cited_entries: Vec<CitedEntry>,
    gap_detected: bool,
    gap_suggestion: Option<String>,
},
```

The existing `events.rs` serialisation switch that converts `SseMessage` to JSON must handle both new variants. The serialization pattern is identical to other variants — pattern-match and emit a JSON object with a `"type"` discriminant.

## Server Handler: `ask_knowledge`

Location: `crates/sdlc-server/src/routes/knowledge.rs`

```
POST /api/knowledge/ask
Body: { "question": "<free-form text>" }
```

### Prompt builder (inline function `build_ask_prompt`)

Loads:
- `sdlc_core::knowledge::load_catalog(&root)` → taxonomy tree (classes + divisions)
- `sdlc_core::knowledge::list(&root)` → all entries; for each: slug, code, title, summary

Formats catalog as a nested markdown list and entries as a simple table.

Maximum prompt size: entries are capped at 500 chars summary each to avoid hitting context limits. The catalog tree itself is typically small (<5 KB).

### Gap detection

The agent is instructed to end its response with a JSON sentinel block when it detects a gap:

```
```json
{"gap": true, "suggestion": "<topic>"}
```
```

The completion handler runs a simple regex/string scan over the final agent text to extract this block. If found, `gap_detected = true` and `gap_suggestion` is populated. If not found, both are false/None.

This parsing is done inside the `completion_event` closure — the pattern used by advisory.rs demonstrates how to post-process agent output before emitting the domain event.

### Citation extraction

The agent is instructed to cite entries as `[CODE slug]` e.g. `[100.20 trunk-based-dev]`. The completion handler runs a regex `\[(\d+(?:\.\d+)?) ([\w-]+)\]` over the full answer text and resolves each match against the loaded entry list to produce `Vec<CitedEntry>`. Unknown slugs are silently skipped.

### Route registration in `lib.rs`

```rust
.route(
    "/api/knowledge/ask",
    post(routes::knowledge::ask_knowledge),
)
```

Added after the existing `/api/knowledge/catalog` route, before `/api/knowledge`.

## CLI Handler: `Ask` subcommand

Location: `crates/sdlc-cli/src/cmd/knowledge.rs`

```rust
/// Ask the knowledge librarian a question
Ask {
    /// Question (free-form natural language)
    question: String,
},
```

Added to `KnowledgeSubcommand` enum and matched in `run()`.

### Handler flow

```
fn ask(root: &Path, question: &str, json: bool) -> anyhow::Result<()>
```

1. Discover server URL from `sdlc_core::config::Config::load(root)` → `port`. If no server, print error and return.
2. `POST http://localhost:{port}/api/knowledge/ask` with `{"question": question}`.
3. On `409 Conflict` → print "A knowledge query is already running. Wait for it to complete."
4. On `202` → subscribe to `GET http://localhost:{port}/api/events` (SSE).
5. Filter for events of type `KnowledgeQueryCompleted`. Print a spinner or progress dots while waiting.
6. On completion: parse `answer`, `cited_entries`, `gap_detected`, `gap_suggestion` from event data.
7. Print formatted output.

### Output format (non-JSON)

```
Answer
──────
<answer text>

Sources
───────
[100.20] Trunk-based development (trunk-based-dev)
[200.10] Git branching conventions (git-branch-policy)

Knowledge gap detected.
Run: sdlc knowledge research "deployment pipeline overview"
```

### HTTP client in CLI

Uses the existing `ureq` dependency (already in `sdlc-cli/Cargo.toml`) for the POST. For SSE streaming from the CLI, uses `ureq` in blocking mode reading the event stream line-by-line until the `KnowledgeQueryCompleted` event is seen.

## QueryOptions

Follows the pattern in `sdlc_guideline_query_options`:

```rust
fn knowledge_ask_query_options(root: std::path::PathBuf) -> QueryOptions {
    let mut opts = sdlc_query_options(root, 10);
    // Remove sdlc MCP tools — this agent only reads knowledge, no state mutations.
    opts.allowed_tools = vec![
        "WebSearch".into(),
        "WebFetch".into(),
    ];
    opts.mcp_servers.clear();
    opts
}
```

No sdlc CLI tools, no file read/write tools — the knowledge context is injected into the prompt directly. This keeps the agent narrowly scoped and prevents unexpected side effects.

## Error Handling

| Scenario | Response |
|---|---|
| Empty question | `400 Bad Request` `{"error": "question must not be empty"}` |
| Knowledge base not initialized | `200` with answer `"Knowledge base is empty. Run \`sdlc knowledge librarian init\` first."` — no agent spawned |
| Agent already running | `409 Conflict` `{"error": "query already running"}` |
| Agent errors internally | `RunFinished {status: "failed"}` SSE + `KnowledgeQueryCompleted` with error answer |

## Testing Plan

Unit tests in `crates/sdlc-server/tests/integration.rs`:

- `POST /api/knowledge/ask` with empty question → 400.
- `POST /api/knowledge/ask` with valid question → 202, `run_id` present.
- Second concurrent `POST /api/knowledge/ask` → 409.

Unit tests in `crates/sdlc-cli/src/cmd/knowledge.rs` (or a companion test module):

- `build_ask_prompt` produces expected markdown structure.
- Gap JSON extraction regex matches known sentinel block.
- Citation regex extracts known `[CODE slug]` patterns correctly.

No new integration test fixtures needed — the server integration test can use an empty knowledge dir and assert the 200-with-empty-message path.

## Non-Goals

- No persistent query history storage — each run record already captures the prompt via `RunRecord.prompt`.
- No UI panel in this feature — the SSE event schema is defined so a follow-on feature can render it.
- No streaming of partial answer to CLI — CLI waits for `KnowledgeQueryCompleted` then prints in full.
