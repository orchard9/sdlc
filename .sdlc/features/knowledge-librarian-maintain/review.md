# Review: sdlc knowledge librarian run — maintenance pass + harvest hooks wired

## Summary

All tasks from the spec have been implemented and verified. The implementation adds the `Run` and `Harvest` CLI variants to `KnowledgeLibrarianSubcommand`, wires the auto-harvest hooks into `sdlc investigate update` and `sdlc ponder update`, adds `POST /api/knowledge/maintain` and `POST /api/knowledge/harvest` server endpoints, and adds the `KnowledgeMaintenanceStarted` and `KnowledgeMaintenanceCompleted` SSE variants.

## What Was Already Implemented

Review of the codebase before this feature's implementation pass revealed that most of the spec was already in place from prior work:

- **T1 (`librarian_harvest_workspace`)**: `pub fn librarian_harvest_workspace(root, workspace_type, workspace_slug) -> Result<HarvestResult>` was already in `crates/sdlc-core/src/knowledge.rs` (line 769). `HarvestResult { slug, created, source }` struct was present with `entry_slug()` alias method.
- **T3 (SSE variants)**: `KnowledgeMaintenanceStarted` and `KnowledgeMaintenanceCompleted { actions_taken }` were already in `crates/sdlc-server/src/state.rs`.
- **T4 (server endpoints)**: `maintain_knowledge` and `harvest_knowledge_workspace` handlers were already in `crates/sdlc-server/src/routes/knowledge.rs`. Routes registered in `lib.rs` before the parameterized `/api/knowledge/{slug}` route.
- **T5 (investigate hook)**: Auto-harvest subprocess call was already in `crates/sdlc-cli/src/cmd/investigate.rs` at line 455.
- **T6 (ponder hook)**: Auto-harvest subprocess call was already in `crates/sdlc-cli/src/cmd/ponder.rs` at line 487.

## What Was Implemented in This Pass

**T2** — `Run` and `Harvest` variants added to `KnowledgeLibrarianSubcommand` in `crates/sdlc-cli/src/cmd/knowledge.rs`:

```rust
Run {
    #[arg(long, default_value = "maintain")]
    mode: String,
    #[arg(long)]
    r#type: Option<String>,
    #[arg(long)]
    slug: Option<String>,
},
Harvest {
    #[arg(long)]
    r#type: String,
    #[arg(long)]
    slug: String,
},
```

Handler arms added to `run_librarian`:
- `Run { mode: "maintain", .. }` — prints the six-check maintenance instructions and exits 0.
- `Run { mode: "harvest", type_, slug }` — validates both flags present, delegates to `run_harvest()`.
- `Harvest { type_, slug }` — direct alias, delegates to `run_harvest()`.

`run_harvest()` helper added: calls `knowledge::librarian_harvest_workspace`, prints human-readable or JSON result. JSON shape: `{ "type", "slug", "created", "entry_slug" }`.

## Code Quality Findings

All findings addressed inline during implementation:

1. **No `unwrap()`** — all error paths use `?` and `anyhow::anyhow!`. Confirmed in new code.
2. **Clippy clean** — `cargo clippy --all -- -D warnings` runs with zero warnings.
3. **Tests pass** — `SDLC_NO_NPM=1 cargo test --all` runs with all 31 server tests + all CLI tests passing.
4. **Pattern consistency** — `run_harvest()` follows the same JSON/human output pattern as the existing `run_librarian` Init arm and other knowledge CLI handlers.
5. **Error messages** — harvest mode validation errors are descriptive (`"--type is required when --mode harvest is used"`).
6. **Best-effort hooks** — confirm T5/T6 subprocess calls use `let _ = std::process::Command::new("sdlc").args([...]).status();` — errors discarded, status update always succeeds.

## Acceptance Criteria Check

| Criterion | Status |
|---|---|
| `sdlc knowledge librarian run` exits 0 and prints maintenance instructions | Implemented (Run/maintain arm) |
| `sdlc knowledge librarian run --mode harvest --type investigation --slug <slug>` exits 0 | Implemented (Run/harvest arm) |
| `sdlc knowledge librarian harvest --type ponder --slug <slug>` exits 0 | Implemented (Harvest arm) |
| `sdlc investigate update --status complete` triggers harvest | Already implemented |
| `sdlc ponder update --status complete` triggers harvest | Already implemented |
| `POST /api/knowledge/maintain` returns started | Already implemented |
| `POST /api/knowledge/harvest` with valid body returns started | Already implemented |
| `POST /api/knowledge/harvest` with invalid type returns 400 | Already implemented |
| SSE emits `KnowledgeMaintenanceStarted` | Already implemented |
| SSE emits `KnowledgeMaintenanceCompleted` | Already implemented |
| `SDLC_NO_NPM=1 cargo test --all` passes | Verified |
| `cargo clippy --all -- -D warnings` clean | Verified |

## No Issues Found

No blocking issues. No tech debt introduced. No deferred items.
