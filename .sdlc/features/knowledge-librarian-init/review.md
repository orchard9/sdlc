# Review: sdlc knowledge librarian init

## Summary

The implementation is complete and correct. The `librarian_init` function and all supporting helpers are in `crates/sdlc-core/src/knowledge.rs`. The CLI dispatch is in `crates/sdlc-cli/src/cmd/knowledge.rs`. All 3 tasks (T1 idempotency test, T2 CLI integration tests, T3 clippy clean) are verified passing.

## Spec Compliance

| Acceptance Criterion | Status | Notes |
|---|---|---|
| 1. Empty project: creates `.sdlc/knowledge/` and `catalog.yaml` with ≥1 class | PASS | `librarian_init_on_empty_project` test |
| 2. Completed investigations → one entry per investigation | PASS | `harvest_investigation_creates_entry` test |
| 3. Committed ponders → one entry per ponder | PASS | `harvest_ponder_committed_creates_entry` test |
| 4. Published guideline → entry with `origin: guideline` | PASS | `harvest_guideline_creates_entry` test |
| 5. Running twice produces same result (idempotency) | PASS | `librarian_init_idempotent_full` test (T1) |
| 6. Librarian agent file always written with project name and catalog | PASS | `librarian_init_creates_agent_file` test |
| 7. Cross-ref links ≥2-tag entries; no duplicate links on second run | PASS | `cross_ref_pass_links_entries`, `cross_ref_pass_no_duplicate_links` |
| 8. `--json` outputs valid JSON matching schema | PASS | `knowledge_librarian_init_json_output` CLI test |
| 9. Step failure returns non-zero with error; previous steps not rolled back | PASS | All IO uses `?`, partial state is preserved |

## Code Quality

- No `unwrap()` in library code — all IO uses `?` operator throughout `librarian_init`, `harvest_*`, `seed_catalog`, `write_librarian_agent_file`, `cross_ref_pass`, `upsert_knowledge_entry`
- One intentional `unwrap_or_default()` in `write_librarian_agent_file` for YAML serialization failure (safe: returns empty string, template still written)
- `#[allow(clippy::too_many_arguments)]` on `upsert_knowledge_entry` is appropriate given the function's role as an internal upsert helper — not a public API
- All file writes go through `crate::io::atomic_write` (design spec requirement)
- `write_librarian_agent_file` is `pub` to allow direct invocation from server routes — correctly scoped

## Architecture Compliance

- No LLM calls in the Rust layer — confirmed
- No server routes or frontend changes — confirmed, CLI-only
- Decision logic (what "durable insight" means, when to re-run) remains in skill text, not Rust — confirmed
- `cross_ref_pass` correctly counts one pair as one link event even if both directions are added

## Test Coverage

- 36 knowledge unit tests pass (including the new `librarian_init_idempotent_full`)
- 3 CLI integration tests pass (`exits_zero`, `is_idempotent`, `json_output`)
- Clippy: zero warnings on both `sdlc-core` and `sdlc-cli`

## Minor Observations (non-blocking)

1. `seed_catalog` calls `add_class` in a loop which writes `catalog.yaml` on each call — minor write amplification (5 writes for 5 defaults). Acceptable at current scale; track with `sdlc task add` if ever needed.
2. `cross_ref_pass` loads all entries into memory and then O(n²) compares — design-acknowledged as acceptable for <200 entries.
3. `harvest_guidelines` reads `std::fs::read_to_string(publish_path).unwrap_or_default()` — silently treats missing file as empty content. Consistent with fail-independently semantics; no change needed.

## Verdict

APPROVED. Implementation matches spec, all tests pass, clippy clean, conventions followed.
