# Spec: knowledge-research-mode

## Overview

Add an agent-driven research mode to the knowledge base. When invoked, an agent
autonomously researches a topic via web search and local file scan, synthesizes
findings into a knowledge entry, and logs the research session using the
`workspace.rs` session protocol.

This feature extends the existing knowledge base infrastructure (knowledge.rs,
knowledge routes, `sdlc knowledge` CLI) with a `research` subcommand and a
corresponding server endpoint that uses `spawn_agent_run`.

---

## Problem Statement

Currently, knowledge entries are created and populated manually or by the
librarian-init harvest. There is no way to kick off an autonomous agent run that
researches a topic end-to-end and populates an entry with synthesized findings.
Users who want to add a knowledge entry on a new topic must either write content
by hand or supply a URL. Neither captures deep multi-source research.

---

## Solution

### CLI: `sdlc knowledge research '<topic>' [--code '100.20']`

- A new `Research` variant is added to `KnowledgeSubcommand` in
  `crates/sdlc-cli/src/cmd/knowledge.rs`.
- If no entry with a derived slug exists yet, it is created first
  (`knowledge::create`).
- The CLI then calls `POST /api/knowledge/:slug/research` against the running
  server (using `ureq` like the existing `from_url` fetch), which starts the
  agent run.
- `--code` is optional; defaults to `"uncategorized"`.
- The command emits the slug and a message: "Research started for '<slug>'. Watch
  progress in the UI or via `sdlc knowledge session list <slug>`."
- A `--json` flag returns `{ "slug": "...", "started": true }`.

### Server: `POST /api/knowledge/:slug/research`

Located in `crates/sdlc-server/src/routes/knowledge.rs`.

**Request body** (JSON):
```json
{ "topic": "optional override topic string" }
```

**Behaviour:**
1. Load (or create) the knowledge entry for `slug`.
2. Build a research prompt from the entry title + topic.
3. Call `spawn_agent_run` with:
   - key: `"knowledge:{slug}"`
   - max_turns: 20
   - allowed tools: Read, Write, Edit, Glob, Grep, Bash, WebSearch
   - completion_event: `SseMessage::KnowledgeResearchCompleted { slug }`
4. Return `202 Accepted` with `{ "slug": "...", "started": true }`.

The agent prompt instructs it to:
- Search the web for the topic (using WebSearch tool).
- Scan local project files relevant to the topic (using Grep/Read).
- Synthesize findings into a concise markdown document.
- Write findings to `.sdlc/knowledge/<slug>/content.md` (via Write tool).
- Update the entry summary via `sdlc knowledge update <slug> --summary "..."`.
- Log the session via the two-step protocol:
  1. Write session log to `/tmp/knowledge-session-<slug>.md`
  2. `sdlc knowledge session log <slug> --file /tmp/knowledge-session-<slug>.md`

### SSE Events

Two new variants added to `SseMessage` in `crates/sdlc-server/src/state.rs`:

```rust
KnowledgeResearchStarted { slug: String },
KnowledgeResearchCompleted { slug: String },
```

`KnowledgeResearchStarted` is emitted immediately when `spawn_agent_run` is
called (before the async task starts). `KnowledgeResearchCompleted` is passed as
the `completion_event` to `spawn_agent_run`.

### `SseMessage` serialization

The existing `to_sse_event()` method on `SseMessage` must be extended for the
two new variants. They serialize under the existing `"knowledge"` event channel:

```
event: knowledge
data: {"type":"KnowledgeResearchStarted","slug":"..."}

event: knowledge
data: {"type":"KnowledgeResearchCompleted","slug":"..."}
```

---

## Data Contracts

No new data structures in `sdlc-core`. Research sessions are stored using the
existing `knowledge::log_session` function — same format as manually logged
sessions. The `OriginKind` on the entry is updated to `Research` (new variant,
see below).

### New `OriginKind` variant: `Research`

Add `Research` to the `OriginKind` enum in `crates/sdlc-core/src/knowledge.rs`.
Serialized as `"research"`. The agent sets this after completing the run via
`sdlc knowledge update <slug>` — or the server handler sets it synchronously
before spawning the agent.

---

## Scope Boundaries

**In scope:**
- `Research` CLI subcommand
- `POST /api/knowledge/:slug/research` server handler
- Two new `SseMessage` variants (`KnowledgeResearchStarted`,
  `KnowledgeResearchCompleted`)
- New `OriginKind::Research` variant in `sdlc-core`
- Agent prompt logic embedded in the server handler (string constant)

**Out of scope:**
- Frontend UI changes (research can be triggered from CLI; UI reads SSE events
  and refreshes the knowledge list automatically via the existing file-watcher)
- New `sdlc-core` logic beyond the `OriginKind` variant
- Scheduling or periodic re-research
- Progress streaming to CLI (the server runs async; CLI fires and returns)

---

## Acceptance Criteria

1. `sdlc knowledge research 'rust async runtimes' --code '200.10'` creates an
   entry, starts the agent run via the server, and returns without blocking.
2. The server `POST /api/knowledge/:slug/research` returns 202 within 500ms.
3. After the agent completes, `sdlc knowledge session list <slug>` shows at
   least one session.
4. `sdlc knowledge show <slug>` shows a non-empty content section.
5. SSE clients receive `KnowledgeResearchStarted` and
   `KnowledgeResearchCompleted` events on the `knowledge` channel.
6. `cargo test --all` passes.
7. `cargo clippy --all -- -D warnings` passes.
