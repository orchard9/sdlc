# Tasks: Knowledge-Advisory Integration

## T1 — Add `relevant_entries` to `sdlc-core/src/knowledge.rs`

Add the following public function to `crates/sdlc-core/src/knowledge.rs`:

```rust
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
            let score = tags.iter().filter(|t| e.tags.contains(*t)).count();
            (score, e)
        })
        .filter(|(score, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
    });
    Ok(scored.into_iter().take(limit).map(|(_, e)| e).collect())
}
```

Add a unit test `relevant_entries_scores_by_tag_overlap` that:
- Creates 3 published entries with varying tag overlap against a query set
- Creates 1 draft entry with matching tags (must be excluded)
- Verifies ordering by score, then recency
- Verifies draft entries are excluded

## T2 — Add `GET /api/knowledge/relevant` route handler

In `crates/sdlc-server/src/routes/knowledge.rs`, add:

```rust
#[derive(serde::Deserialize, Default)]
pub struct RelevantKnowledgeQuery {
    pub tags: Option<String>,
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

## T3 — Register route in `lib.rs` before `{slug}` route

In `crates/sdlc-server/src/lib.rs`, add the route **before** the existing `GET /api/knowledge/{slug}` registration:

```rust
.route("/api/knowledge/relevant", get(routes::knowledge::get_relevant_knowledge))
```

It must appear before `.route("/api/knowledge/{slug}", ...)` to avoid Axum matching "relevant" as a slug.

## T4 — Inject knowledge context into advisory prompt

In `crates/sdlc-server/src/routes/advisory.rs`, modify `start_advisory_run`:

1. Define the advisory tag set as a constant:
```rust
const ADVISORY_TAGS: &[&str] = &[
    "advisory", "architecture", "pattern", "anti-pattern",
    "best-practice", "refactor", "testing", "security",
];
```

2. After `history_context` is built, query the knowledge base:
```rust
let root_kb = app.root.clone();
let advisory_tags: Vec<String> = ADVISORY_TAGS.iter().map(|s| s.to_string()).collect();
let kb_entries = tokio::task::spawn_blocking(move || {
    sdlc_core::knowledge::relevant_entries(&root_kb, &advisory_tags, 10)
})
.await
.map_err(|e| AppError(anyhow::anyhow!("task join error: {e}")))?
.unwrap_or_default();
```

3. Build the KB block:
```rust
let kb_block = if kb_entries.is_empty() {
    String::new()
} else {
    let rows: String = kb_entries
        .iter()
        .map(|e| {
            let summary = e.summary.as_deref().unwrap_or("—");
            format!("| `{}` | {} | {} |\n", e.slug, e.title, summary)
        })
        .collect();
    format!(
        "\n## Project Knowledge\n\n\
         The following entries from the project knowledge base are relevant to this advisory run.\n\
         Cite the slug (e.g. `[kb: <slug>]`) in findings where this knowledge informed your analysis.\n\n\
         | Slug | Title | Summary |\n\
         |------|-------|---------|\n\
         {rows}"
    )
};
```

4. Inject `{kb_block}` into the prompt string between `{history_context}` and `## Steps`.

## T5 — Integration test for `GET /api/knowledge/relevant`

In `crates/sdlc-server/tests/integration.rs`, add a test that:
- Creates a temp dir with published and draft knowledge entries (using `sdlc_core::knowledge::create` + `update`)
- Calls `GET /api/knowledge/relevant?tags=rust,async&limit=2`
- Asserts the response is a JSON array with at most 2 items
- Asserts all returned entries have `status: "published"`
- Asserts no draft entries are included

## T6 — Run `cargo test --all` and `cargo clippy`

Verify all tests pass and no clippy warnings are introduced. Fix any issues found.
