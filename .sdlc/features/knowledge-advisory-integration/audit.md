# Audit: Knowledge-Advisory Integration

## Scope

Security and quality audit of the changes introduced by this feature:
1. `pub fn relevant_entries` in `sdlc-core/src/knowledge.rs`
2. `GET /api/knowledge/relevant` handler in `sdlc-server/src/routes/knowledge.rs`
3. Route registration in `sdlc-server/src/lib.rs`
4. KB injection block in `sdlc-server/src/routes/advisory.rs`

---

## Security Surface Analysis

### Input Handling: `GET /api/knowledge/relevant?tags=a,b&limit=10`

**Finding A1:** The `tags` query parameter is a comma-separated string parsed by splitting on `,`. There is no length cap on the tags string or the number of tags. An attacker could send a very large number of tags (e.g., 10,000 tags) to cause O(n*m) work in `relevant_entries` where n = knowledge entries and m = tags. However:
- The knowledge base is a local YAML file store — in practice it will have tens to hundreds of entries, not millions
- The operation is CPU-bound `String` comparison, not a DB query or network call
- The `limit` cap (50) bounds the output, not the computation

**Action:** Accept as-is. The knowledge base is a local developer tool, not a multi-tenant service. The DoS surface is negligible for the intended use case. Track as a known limitation if the knowledge base grows to thousands of entries.

**Finding A2:** The `limit` parameter is capped at 50 via `.min(50)`. This is correct and prevents accidentally requesting unbounded results.

### Prompt Injection via Knowledge Base Content

**Finding A3:** Knowledge base entry content (`title`, `summary`) is injected into the advisory agent prompt. If a malicious entry's title or summary contained a prompt injection payload (e.g., "Ignore previous instructions and..."), it could influence the advisory agent's behavior.

**Risk assessment:** Low. The knowledge base is written by the project owner and team. It is a local YAML file store with no external write path in this feature. The advisory agent is already trusted with broad tool access.

**Action:** Accept as-is. The threat model is a local developer tool. Adding sanitization would be premature given the trust boundary.

### Graceful Degradation

**Finding A4:** The `spawn_blocking` call for KB lookup in `start_advisory_run` uses `.unwrap_or_default()`. If the blocking task panics, this will propagate the `JoinError` as an `AppError`, which is the correct behavior. If the `relevant_entries` function returns `Err(...)`, it is silently discarded. This is intentional and documented.

**Action:** No issue. Correct behavior.

### Route Ordering

**Finding A5:** The `GET /api/knowledge/relevant` route is registered before `GET /api/knowledge/{slug}`. This is required for correct Axum routing. If the order were reversed, `"relevant"` would be matched as a slug value, causing a `KnowledgeNotFound` error. The inline comment documents this requirement.

**Action:** No issue. Documented and correct.

---

## Code Quality

**Finding A6:** The `unwrap()` in `history.runs.last().unwrap()` in `advisory.rs` was pre-existing (not introduced by this feature). The `runs.is_empty()` check guards it correctly. No regression introduced.

**Finding A7:** The `ADVISORY_TAGS` constant is a `&[&str]` — it's converted to `Vec<String>` before passing to `relevant_entries` because that function takes `&[String]`. This is a minor inefficiency (allocation) but is called once per advisory run, not in a hot path. Acceptable.

---

## Summary

All findings have been addressed. No blocking issues. One accepted limitation (A1) and one accepted risk (A3) that are both appropriate for a local developer tool with a trusted write path.
