# QA Plan: spike-data-layer

## Test matrix

### Unit tests (in spikes.rs)

| Test | What it verifies |
|---|---|
| `parse_adopt_verdict` | `**Verdict:** ADOPT` → `SpikeVerdict::Adopt` |
| `parse_adapt_verdict` | `**Verdict:** ADAPT` → `SpikeVerdict::Adapt` |
| `parse_reject_verdict` | `**Verdict:** REJECT` → `SpikeVerdict::Reject` |
| `parse_no_verdict` | findings.md with no verdict line → `None` |
| `parse_title` | `# Spike: My Title` → title = "My Title" |
| `parse_date` | `**Date:** 2026-03-04` → date = "2026-03-04" |
| `parse_the_question` | `## The Question\nBody text\n\n## Next Section` → "Body text" |
| `parse_open_questions` | `## Risks and Open Questions\n- Item` → "- Item" |
| `parse_missing_sections` | findings.md with no `## The Question` → `None` |
| `list_sorts_by_date` | Two entries with different dates → most recent first |
| `list_empty_dir` | `.sdlc/spikes/` absent → empty Vec, no error |
| `list_absent_findings` | slug dir exists, no findings.md → entry with None fields |
| `promote_creates_ponder` | Creates ponder with spike-findings.md present |
| `promote_seeds_open_questions` | open-questions.md populated from section |
| `promote_records_ponder_slug` | state.yaml ponder_slug set after promote |
| `promote_with_override_slug` | ponder_slug_override respected |
| `store_in_knowledge_creates_entry` | Knowledge entry created with "spike","rejected" tags |
| `store_in_knowledge_idempotent` | Second call returns same slug, no duplicate entry |
| `state_yaml_default_on_absent` | Missing state.yaml → SpikeState::default() |
| `state_yaml_malformed_fallback` | Corrupt state.yaml → SpikeState::default(), no panic |

### Clippy / compile

```bash
SDLC_NO_NPM=1 cargo clippy --all -- -D warnings
SDLC_NO_NPM=1 cargo test --all
```

Both must pass with zero warnings.

## Quality thresholds

- No `unwrap()` or `expect()` in library code
- All public functions return `Result<_>`
- `SpikeState` uses `#[serde(default)]` so missing fields don't fail deserialization
- `list()` side-effect errors (store_in_knowledge) captured and logged, not propagated
