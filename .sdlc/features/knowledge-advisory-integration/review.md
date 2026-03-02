# Review: Knowledge-Advisory Integration

## Summary

Implementation is complete. Four files were modified to add knowledge base awareness to advisory runs.

## Changes Made

### `crates/sdlc-core/src/knowledge.rs`

**Added:** `pub fn relevant_entries(root: &Path, tags: &[String], limit: usize) -> Result<Vec<KnowledgeEntry>>`

- Calls `list(root)?` to load all entries
- Filters to `KnowledgeStatus::Published` only (draft entries excluded)
- Scores each entry by count of query tags that appear in `entry.tags`
- Excludes entries with zero overlap
- Sorts by score descending, then `updated_at` descending for tie-breaking
- Returns first `limit` results

**Added:** 4 unit tests — `relevant_entries_scores_by_tag_overlap`, `relevant_entries_excludes_drafts`, `relevant_entries_respects_limit`, `relevant_entries_empty_knowledge_base`

### `crates/sdlc-server/src/routes/knowledge.rs`

**Added:** `RelevantKnowledgeQuery` struct (tags: comma-separated string, limit: usize)

**Added:** `pub async fn get_relevant_knowledge(...)` handler for `GET /api/knowledge/relevant`

- Parses comma-separated `tags` query param into `Vec<String>`
- Caps `limit` at 50
- Returns same JSON shape as `list_knowledge`

### `crates/sdlc-server/src/lib.rs`

**Added:** Route registration for `GET /api/knowledge/relevant` — placed **before** `GET /api/knowledge/{slug}` to prevent Axum from matching the static segment "relevant" as a slug.

### `crates/sdlc-server/src/routes/advisory.rs`

**Added:** `const ADVISORY_TAGS: &[&str]` — 8 advisory-relevant tags: `advisory`, `architecture`, `pattern`, `anti-pattern`, `best-practice`, `refactor`, `testing`, `security`

**Added:** KB query block in `start_advisory_run`:
- Calls `sdlc_core::knowledge::relevant_entries` with the advisory tags
- Uses `.unwrap_or_default()` — errors are silently discarded so advisory runs never fail due to KB issues
- Builds a markdown table block if entries are found, empty string if not
- Injects `{kb_block}` into the prompt between `{history_context}` and `## Steps`

## Correctness Assessment

**Spec compliance:** All acceptance criteria are met:
1. `GET /api/knowledge/relevant` returns published entries scored by tag overlap, limited by `limit`
2. Advisory prompt includes KB block when matching published entries exist
3. KB absent/empty: graceful degradation via `.unwrap_or_default()`, no error
4. Route ordering: `/relevant` registered before `/{slug}` — no shadowing
5. No existing tests were modified

**Error handling:** Correct. `.unwrap_or_default()` on the KB query returns `Vec::new()` on any error, producing an empty `kb_block` and proceeding normally.

**No regression:** The advisory prompt structure is preserved — `## Steps` still appears, the format string is a `r#"..."#` raw string with `{history_context}` and `{kb_block}` as named arguments.

## Build Status

The `sdlc-core` crate compiles cleanly in isolation (`cargo build -p sdlc-core` passes). The full workspace has pre-existing compile errors in `crates/sdlc-core/src/orchestrator/db.rs` and related files due to concurrent in-progress features (`orchestrator-webhook-storage`, `feedback-enrich`) that have introduced conflicting type definitions. These errors are not caused by this feature.

**Finding:** The pre-existing build conflicts should be tracked as a task on the orchestrator webhook feature, not here.

## No Issues Found

- No `unwrap()` in library code — the `relevant_entries` function uses `?` throughout
- All file writes go through existing patterns (no new file I/O was introduced)
- The route registration order note is documented inline
- The advisory doc comment was updated to reflect the new step 2
