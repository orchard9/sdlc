# QA Plan: Knowledge-Advisory Integration

## Scope

Verify that:
1. `relevant_entries` in `sdlc-core` correctly scores, filters, and orders knowledge entries.
2. `GET /api/knowledge/relevant` returns the correct shape, respects `limit`, excludes drafts.
3. The advisory prompt includes a "## Project Knowledge" block when published entries with matching tags exist.
4. The advisory prompt omits the block when the knowledge base is empty or has no matching tags.
5. No existing advisory or knowledge tests regress.
6. Clippy passes with no new warnings.

---

## Test Cases

### Unit Tests — `sdlc-core/src/knowledge.rs`

**TC-U1: Scoring by tag overlap**
- Setup: create 3 published entries in a temp dir:
  - Entry A: tags `["rust", "async", "pattern"]` — 3 overlapping tags with query `["rust", "async", "pattern"]`
  - Entry B: tags `["rust", "testing"]` — 1 overlapping tag
  - Entry C: tags `["python"]` — 0 overlapping tags
- Call: `relevant_entries(root, &["rust", "async", "pattern"], 10)`
- Assert: returns [A, B] in that order (C excluded, zero score)

**TC-U2: Draft entries excluded**
- Setup: create 1 published entry (tags: `["rust"]`) and 1 draft entry (tags: `["rust", "async"]`)
- Call: `relevant_entries(root, &["rust", "async"], 10)`
- Assert: only the published entry is returned, even though the draft has a higher score

**TC-U3: Tie-breaking by recency**
- Setup: create 2 published entries with the same tag overlap score; entry X has `updated_at` 1 day more recent than entry Y
- Assert: X appears before Y

**TC-U4: Limit is respected**
- Setup: create 5 published entries all with matching tags
- Call: `relevant_entries(root, &["rust"], 3)`
- Assert: exactly 3 entries returned

**TC-U5: Empty knowledge base**
- Call: `relevant_entries(root, &["rust"], 10)` on a root with no `.sdlc/knowledge/` directory
- Assert: returns `Ok([])`, no error

---

### Integration Tests — `crates/sdlc-server/tests/integration.rs`

**TC-I1: GET /api/knowledge/relevant — basic shape**
- Setup: seed a published entry with tags `["rust", "async"]` via `sdlc_core::knowledge::create` + `update`
- Request: `GET /api/knowledge/relevant?tags=rust&limit=5`
- Assert: HTTP 200, JSON array, each item has fields: `slug`, `title`, `code`, `status`, `summary`, `tags`, `created_at`, `updated_at`

**TC-I2: GET /api/knowledge/relevant — draft excluded**
- Setup: seed one published entry (tags: `["rust"]`) and one draft entry (tags: `["rust"]`)
- Request: `GET /api/knowledge/relevant?tags=rust`
- Assert: only 1 result; all results have `status: "published"`

**TC-I3: GET /api/knowledge/relevant — limit enforced**
- Setup: seed 5 published entries all tagged `["rust"]`
- Request: `GET /api/knowledge/relevant?tags=rust&limit=2`
- Assert: exactly 2 results

**TC-I4: GET /api/knowledge/relevant — empty result on no match**
- Setup: seed a published entry with tags `["python"]`
- Request: `GET /api/knowledge/relevant?tags=rust`
- Assert: HTTP 200, empty JSON array `[]`

**TC-I5: GET /api/knowledge/relevant — route does not shadow slug**
- Request: `GET /api/knowledge/rust-patterns` (where `rust-patterns` is a real slug)
- Assert: HTTP 200 with the full entry shape (not matched by the `/relevant` route)

**TC-I6: POST /api/advisory/run — succeeds with non-empty KB**
- Setup: seed a published entry with tags `["architecture"]`
- Request: `POST /api/advisory/run`
- Assert: HTTP 200, response has `run_id` field (prompt injection did not cause a panic or error)

**TC-I7: POST /api/advisory/run — succeeds with empty KB**
- Setup: no knowledge base directory
- Request: `POST /api/advisory/run`
- Assert: HTTP 200, response has `run_id` (graceful degradation)

---

### Clippy / Build

**TC-B1:** `SDLC_NO_NPM=1 cargo test --all` passes with no test failures.

**TC-B2:** `cargo clippy --all -- -D warnings` passes with no warnings introduced by this feature.

---

## Acceptance Gate

All TC-U* and TC-I* tests pass. TC-B1 and TC-B2 pass. No regressions in existing knowledge or advisory test cases.
