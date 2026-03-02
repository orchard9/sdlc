# QA Plan: Enrichments — Attach Research Context to Feedback Notes

## Unit Tests (sdlc-core)

| # | Test | File |
|---|------|------|
| 1 | `enrich_adds_enrichment` — enrich adds to note and persists | `sdlc-core/src/feedback.rs` |
| 2 | `enrich_missing_returns_error` — error for unknown ID | `sdlc-core/src/feedback.rs` |
| 3 | `enrich_multiple_accumulates` — multiple enrichments stack | `sdlc-core/src/feedback.rs` |
| 4 | `old_yaml_backward_compat_no_enrichments` — old YAML deserializes cleanly | `sdlc-core/src/feedback.rs` |
| 5 | `to_markdown_includes_enrichments` — markdown contains Context block | `sdlc-core/src/feedback.rs` |
| 6 | `to_markdown_no_enrichment_section_when_empty` — no Context block when empty | `sdlc-core/src/feedback.rs` |

## Integration Tests (sdlc-server)

| # | Test | File |
|---|------|------|
| 7 | `enrich_note_returns_updated_note` — POST returns note with enrichments array | `sdlc-server/src/routes/feedback.rs` |
| 8 | `enrich_note_missing_returns_404` — 404 for unknown ID | `sdlc-server/src/routes/feedback.rs` |

## Regression

All existing feedback tests must continue to pass (15 core, 7 server).

## Acceptance Criteria Verification

1. `enrich()` function adds enrichment and returns it — covered by T1/T2
2. POST route returns note with enrichments — covered by T7
3. Backward compat YAML — covered by T4
4. UI "Add context" button on hover — manual/visual check
5. Enrichment updates card in place — manual/visual check
6. `to_markdown` enrichment blocks — covered by T5/T6
7. Existing tests pass — covered by regression suite
