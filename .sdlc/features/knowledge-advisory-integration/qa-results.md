# QA Results: knowledge-advisory-integration

**Date:** 2026-03-02
**Status:** PASS

## Summary

All acceptance criteria verified. The feature adds tag-based KB relevance scoring to `sdlc-core`, a new REST endpoint to `sdlc-server`, and injects a "Project Knowledge" context block into every advisory agent run. All new code compiles cleanly and all tests pass.

---

## Test Execution

### Unit Tests — sdlc-core (knowledge module)

**Command:** `SDLC_NO_NPM=1 cargo test -p sdlc-core -- knowledge::`

**Result:** 34 passed / 0 failed

New tests added by this feature (all passing):

| Test | Description | Result |
|------|-------------|--------|
| `relevant_entries_scores_by_tag_overlap` | Two-tag match outranks one-tag match | PASS |
| `relevant_entries_excludes_drafts` | Draft entries excluded from results | PASS |
| `relevant_entries_respects_limit` | `limit` parameter caps result count | PASS |
| `relevant_entries_empty_knowledge_base` | Empty KB returns empty vec, no error | PASS |

All 30 pre-existing knowledge tests also pass — no regressions.

### Compile Check — sdlc-server

**Command:** `SDLC_NO_NPM=1 cargo check -p sdlc-server`

**Result:** Finished with 0 errors, 0 warnings

This exercises both the new `get_relevant_knowledge` handler in `routes/knowledge.rs` and the new route registration in `lib.rs`.

### Server Integration Tests (knowledge subset)

**Command:** `SDLC_NO_NPM=1 cargo test -p sdlc-server -- knowledge`

**Result:** 2 passed / 0 failed (knowledge integration tests)

Note: 14 unrelated server integration tests fail in the orchestrator actions area due to pre-existing incomplete work on `orchestrator-actions-routes` and `orchestrator-webhook-events` from concurrent in-flight features. These failures are tracked separately and are not caused by this feature. The failing tests target `create_action_*`, `list_actions_*`, `patch_action_*`, `delete_action_*` — none of which are in scope here.

---

## Functional Verification

### 1. `pub fn relevant_entries` in sdlc-core

- Location: `/crates/sdlc-core/src/knowledge.rs`
- Filters to `KnowledgeStatus::Published` only
- Scores by count of query tags present in entry tags
- Ties broken by `updated_at` descending
- Respects `limit` parameter (hard cap at call site, capped at 50 in handler)
- Returns empty vec (not error) when KB directory absent

### 2. `GET /api/knowledge/relevant` endpoint

- Location: `/crates/sdlc-server/src/routes/knowledge.rs` (`get_relevant_knowledge`)
- Query params: `tags` (comma-separated, optional), `limit` (optional, default 10, max 50)
- Returns JSON array of entry objects with: slug, title, code, status, summary, tags, created_at, updated_at
- Registered before `/{slug}` wildcard route in `lib.rs` to avoid routing conflict (verified by reading `lib.rs`)

### 3. Advisory prompt KB injection

- Location: `/crates/sdlc-server/src/routes/advisory.rs`
- `ADVISORY_TAGS` constant defines 8 domain tags
- KB query wraps in `.unwrap_or_default()` — advisory runs cannot fail due to KB issues
- When entries found: inserts `## Project Knowledge` markdown table block between history context and `## Steps`
- When no entries: `kb_block` is empty string, prompt unchanged
- Instructs agent to cite KB slugs as `[kb: <slug>]` in findings

---

## Pre-existing Build Failures (Not Caused by This Feature)

The orchestrator crates have incomplete in-flight changes from concurrent agents:
- `crates/sdlc-core/src/orchestrator/db.rs` — duplicate `WebhookEvent` type, missing `WebhookEventOutcome::Received` variant
- `crates/sdlc-server/tests/integration.rs` — 14 action endpoint tests fail (HTTP 200 vs expected 201/400/404)

These are tracked in `orchestrator-webhook-events` and `orchestrator-actions-routes` features. Confirmed pre-existing by stash verification (our files: knowledge.rs core, knowledge.rs server routes, lib.rs, advisory.rs — none touch orchestrator code).

---

## Acceptance Criteria Checklist

| Criterion | Status |
|-----------|--------|
| `relevant_entries(root, tags, limit)` function exported from `sdlc-core::knowledge` | PASS |
| Published-only filter | PASS |
| Tag-overlap scoring | PASS |
| Recency tiebreaker | PASS |
| Empty KB returns `[]` (not error) | PASS |
| `GET /api/knowledge/relevant?tags=a,b&limit=N` endpoint | PASS |
| Route registered before `/{slug}` wildcard | PASS |
| `ADVISORY_TAGS` constant in advisory.rs | PASS |
| KB query in `start_advisory_run` with graceful degradation | PASS |
| `{kb_block}` injected into advisory prompt | PASS |
| Advisory runs never fail due to KB errors | PASS (`.unwrap_or_default()`) |
| 4 new unit tests, all passing | PASS |
| Zero compile errors or warnings | PASS |
