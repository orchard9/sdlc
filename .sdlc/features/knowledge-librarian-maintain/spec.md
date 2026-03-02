# Spec: sdlc knowledge librarian run — maintenance pass + harvest hooks wired

## Problem

The knowledge base can become stale, fragmented, and disconnected from active workspaces over time. There is currently no automated way to:

1. Detect and flag dead web links (404s) in entries with web sources.
2. Detect code references that no longer exist in the codebase.
3. Identify duplicate entries (same tag set + code prefix + similar title).
4. Suggest catalog restructuring when a class grows too large.
5. Suggest cross-references between entries that share tags but don't reference each other.
6. Harvest completed investigation/ponder workspaces into the knowledge base automatically.

The existing `sdlc knowledge librarian init` only runs once for bootstrapping. There is no recurring maintenance command, no server-side endpoint to trigger a maintenance agent, and no hook to auto-harvest workspaces when they are completed.

## Proposed Solution

### 1. CLI: `sdlc knowledge librarian run [--mode maintain|harvest]`

Add a `Run` variant to `KnowledgeLibrarianSubcommand` with an optional `--mode` flag defaulting to `maintain`.

**Maintenance mode** (`--mode maintain`, the default):

The CLI subcommand itself does **not** perform the maintenance logic. It prints a prompt for the agent and exits. The agent (invoked separately via `sdlc agent run`) executes the six maintenance checks by reading knowledge entries, fetching URLs, grepping codebase symbols, and calling `sdlc knowledge update` / `sdlc knowledge catalog add` to apply changes. All actions are appended to `.sdlc/knowledge/maintenance-log.yaml` via the existing `MaintenanceAction` / `MaintenanceLog` data structs in `sdlc-core`. Git commits with `librarian:` prefix are made by the agent.

The six maintenance checks (agent executes, not Rust):

1. **URL health** — for each entry with a `SourceType::Web` source, fetch the URL; if HTTP status is 4xx/5xx, add `"url_404"` to `entry.staleness_flags` via `sdlc knowledge update`.
2. **Code ref health** — for each entry where `origin == Research` or `tags` contains `"code-ref"`, attempt to grep the codebase for the referenced symbol (stored in the entry content or tags); if not found, add `"code_ref_gone"` to `staleness_flags`.
3. **Duplication** — entries sharing all of: same code prefix (first segment), same tag set, and similar title (edit distance < 20% of title length) are flagged with `"duplicate_candidate"` in `staleness_flags`.
4. **Catalog fitness** — for each catalog class with >10 entries, log a `catalog_update` MaintenanceAction suggesting subdivision. For empty classes, log a `catalog_update` action suggesting removal.
5. **Cross-ref suggestions** — entries sharing 2+ tags that do not reference each other are updated with mutual `related` entries via `sdlc knowledge update --related`.
6. **Harvest pending** — list all investigations and ponders; for each with `status == complete` that has no corresponding knowledge entry in `harvested_from`, trigger `sdlc knowledge librarian harvest --type <investigation|ponder> --slug <slug>`.

**Harvest mode** (`--mode harvest --type <investigation|ponder> --slug <slug>`):

Triggers a single-workspace harvest: creates or updates a knowledge entry from the given workspace's session artifacts, scrapbook, and output artifacts. This is the same logic as `librarian_init` single-entry harvesting, but callable incrementally.

### 2. Server: `POST /api/knowledge/maintain`

New endpoint in `crates/sdlc-server/src/routes/knowledge.rs` (or a new `runs.rs` handler) that spawns a maintenance agent run via `spawn_agent_run`. The run key is `"knowledge:maintain"`.

Response: `{ "run_id": "...", "status": "started" }` (standard `spawn_agent_run` response).

SSE events emitted:
- `KnowledgeMaintenanceStarted` — when the run begins.
- `KnowledgeMaintenanceCompleted { actions_taken: usize }` — when the run finishes successfully.

Both variants are added to `SseMessage` in `crates/sdlc-server/src/state.rs`.

### 3. Server: `POST /api/knowledge/harvest`

New endpoint accepting `{ "type": "investigation" | "ponder", "slug": "<slug>" }`. Spawns a harvest agent run for the specific workspace. Run key: `"knowledge:harvest:<slug>"`. No new SSE variants needed — uses the standard `RunStarted` / `RunFinished` events.

### 4. Hook wiring: auto-harvest on workspace completion

Modify `sdlc investigate update --status complete` and `sdlc ponder update --status complete` CLI paths to call `sdlc knowledge librarian harvest --type <investigation|ponder> --slug <slug>` after the status write succeeds (best-effort, failure does not abort the update).

This wiring is done inside the CLI command handlers:
- `crates/sdlc-cli/src/cmd/investigate.rs` — `update` subcommand, after `investigation::update_status`.
- `crates/sdlc-cli/src/cmd/ponder.rs` — `update` subcommand, after `ponder::update_status`.

The subprocess call is `std::process::Command::new("sdlc").args(["knowledge", "librarian", "harvest", "--type", type, "--slug", slug])`. Errors are printed as warnings, not returned.

## Data Layer Changes (sdlc-core)

No new structs required. The existing `MaintenanceAction`, `MaintenanceLog`, `KnowledgeEntry.staleness_flags`, and `KnowledgeEntry.related` are sufficient.

`knowledge::log_maintenance_action(root, action)` (already in `knowledge.rs`) is used by the agent to record each action via `sdlc knowledge` CLI calls. No new `sdlc-core` functions are needed.

## File Changes Summary

| File | Change |
|---|---|
| `crates/sdlc-cli/src/cmd/knowledge.rs` | Add `Run { mode, type_, slug }` variant to `KnowledgeLibrarianSubcommand`; implement `run_librarian` match arm |
| `crates/sdlc-cli/src/cmd/investigate.rs` | Add auto-harvest subprocess call on `--status complete` |
| `crates/sdlc-cli/src/cmd/ponder.rs` | Add auto-harvest subprocess call on `--status complete` |
| `crates/sdlc-server/src/routes/knowledge.rs` | Add `maintain_knowledge` and `harvest_knowledge_workspace` handlers |
| `crates/sdlc-server/src/routes/runs.rs` | Add `start_knowledge_maintain` and `start_knowledge_harvest` using `spawn_agent_run` |
| `crates/sdlc-server/src/state.rs` | Add `KnowledgeMaintenanceStarted`, `KnowledgeMaintenanceCompleted { actions_taken }` to `SseMessage` |
| `crates/sdlc-server/src/lib.rs` | Register new routes: `POST /api/knowledge/maintain`, `POST /api/knowledge/harvest` |

## Out of Scope

- Frontend UI for triggering maintenance (covered by a separate feature).
- The agent decision logic itself (agent instructions live in skill templates, not Rust).
- Changing the existing `librarian_init` behavior.
- Deduplication merges (flagging only, no auto-merge).

## Acceptance Criteria

1. `sdlc knowledge librarian run` exits 0 and prints a maintenance summary or agent-invocation instructions.
2. `sdlc knowledge librarian run --mode harvest --type investigation --slug <slug>` exits 0 and produces or updates a knowledge entry for the given workspace.
3. `sdlc investigate update --status complete` triggers a best-effort harvest subprocess call without failing.
4. `sdlc ponder update --status complete` triggers a best-effort harvest subprocess call without failing.
5. `POST /api/knowledge/maintain` returns `{ run_id, status: "started" }` and emits `KnowledgeMaintenanceStarted` over SSE.
6. `POST /api/knowledge/harvest` with valid body returns `{ run_id, status: "started" }`.
7. `SDLC_NO_NPM=1 cargo test --all` passes. `cargo clippy --all -- -D warnings` clean.
