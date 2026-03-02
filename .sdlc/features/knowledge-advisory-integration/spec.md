# Spec: Knowledge-Advisory Integration

## Overview

Advisory runs currently operate without awareness of the project's knowledge base. This means the advisory agent may rediscover patterns, anti-patterns, and best practices that have already been captured as institutional knowledge. This feature injects relevant knowledge base entries into the advisory agent's context before each run, ensuring the agent can build on (and cite) existing knowledge rather than rediscovering it from scratch.

## Goal

When `POST /api/advisory/run` is called, the server queries the knowledge base for the top-N most relevant entries (by tag intersection + recency, N=10) and injects them into the advisory agent prompt as a "Project Knowledge" context block. The advisory output should cite relevant knowledge entries by slug when they inform findings.

Additionally, a new `GET /api/knowledge/relevant?tags=a,b&limit=10` endpoint is added to support this query pattern generically.

## Scope

### In Scope

1. **`GET /api/knowledge/relevant` endpoint** — query params: `tags` (comma-separated), `limit` (integer, default 10). Returns entries sorted by tag-overlap score descending, then `updated_at` descending. Only `published` entries are included.

2. **Advisory prompt injection** — before spawning the advisory agent, `start_advisory_run` calls `knowledge::list()` (or the new relevance function), filters to published entries, computes overlap against a fixed set of advisory-relevant tags (`advisory`, `architecture`, `pattern`, `anti-pattern`, `best-practice`, `refactor`, `testing`, `security`), selects top-10, and injects them into the prompt as a markdown table.

3. **Knowledge core function** — add `pub fn relevant_entries(root: &Path, tags: &[String], limit: usize) -> Result<Vec<KnowledgeEntry>>` to `sdlc-core/src/knowledge.rs`. Scores entries by tag intersection count; ties broken by `updated_at` descending. Filters to `KnowledgeStatus::Published` only.

4. **Prompt block format** — the injected block appears after the history context line and before the `## Steps` section:

```
## Project Knowledge

The following entries from the project knowledge base are relevant to this advisory run.
Cite the slug (e.g. `[kb: <slug>]`) in findings where this knowledge informed your analysis.

| Slug | Title | Summary |
|------|-------|---------|
| error-handling-patterns | Error Handling Patterns | ... |
| ...  | ...   | ...     |

If no entries are shown, the knowledge base is empty or no relevant entries were found.
```

5. **Graceful degradation** — if the knowledge base is absent or empty, the block is omitted from the prompt silently. No error is returned.

### Out of Scope

- Automatic tagging of advisory findings with knowledge slugs (advisory agent does this agentively via text)
- Bidirectional linking (updating KB entries when cited in findings)
- Frontend UI changes (the advisory panel is unchanged)
- Auto-seeding the knowledge base from advisory findings

## API

### GET /api/knowledge/relevant

**Query parameters:**
- `tags` — comma-separated list of tags (e.g. `tags=rust,async,error-handling`)
- `limit` — maximum results to return (default: 10, max: 50)

**Response:** Same shape as `GET /api/knowledge` list items:
```json
[
  {
    "slug": "error-handling-patterns",
    "title": "Error Handling Patterns",
    "code": "500.10",
    "status": "published",
    "summary": "...",
    "tags": ["rust", "error-handling", "pattern"],
    "created_at": "...",
    "updated_at": "..."
  }
]
```

**Scoring:** entries are scored by how many of the requested tags they share. Ties broken by `updated_at` descending. Only `published` entries are returned.

**Empty knowledge base:** returns `[]` (not an error).

## Implementation Plan

### 1. `sdlc-core/src/knowledge.rs`

Add `pub fn relevant_entries(root: &Path, tags: &[String], limit: usize) -> Result<Vec<KnowledgeEntry>>`:
- Call `list(root)?` to get all entries
- Filter to `KnowledgeStatus::Published`
- For each entry, compute score = number of `tags` that appear in `entry.tags`
- Sort by score descending, then `updated_at` descending
- Return the first `limit` entries

### 2. `crates/sdlc-server/src/routes/knowledge.rs`

Add handler `get_relevant_knowledge`:
- Extract `tags: Option<String>` and `limit: Option<usize>` from query params
- Parse comma-separated tags string into `Vec<String>`
- Call `spawn_blocking` with `knowledge::relevant_entries(&root, &tags, limit.unwrap_or(10).min(50))`
- Return same JSON shape as `list_knowledge`

### 3. `crates/sdlc-server/src/lib.rs`

Register the new route:
```rust
.route("/api/knowledge/relevant", get(routes::knowledge::get_relevant_knowledge))
```

Note: this route must appear **before** `/api/knowledge/{slug}` to avoid slug-capturing `"relevant"` as a slug.

### 4. `crates/sdlc-server/src/routes/advisory.rs`

In `start_advisory_run`, after building `history_context`:
- Call `spawn_blocking` to run `knowledge::relevant_entries(&root, &advisory_tags, 10)`
- `advisory_tags = ["advisory", "architecture", "pattern", "anti-pattern", "best-practice", "refactor", "testing", "security"]`
- If result is non-empty, build a markdown table string `knowledge_context`
- If result is empty or knowledge base absent, set `knowledge_context = ""`
- Inject into the prompt between the history context line and `## Steps`

## Acceptance Criteria

1. `GET /api/knowledge/relevant?tags=rust,async&limit=5` returns at most 5 published entries with the highest tag-overlap score. Draft entries are excluded.
2. When the knowledge base has published entries with matching tags, the advisory prompt contains a "## Project Knowledge" block with those entries in a markdown table.
3. When the knowledge base is empty or absent, no "## Project Knowledge" block appears in the prompt and the advisory run proceeds normally.
4. The `GET /api/knowledge/relevant` route does not shadow the `GET /api/knowledge/{slug}` route (i.e., `/api/knowledge/rust-patterns` still resolves to the slug handler).
5. All existing advisory and knowledge tests continue to pass.

## Non-Goals

- This feature does not change how advisory findings are stored or displayed.
- This feature does not require changes to `advisory.yaml` schema.
- This feature does not change the frontend.
