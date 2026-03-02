# Design: Knowledge-Advisory Integration

## Summary

This feature adds knowledge-base awareness to advisory runs via two focused changes:
1. A new `relevant_entries` function in `sdlc-core` that scores entries by tag overlap.
2. A new `GET /api/knowledge/relevant` HTTP endpoint exposing that function.
3. Injection of the top-10 relevant entries into the advisory agent prompt before the run begins.

No frontend changes. No schema changes. No new files — all changes are additions to existing files.

## Data Flow

```
POST /api/advisory/run
        │
        ▼
start_advisory_run()
        │
        ├─► load AdvisoryHistory (existing)
        │
        ├─► spawn_blocking {
        │       knowledge::relevant_entries(root, ADVISORY_TAGS, 10)
        │   }
        │       │
        │       ▼
        │   knowledge.rs: relevant_entries()
        │       - list() all entries
        │       - filter: status == Published
        │       - score: count of tags in ADVISORY_TAGS that entry.tags contains
        │       - sort: score desc, updated_at desc
        │       - return top-N
        │
        ├─► build knowledge_context markdown block
        │       (empty string if no relevant entries)
        │
        ├─► build full prompt (history_context + knowledge_context + ## Steps)
        │
        └─► spawn_agent_run(prompt, ...)
```

## Component Design

### `sdlc-core/src/knowledge.rs` — new function

```rust
/// Return the top `limit` published entries most relevant to `tags`.
///
/// Scoring: count of `tags` that appear in `entry.tags`.
/// Ties broken by `updated_at` descending (most recently updated first).
/// Only entries with `KnowledgeStatus::Published` are considered.
/// Returns `[]` if the knowledge directory is absent.
pub fn relevant_entries(
    root: &Path,
    tags: &[String],
    limit: usize,
) -> Result<Vec<KnowledgeEntry>> {
    let all = list(root)?;
    let mut scored: Vec<(usize, KnowledgeEntry)> = all
        .into_iter()
        .filter(|e| e.status == KnowledgeStatus::Published)
        .map(|e| {
            let score = tags.iter().filter(|t| e.tags.contains(t)).count();
            (score, e)
        })
        .filter(|(score, _)| *score > 0)  // only include entries with ≥1 matching tag
        .collect();

    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });

    Ok(scored.into_iter().take(limit).map(|(_, e)| e).collect())
}
```

Note: entries with zero tag overlap are excluded entirely. This ensures the context block is meaningful.

### `sdlc-server/src/routes/knowledge.rs` — new handler

```rust
#[derive(serde::Deserialize, Default)]
pub struct RelevantKnowledgeQuery {
    pub tags: Option<String>,   // comma-separated
    pub limit: Option<usize>,
}

pub async fn get_relevant_knowledge(
    State(app): State<AppState>,
    Query(params): Query<RelevantKnowledgeQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let tags: Vec<String> = params
        .tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let limit = params.limit.unwrap_or(10).min(50);

    let result = tokio::task::spawn_blocking(move || {
        let entries = sdlc_core::knowledge::relevant_entries(&root, &tags, limit)?;
        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| serde_json::json!({
                "slug": e.slug,
                "title": e.title,
                "code": e.code,
                "status": e.status.to_string(),
                "summary": e.summary,
                "tags": e.tags,
                "created_at": e.created_at,
                "updated_at": e.updated_at,
            }))
            .collect();
        Ok::<_, sdlc_core::SdlcError>(serde_json::json!(list))
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))??;

    Ok(Json(result))
}
```

### `sdlc-server/src/lib.rs` — route registration

The new route is registered **before** `/api/knowledge/{slug}` to prevent Axum from treating `"relevant"` as a slug:

```rust
.route("/api/knowledge/relevant", get(routes::knowledge::get_relevant_knowledge))
.route("/api/knowledge/{slug}", get(routes::knowledge::get_knowledge).put(...))
```

### `sdlc-server/src/routes/advisory.rs` — prompt injection

```rust
const ADVISORY_TAGS: &[&str] = &[
    "advisory", "architecture", "pattern", "anti-pattern",
    "best-practice", "refactor", "testing", "security",
];

// In start_advisory_run(), after building history_context:

let root_clone2 = app.root.clone();
let advisory_tags: Vec<String> = ADVISORY_TAGS.iter().map(|s| s.to_string()).collect();
let knowledge_context = tokio::task::spawn_blocking(move || {
    sdlc_core::knowledge::relevant_entries(&root_clone2, &advisory_tags, 10)
})
.await
.map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
.unwrap_or_default();  // silently ignore errors — advisory still runs

let kb_block = if knowledge_context.is_empty() {
    String::new()
} else {
    let rows: String = knowledge_context
        .iter()
        .map(|e| {
            let summary = e.summary.as_deref().unwrap_or("—");
            format!("| `{}` | {} | {} |\n", e.slug, e.title, summary)
        })
        .collect();
    format!(
        "\n## Project Knowledge\n\nThe following entries from the project knowledge base are \
         relevant to this advisory run.\nCite the slug (e.g. `[kb: <slug>]`) in findings \
         where this knowledge informed your analysis.\n\n\
         | Slug | Title | Summary |\n\
         |------|-------|---------|\n\
         {rows}"
    )
};

let prompt = format!(
    r#"You are an expert engineering advisor. ...

Context from previous runs: {history_context}
{kb_block}

## Steps
..."#
);
```

## Routing Order Rationale

Axum resolves routes in registration order for static vs. parameterized segments. `/api/knowledge/relevant` (static) must appear before `/api/knowledge/{slug}` (parameterized) or Axum will match the slug route first, treating `"relevant"` as a slug value and returning 404 from the load call.

## Error Handling

- If `relevant_entries` returns `Err` (e.g., malformed manifest), the error is silently discarded via `.unwrap_or_default()` and the prompt proceeds without the KB block. Advisory runs should never fail due to knowledge base issues.
- If `relevant_entries` returns `Ok([])` (empty KB or no matching tags), the KB block is omitted and the prompt proceeds normally.

## Testing

- Unit test `relevant_entries` in `knowledge.rs`: create published and draft entries with tags, verify only published entries with matching tags are returned, verify score ordering.
- Integration test `GET /api/knowledge/relevant?tags=rust&limit=2`: verify correct shape and that draft entries are excluded.
- Integration test `POST /api/advisory/run` with a seeded KB: verify the returned run ID is valid (the agent prompt injection is tested indirectly — the route must not error out).
