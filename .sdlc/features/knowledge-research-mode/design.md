# Design: knowledge-research-mode

## Architecture Overview

This feature adds an agent-driven research mode as a thin extension over the
existing knowledge-base and `spawn_agent_run` infrastructure. No new core data
structures are introduced; the only new data field is `OriginKind::Research`.

```
┌─────────────────────────────────────────────────────────┐
│  User                                                   │
│  $ sdlc knowledge research 'rust async runtimes'        │
└──────────────────────┬──────────────────────────────────┘
                       │ HTTP POST /api/knowledge/:slug/research
                       ▼
┌─────────────────────────────────────────────────────────┐
│  sdlc-server  (knowledge.rs route)                      │
│                                                         │
│  1. Ensure entry exists (create if not)                 │
│  2. Set entry.origin = Research                         │
│  3. Emit KnowledgeResearchStarted SSE                   │
│  4. spawn_agent_run("knowledge:{slug}", prompt, ...)    │
│     → completion_event: KnowledgeResearchCompleted      │
│  5. Return 202 Accepted                                 │
└─────────────────────────┬───────────────────────────────┘
                          │ async (Tokio task)
                          ▼
┌─────────────────────────────────────────────────────────┐
│  Claude Agent (max_turns=20)                            │
│                                                         │
│  Tools: WebSearch, Read, Write, Edit, Glob, Grep, Bash  │
│                                                         │
│  1. WebSearch: gather multi-source information          │
│  2. Grep/Read: scan local project files                 │
│  3. Synthesize → write /tmp/knowledge-research-<slug>.md│
│  4. Write → .sdlc/knowledge/<slug>/content.md           │
│  5. Bash: sdlc knowledge update <slug> --summary "..."  │
│  6. Write → /tmp/knowledge-session-<slug>.md            │
│  7. Bash: sdlc knowledge session log <slug>             │
│            --file /tmp/knowledge-session-<slug>.md      │
└─────────────────────────────────────────────────────────┘
```

---

## Component Changes

### 1. `crates/sdlc-core/src/knowledge.rs`

**Add `OriginKind::Research` variant:**

```rust
pub enum OriginKind {
    Manual,
    Web,
    LocalFile,
    Workspace,
    Research,   // ← new
}
```

Serialized as `"research"`. `Display` and `FromStr` updated accordingly.

### 2. `crates/sdlc-server/src/state.rs`

**Add two SSE variants:**

```rust
pub enum SseMessage {
    // ... existing ...
    KnowledgeResearchStarted { slug: String },
    KnowledgeResearchCompleted { slug: String },
}
```

**Extend `to_sse_event()`:**

```rust
SseMessage::KnowledgeResearchStarted { slug } => Event::default()
    .event("knowledge")
    .data(serde_json::json!({
        "type": "KnowledgeResearchStarted",
        "slug": slug
    }).to_string()),

SseMessage::KnowledgeResearchCompleted { slug } => Event::default()
    .event("knowledge")
    .data(serde_json::json!({
        "type": "KnowledgeResearchCompleted",
        "slug": slug
    }).to_string()),
```

### 3. `crates/sdlc-server/src/routes/knowledge.rs`

**Add `POST /api/knowledge/:slug/research` handler:**

```rust
#[derive(serde::Deserialize, Default)]
pub struct ResearchKnowledgeBody {
    pub topic: Option<String>,
}

pub async fn research_knowledge(
    State(app): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<ResearchKnowledgeBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_slug(&slug)?;
    let root = app.root.clone();
    let slug_clone = slug.clone();

    // Ensure entry exists; create it if not
    let title = {
        let r = root.clone();
        let s = slug.clone();
        tokio::task::spawn_blocking(move || {
            match sdlc_core::knowledge::load(&r, &s) {
                Ok(e) => Ok(e.title),
                Err(_) => {
                    let entry = sdlc_core::knowledge::create(&r, &s, &s, "uncategorized")?;
                    // Set origin = Research
                    let mut e2 = entry;
                    e2.origin = sdlc_core::knowledge::OriginKind::Research;
                    sdlc_core::knowledge::save(&r, &e2)?;
                    Ok(e2.title)
                }
            }
        }).await.map_err(|e| AppError(anyhow::anyhow!("{e}")))??
    };

    let topic = body.topic.unwrap_or_else(|| title.clone());
    let prompt = build_research_prompt(&slug_clone, &title, &topic, &root);

    // Emit started event immediately
    let _ = app.event_tx.send(SseMessage::KnowledgeResearchStarted {
        slug: slug_clone.clone(),
    });

    let opts = sdlc_query_options(&app);
    // Allow WebSearch
    // (opts already has Bash, Read, Write, Edit, Glob, Grep from sdlc_query_options)

    spawn_agent_run(
        format!("knowledge:{slug_clone}"),
        prompt,
        opts,
        &app,
        "knowledge_research",
        &format!("Research: {title}"),
        Some(SseMessage::KnowledgeResearchCompleted { slug: slug_clone }),
    ).await
}
```

**Prompt constant:**

```rust
fn build_research_prompt(slug: &str, title: &str, topic: &str, root: &Path) -> String {
    format!(
        r#"You are a research agent tasked with building a comprehensive knowledge entry.

Topic: {topic}
Knowledge entry slug: {slug}
Knowledge entry title: {title}
Project root: {root}

## Steps

1. Use WebSearch to gather information about the topic from multiple sources.
2. Use Grep and Read to scan local project files for relevant existing knowledge.
3. Synthesize all findings into a comprehensive markdown document.
4. Write the synthesized content to `.sdlc/knowledge/{slug}/content.md`.
5. Run: `sdlc knowledge update {slug} --summary "<one-sentence summary>"`
6. Log your research session:
   a. Write a session summary to `/tmp/knowledge-session-{slug}.md`
   b. Run: `sdlc knowledge session log {slug} --file /tmp/knowledge-session-{slug}.md`

## Quality bar
- Content must be factual and well-organized with headers.
- Include source URLs as markdown links.
- Session log must include: what you searched, what you found, what you synthesized.
"#,
        root = root.display()
    )
}
```

### 4. `crates/sdlc-server/src/lib.rs`

Register the new route in the router:

```rust
.route("/api/knowledge/:slug/research", post(knowledge::research_knowledge))
```

### 5. `crates/sdlc-cli/src/cmd/knowledge.rs`

**Add `Research` subcommand variant:**

```rust
/// Spawn an agent research run for a topic
Research {
    /// The topic or question to research
    topic: String,
    /// Optional classification code (e.g. 100.20)
    #[arg(long)]
    code: Option<String>,
},
```

**Handler:**

```rust
KnowledgeSubcommand::Research { topic, code } => {
    research(root, &topic, code.as_deref(), json)
}
```

```rust
fn research(root: &Path, topic: &str, code: Option<&str>, json: bool) -> anyhow::Result<()> {
    let slug = slugify_title(topic);
    // Ensure entry exists locally before calling server
    if knowledge::load(root, &slug).is_err() {
        knowledge::create(root, &slug, topic, code.unwrap_or("uncategorized"))
            .with_context(|| format!("failed to create entry '{slug}'"))?;
    }

    // Call running server
    let port = read_server_port(root)?;
    let url = format!("http://localhost:{port}/api/knowledge/{slug}/research");
    let body = serde_json::json!({ "topic": topic });
    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .context("failed to contact sdlc server — is `sdlc ui` running?")?;

    if resp.status() == 202 {
        if json {
            print_json(&serde_json::json!({ "slug": slug, "started": true }))?;
        } else {
            println!("Research started for '{slug}'.");
            println!("Watch progress in the UI or via: sdlc knowledge session list {slug}");
        }
    } else {
        anyhow::bail!("server returned {}: {}", resp.status(), resp.into_string()?);
    }
    Ok(())
}
```

`read_server_port` reads the port from `.sdlc/server.port` (same file used by
other CLI-to-server calls in the codebase — check `orchestrate.rs` for the
pattern).

---

## File Map

| File | Change |
|---|---|
| `crates/sdlc-core/src/knowledge.rs` | Add `OriginKind::Research` variant + Display/FromStr |
| `crates/sdlc-server/src/state.rs` | Add `KnowledgeResearchStarted`, `KnowledgeResearchCompleted` SSE variants + serialization |
| `crates/sdlc-server/src/routes/knowledge.rs` | Add `research_knowledge` handler + `build_research_prompt` |
| `crates/sdlc-server/src/lib.rs` | Register `/api/knowledge/:slug/research` route |
| `crates/sdlc-cli/src/cmd/knowledge.rs` | Add `Research` subcommand + `research()` handler |
| `crates/sdlc-server/tests/integration.rs` | Integration test for research endpoint |

---

## No Frontend Changes Required

The frontend already polls `/api/knowledge` and refreshes on `knowledge` SSE
events via the existing file watcher. `KnowledgeResearchCompleted` fires on the
`knowledge` channel, so the knowledge list in the UI auto-refreshes with no code
changes.

---

## Error Handling

- If the server is not running when CLI calls `POST /api/knowledge/:slug/research`,
  the CLI prints a clear error: "failed to contact sdlc server — is `sdlc ui` running?"
- If the entry slug already has an active research run
  (`agent_runs.contains("knowledge:{slug}")`), the server returns `409 Conflict`
  with `{ "error": "research already running for this entry" }`.
- Agent errors are captured in the RunRecord (same as all `spawn_agent_run`
  paths) and visible in the UI run history.
