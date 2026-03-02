# Spec: knowledge-query-mode — sdlc knowledge ask (librarian query mode)

## Overview

Add a `sdlc knowledge ask '<question>'` CLI subcommand and a `POST /api/knowledge/ask` server endpoint that invoke a librarian agent via `spawn_agent_run`. The agent receives the user's question plus the full knowledge base catalog and entry summaries as context, synthesizes an answer, cites relevant entry codes, and — when a gap is detected — suggests running `sdlc knowledge research <topic>`.

This is the first "read-side" intelligence layer for the knowledge base: a fast, conversational interface into the accumulated knowledge without requiring the user to know which entries to query.

## User Stories

- As a developer, I can ask `sdlc knowledge ask "How does our trunk-based branching strategy work?"` and get a synthesized answer drawn from knowledge base entries, with cited entry codes I can look up.
- As a developer, when the knowledge base does not have enough information, I see a clear gap message that suggests `sdlc knowledge research <topic>` to fill it.
- As a user of the web UI, I can type a question into the Knowledge page and see a streaming answer with citations, rendered inline.

## CLI Specification

### Subcommand

```
sdlc knowledge ask '<question>'
```

Added to `KnowledgeSubcommand` enum:

```rust
/// Ask the knowledge librarian a question (spawns agent via sdlc-server)
Ask {
    /// The question to ask (free-form natural language)
    question: String,
},
```

### CLI Behavior

The CLI `ask` subcommand makes an HTTP request to the local `sdlc-server` at `POST /api/knowledge/ask` with `{"question": "<question>"}`. It then streams the agent run SSE to stdout until `KnowledgeQueryCompleted` is received. If the server is not running, it prints a clear error: `sdlc server not running — start it with \`sdlc ui\``.

The final answer is printed to stdout. Citations are formatted as:
```
[100.20] Trunk-based development strategy (trunk-based-dev)
[200.10] Git branching conventions (git-branch-policy)
```

If a gap is detected, append:
```
Knowledge gap detected. Run: sdlc knowledge research <suggested-topic>
```

### JSON output (`--json`)

```json
{
  "answer": "...",
  "cited_entries": [
    {"slug": "trunk-based-dev", "code": "100.20", "title": "Trunk-based development strategy"},
    {"slug": "git-branch-policy", "code": "200.10", "title": "Git branching conventions"}
  ],
  "gap_detected": false,
  "gap_suggestion": null
}
```

## Server API Specification

### `POST /api/knowledge/ask`

**Request body:**
```json
{ "question": "How does our trunk-based branching strategy work?" }
```

**Behavior:**
1. Validates `question` is non-empty (400 if blank).
2. Loads the full knowledge base: catalog + all entry summaries + entry codes.
3. Builds a prompt containing the catalog taxonomy, entry list (slug, code, title, summary), and the user's question.
4. Calls `spawn_agent_run` with key `"knowledge:ask"`, `run_type = "knowledge"`, label `"Knowledge query"`, `max_turns = 10`, web search tools allowed, and `completion_event = Some(SseMessage::KnowledgeQueryCompleted { question, answer, cited_entries })`.
5. Returns `202 Accepted` immediately with `{"run_id": "<id>"}`.

**Concurrent protection:** if a `knowledge:ask` run is already active, returns `409 Conflict` with `{"error": "query already running"}`.

### SSE Events

Two new `SseMessage` variants added to `crates/sdlc-server/src/state.rs`:

```rust
/// A knowledge query agent run has started.
KnowledgeQueryStarted { question: String },
/// A knowledge query agent run completed — answer synthesized and cited.
KnowledgeQueryCompleted {
    answer: String,
    cited_entries: Vec<CitedEntry>,
    gap_detected: bool,
    gap_suggestion: Option<String>,
},
```

Where `CitedEntry` is a plain data struct:

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CitedEntry {
    pub slug: String,
    pub code: String,
    pub title: String,
}
```

Both events are serialized and emitted on the SSE channel in the same format as all other `SseMessage` variants.

## Agent Prompt

The agent is a librarian. The prompt structure:

```
You are the knowledge librarian for this project. Your job is to answer questions
using only the knowledge base entries provided below.

## Knowledge Base Catalog
<catalog taxonomy tree here>

## Knowledge Base Entries
<for each entry: slug, code, title, summary>

## Question
<user question>

## Instructions
- Answer the question using only the knowledge in the entries above.
- Cite every entry you draw from using its code and slug in square brackets: [100.20 trunk-based-dev].
- If the knowledge base does not contain enough information to answer the question fully,
  say so clearly and output a JSON block at the end:
  {"gap": true, "suggestion": "<research topic>"}
- If you use web search (via tools), note the source.
- Output max_turns is 10 — be direct.
```

The agent is allowed to use web search tools so it can supplement thin entries with recent public context (the user has opted in by querying). The gap JSON block is parsed by the server completion handler to populate `gap_detected` and `gap_suggestion` in the SSE event.

## QueryOptions

```rust
QueryOptions {
    max_turns: Some(10),
    allowed_tools: vec![
        "WebSearch".to_string(),
        "WebFetch".to_string(),
    ],
    ..sdlc_query_options(&root)
}
```

No Playwright MCP tools; no sdlc CLI tools. Web search only.

## Data Flow

```
CLI: sdlc knowledge ask '<question>'
  → POST /api/knowledge/ask {question}
  → spawn_agent_run("knowledge:ask", prompt, opts, ...)
  → [SSE] KnowledgeQueryStarted {question}
  → [agent streams turns via run SSE channel]
  → completion handler parses agent output for gap block
  → [SSE] KnowledgeQueryCompleted {answer, cited_entries, gap_detected, gap_suggestion}
  → CLI receives KnowledgeQueryCompleted, prints answer + citations
```

## Out of Scope

- Persistent query history — queries are ephemeral (run record captures the prompt).
- Interactive multi-turn conversation — each `ask` is a fresh single-question run.
- Frontend UI beyond wiring the SSE event type — no new pages or panels in this feature. The SSE event payload is defined so a future UI feature can consume it.

## Acceptance Criteria

1. `sdlc knowledge ask "What is our deployment process?"` returns an answer with at least one cited entry if relevant entries exist.
2. If no entries match, the response includes `gap_detected: true` and a suggested research topic.
3. `POST /api/knowledge/ask` returns `202` with a `run_id`.
4. `KnowledgeQueryStarted` and `KnowledgeQueryCompleted` SSE events are emitted on the `/api/events` stream.
5. `--json` flag on the CLI subcommand outputs valid JSON matching the specified schema.
6. Concurrent `ask` calls return `409 Conflict`.
7. `cargo test --all` passes with new unit tests for the prompt builder and the gap JSON parser.
