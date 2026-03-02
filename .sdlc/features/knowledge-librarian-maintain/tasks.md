# Tasks: sdlc knowledge librarian run — maintenance pass + harvest hooks wired

## T1 — Add `librarian_harvest_workspace` to sdlc-core

File: `crates/sdlc-core/src/knowledge.rs`

- Add `HarvestResult { entry_slug: String, created: bool }` struct (pub, derive Serialize/Deserialize).
- Implement `pub fn librarian_harvest_workspace(root: &Path, workspace_type: &str, workspace_slug: &str) -> Result<HarvestResult>`:
  - Load workspace manifest (investigation or ponder) via existing workspace/investigation/ponder load functions.
  - Derive entry slug from `format!("{}-{}", workspace_type, workspace_slug)` (slug-safe).
  - Check if an entry with `harvested_from == Some("<workspace_type>/<workspace_slug>")` already exists via `list()`.
  - If not found: call `create()` with `origin = OriginKind::Harvested`; set `harvested_from`; set source `SourceType::Harvested`.
  - If found: call `update()` to bump `updated_at`.
  - Append workspace session content and scrapbook artifacts to `content.md` via `append_content()`.
  - Call `log_maintenance_action()` with `action_type = "harvest"`.
  - Return `HarvestResult`.
- Add unit test: create a minimal investigation workspace in a TempDir, call `librarian_harvest_workspace`, assert entry created with correct `harvested_from`.

## T2 — Extend `KnowledgeLibrarianSubcommand` with `Run` and `Harvest` variants

File: `crates/sdlc-cli/src/cmd/knowledge.rs`

- Add `Run { mode: String, r#type: Option<String>, slug: Option<String> }` and `Harvest { r#type: String, slug: String }` variants to `KnowledgeLibrarianSubcommand`.
- Implement `Run` arm in `run_librarian`:
  - `mode == "maintain"`: print a fixed maintenance prompt describing the six checks (URL health, code ref health, duplication, catalog fitness, cross-ref suggestions, harvest pending). Exit 0.
  - `mode == "harvest"`: validate that `--type` and `--slug` are both present; call `knowledge::librarian_harvest_workspace(root, type_, slug)`; print summary or JSON.
- Implement `Harvest` arm: directly call `knowledge::librarian_harvest_workspace(root, &type_, &slug)`; print summary or JSON.
- JSON output shape: `{ "type": "...", "slug": "...", "created": bool, "entry_slug": "..." }`.

## T3 — Add `KnowledgeMaintenanceStarted` and `KnowledgeMaintenanceCompleted` to `SseMessage`

File: `crates/sdlc-server/src/state.rs`

- Add two new variants to `SseMessage`:
  ```rust
  KnowledgeMaintenanceStarted,
  KnowledgeMaintenanceCompleted { actions_taken: usize },
  ```
- Update the `SseMessage` → JSON serialization match arm (wherever the enum is serialized to SSE event data) to include these two new variants with `"type"` field.

## T4 — Add `POST /api/knowledge/maintain` and `POST /api/knowledge/harvest` server handlers

Files: `crates/sdlc-server/src/routes/knowledge.rs`, `crates/sdlc-server/src/lib.rs`

- In `knowledge.rs`, add:
  - `pub async fn maintain_knowledge(State(app): State<AppState>) -> Result<Json<serde_json::Value>, AppError>`:
    - Emit `SseMessage::KnowledgeMaintenanceStarted` on `app.event_tx`.
    - Build maintenance prompt string (the six-check instruction set for the agent).
    - Call `sdlc_query_options(app.root.clone(), 50)`.
    - Call `spawn_agent_run("knowledge:maintain", prompt, opts, &app, "knowledge_maintain", "Knowledge maintenance", Some(SseMessage::KnowledgeMaintenanceCompleted { actions_taken: 0 }))`.
  - `HarvestWorkspaceBody { r#type: String, slug: String }` struct.
  - `pub async fn harvest_knowledge_workspace(State(app): State<AppState>, Json(body): Json<HarvestWorkspaceBody>) -> Result<Json<serde_json::Value>, AppError>`:
    - Validate `body.type` is `"investigation"` or `"ponder"`.
    - Build harvest prompt.
    - Run key: `format!("knowledge:harvest:{}", body.slug)`.
    - Call `spawn_agent_run(...)` with run type `"knowledge_harvest"`.
- In `lib.rs`, register routes **before** the parameterized `/api/knowledge/:slug` route:
  ```
  POST /api/knowledge/maintain
  POST /api/knowledge/harvest
  ```
- Add `use super::runs::{sdlc_query_options, spawn_agent_run};` import to `knowledge.rs`.

## T5 — Wire auto-harvest hook in `sdlc investigate update`

File: `crates/sdlc-cli/src/cmd/investigate.rs`

- In the `Update { slug, status, .. }` handler, after a successful status write when `status == "complete"`:
  ```rust
  let _ = std::process::Command::new("sdlc")
      .args(["knowledge", "librarian", "harvest",
             "--type", "investigation", "--slug", &slug])
      .status();
  ```
- Error is silently discarded — the subprocess call is best-effort.

## T6 — Wire auto-harvest hook in `sdlc ponder update`

File: `crates/sdlc-cli/src/cmd/ponder.rs`

- In the `Update { slug, status, .. }` handler, after a successful status write when `status == "complete"`:
  ```rust
  let _ = std::process::Command::new("sdlc")
      .args(["knowledge", "librarian", "harvest",
             "--type", "ponder", "--slug", &slug])
      .status();
  ```
- Error is silently discarded.

## T7 — Tests and lint clean

- `SDLC_NO_NPM=1 cargo test --all` passes with new unit tests from T1.
- `cargo clippy --all -- -D warnings` clean.
- Verify `sdlc knowledge librarian run` exits 0 and prints expected output.
- Verify `sdlc knowledge librarian harvest --type investigation --slug <test-slug>` exits 0.
