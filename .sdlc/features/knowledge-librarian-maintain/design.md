# Design: sdlc knowledge librarian run — maintenance pass + harvest hooks wired

## Overview

This feature adds a recurring maintenance loop to the knowledge base: a CLI subcommand that an agent can be invoked against, two server endpoints to trigger agent runs, SSE lifecycle events, and hook wiring so workspaces auto-harvest when completed.

The design follows the established project pattern: Rust holds data, agents hold logic. The Rust layer adds CLI surface and server endpoints; the maintenance decisions are made by an agent reading those endpoints.

---

## Component Design

### 1. CLI — `sdlc knowledge librarian run`

**New `Run` variant** added to `KnowledgeLibrarianSubcommand` in `crates/sdlc-cli/src/cmd/knowledge.rs`:

```rust
#[derive(Subcommand)]
pub enum KnowledgeLibrarianSubcommand {
    Init,
    Run {
        /// "maintain" (default) or "harvest"
        #[arg(long, default_value = "maintain")]
        mode: String,
        /// Workspace type for harvest mode: "investigation" or "ponder"
        #[arg(long)]
        r#type: Option<String>,
        /// Workspace slug for harvest mode
        #[arg(long)]
        slug: Option<String>,
    },
    Harvest {
        /// Workspace type: "investigation" or "ponder"
        #[arg(long)]
        r#type: String,
        /// Workspace slug
        #[arg(long)]
        slug: String,
    },
}
```

**`run_librarian` match arm for `Run { mode, .. }`:**

- If `mode == "maintain"`: print a human-readable prompt describing what the maintenance agent should do (the six checks), then exit 0. This is the "print instructions and exit" pattern — the agent receives this output and acts on it.
- If `mode == "harvest"` with `--type` and `--slug`: delegates to `librarian_harvest(root, type_, slug)` which calls `knowledge::librarian_harvest_workspace(root, workspace_type, slug)` (new core function).

**`run_librarian` match arm for `Harvest { type_, slug }`:**

Direct alias for single-workspace harvest. Calls the same `librarian_harvest_workspace` core function.

Output (both modes, non-JSON): human-readable harvest summary.
Output (JSON): `{ "type": "...", "slug": "...", "created": bool, "entry_slug": "..." }`.

### 2. sdlc-core — `knowledge::librarian_harvest_workspace`

New public function in `crates/sdlc-core/src/knowledge.rs`:

```rust
pub fn librarian_harvest_workspace(
    root: &Path,
    workspace_type: &str,  // "investigation" | "ponder"
    workspace_slug: &str,
) -> Result<HarvestResult>
```

Where `HarvestResult` is:

```rust
pub struct HarvestResult {
    pub entry_slug: String,
    pub created: bool,  // true = new entry, false = updated existing
}
```

Logic (mirrors what `librarian_init` does per-entry):

1. Load workspace manifest (investigation or ponder) to get `title`, `status`, `tags`.
2. Derive knowledge entry slug from `"<workspace_type>/<workspace_slug>"`.
3. Check if an entry already exists with `harvested_from == Some("<workspace_type>/<workspace_slug>")`.
4. If not found: create a new entry with `origin = Harvested`, `harvested_from = Some(...)`, `source = SourceType::Harvested`.
5. If found: call `knowledge::update` to refresh `updated_at`.
6. Append workspace session artifacts and scrapbook content to `content.md`.
7. Log a `MaintenanceAction { action_type: "harvest", slug: Some(entry_slug), detail: "harvested from <workspace_type>/<workspace_slug>" }` to maintenance log.
8. Return `HarvestResult`.

### 3. Server — `POST /api/knowledge/maintain`

New handler in `crates/sdlc-server/src/routes/knowledge.rs` (or factored into `runs.rs`):

```rust
pub async fn maintain_knowledge(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError>
```

Steps:
1. Build prompt describing maintenance agent task (the six checks, referencing `sdlc knowledge` CLI commands to read state and write updates).
2. Get `opts = sdlc_query_options(app.root.clone(), 50)`.
3. Emit `SseMessage::KnowledgeMaintenanceStarted` on `app.event_tx`.
4. Call `spawn_agent_run("knowledge:maintain", prompt, opts, &app, "knowledge_maintain", "Knowledge maintenance", Some(SseMessage::KnowledgeMaintenanceCompleted { actions_taken: 0 }))`.

The `actions_taken` in the completion event is set to `0` at spawn time (the agent logs actual counts to the maintenance log; a future feature can read and report them).

### 4. Server — `POST /api/knowledge/harvest`

```rust
#[derive(serde::Deserialize)]
pub struct HarvestWorkspaceBody {
    pub r#type: String,  // "investigation" | "ponder"
    pub slug: String,
}

pub async fn harvest_knowledge_workspace(
    State(app): State<AppState>,
    Json(body): Json<HarvestWorkspaceBody>,
) -> Result<Json<serde_json::Value>, AppError>
```

Steps:
1. Validate `type` is `"investigation"` or `"ponder"`.
2. Build prompt for single-workspace harvest agent run.
3. Run key: `format!("knowledge:harvest:{}", body.slug)`.
4. Call `spawn_agent_run(...)` with run type `"knowledge_harvest"`.
5. Return standard `spawn_agent_run` response.

### 5. SSE — New `SseMessage` Variants

Added to `crates/sdlc-server/src/state.rs`:

```rust
/// Knowledge base maintenance agent run started.
KnowledgeMaintenanceStarted,
/// Knowledge base maintenance agent run completed.
KnowledgeMaintenanceCompleted { actions_taken: usize },
```

Serialized in the `SseMessage` to JSON event payload following the existing pattern (`"type"` field + variant fields).

### 6. Hook Wiring — Auto-harvest on `--status complete`

**`crates/sdlc-cli/src/cmd/investigate.rs`** — in the `Update { slug, status, .. }` arm, after the successful `investigation::update_status` call, when `status == "complete"`:

```rust
if status.as_deref() == Some("complete") {
    let _ = std::process::Command::new("sdlc")
        .args(["knowledge", "librarian", "harvest", "--type", "investigation", "--slug", &slug])
        .status();
}
```

**`crates/sdlc-cli/src/cmd/ponder.rs`** — same pattern after `ponder::update_status` when `status == "complete"`:

```rust
if status.as_deref() == Some("complete") {
    let _ = std::process::Command::new("sdlc")
        .args(["knowledge", "librarian", "harvest", "--type", "ponder", "--slug", &slug])
        .status();
}
```

Errors from the subprocess are silently discarded (`.status()` result ignored). This is intentional — the harvest is best-effort and must not block the status update.

### 7. Route Registration

In `crates/sdlc-server/src/lib.rs`, add to the router:

```rust
.route("/api/knowledge/maintain", post(knowledge::maintain_knowledge))
.route("/api/knowledge/harvest", post(knowledge::harvest_knowledge_workspace))
```

These must be registered before the parameterized `/api/knowledge/:slug` route to avoid shadowing.

---

## Data Flow Diagram

```
[Agent]                      [CLI: sdlc]                    [sdlc-core]
  |                               |                               |
  |-- sdlc knowledge list ------> |-- knowledge::list() -------> |
  |<- entry list (JSON) ----------|<- Vec<KnowledgeEntry> -------|
  |                               |                               |
  |-- sdlc knowledge update ----> |-- knowledge::update() ------> |
  |   (--tag "url_404")           |   (appends staleness_flag)    |
  |                               |                               |
  |-- sdlc knowledge librarian -> |-- librarian_harvest_workspace()|
  |   harvest --type X --slug Y   |   → creates/updates entry    |
  |                               |   → logs MaintenanceAction   |
  |<- harvest summary ------------|<- HarvestResult --------------|


[Frontend]            [Server: /api/knowledge/maintain]
  |                               |
  |-- POST /api/knowledge/maintain|
  |<- { run_id, status: "started"}|
  |                               |
  |<-- SSE: KnowledgeMaintenanceStarted
  |<-- SSE: RunStarted { id, key }
  |                               |
  |      (agent runs, takes actions)
  |                               |
  |<-- SSE: RunFinished { id, status }
  |<-- SSE: KnowledgeMaintenanceCompleted { actions_taken }
```

---

## Integration Points

| Existing Function | How Used |
|---|---|
| `knowledge::librarian_init` | Reference for harvest logic — `librarian_harvest_workspace` extracts and reuses its per-entry logic |
| `knowledge::log_maintenance_action` | Called by `librarian_harvest_workspace` to record each harvest |
| `runs::spawn_agent_run` | Used by both `maintain_knowledge` and `harvest_knowledge_workspace` |
| `runs::sdlc_query_options` | Default tool set for maintenance agent runs |
| `SseMessage` enum | Extended with two new variants |

---

## Non-Goals

- No frontend UI changes (future feature).
- No merging of detected duplicates (flag only).
- No changes to `librarian_init` behavior.
- Maintenance agent prompt content is a skill instruction, not Rust logic.
