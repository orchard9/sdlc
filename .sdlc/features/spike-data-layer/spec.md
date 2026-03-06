# Spec: spike-data-layer

## Purpose

A minimal read-only Rust data layer in `sdlc-core/src/spikes.rs` that surfaces spikes for the CLI, REST, and UI layers. Spikes are technical investigations created by `/sdlc-spike` — their findings live in `.sdlc/spikes/<slug>/findings.md`. This layer reads, parses, and acts on those files without owning any write path for spike content itself.

## Non-goals

- Writing or modifying findings.md (owned by the `/sdlc-spike` agent)
- Session logging or artifact management (spikes have no session model)
- Decision logic (e.g., "should this spike be promoted?") — that belongs in skill instructions

## Data Layout

```
.sdlc/spikes/
  <slug>/
    findings.md       ← written by /sdlc-spike agent; source of truth
    state.yaml        ← written by this layer; minimal mutable state only
```

### findings.md parsing

The layer extracts these fields from findings.md headers using line-by-line scan:

| Field | Pattern | Notes |
|---|---|---|
| `title` | `# Spike: <title>` | First H1 line |
| `verdict` | `**Verdict:** ADOPT\|ADAPT\|REJECT` | Bold-wrapped, case-insensitive |
| `date` | `**Date:** <ISO date>` | Parsed as NaiveDate |
| `the_question` | Contents of `## The Question` section | Up to next `##` heading |
| `open_questions` | Contents of `## Risks and Open Questions` section | Up to next `##` heading |

If findings.md does not exist for a slug in state.yaml, the entry is surfaced with `verdict: None`.

### state.yaml schema

```yaml
verdict: ADAPT          # mirrors parsed verdict; written on first list() call
ponder_slug: null       # set by promote_to_ponder
knowledge_slug: null    # set by store_in_knowledge
```

State is written atomically via `crate::io::atomic_write`. The file is optional — if absent, state is derived from findings.md alone.

## Public API

```rust
pub struct SpikeEntry {
    pub slug: String,
    pub title: String,
    pub verdict: Option<SpikeVerdict>,
    pub date: Option<String>,         // ISO date string as-found
    pub the_question: Option<String>, // extracted section text
    pub ponder_slug: Option<String>,  // from state.yaml
    pub knowledge_slug: Option<String>, // from state.yaml
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpikeVerdict {
    Adopt,
    Adapt,
    Reject,
}

/// List all spikes from .sdlc/spikes/. For each REJECT spike that has no
/// knowledge_slug yet, auto-calls store_in_knowledge() as a side effect.
pub fn list(root: &Path) -> Result<Vec<SpikeEntry>>

/// Load full metadata + raw findings.md content for a single spike.
pub fn load(root: &Path, slug: &str) -> Result<(SpikeEntry, String)>

/// Extract the open questions section text from a findings.md string.
pub fn extract_open_questions(findings: &str) -> Option<String>

/// Create a ponder entry pre-seeded with spike findings and open questions.
/// Records the ponder_slug back into state.yaml.
/// Returns the ponder slug.
pub fn promote_to_ponder(root: &Path, slug: &str, ponder_slug_override: Option<&str>) -> Result<String>

/// File a REJECT spike's findings into the knowledge base.
/// Records the knowledge_slug back into state.yaml.
/// Idempotent — does nothing if knowledge_slug already set in state.yaml.
pub fn store_in_knowledge(root: &Path, slug: &str) -> Result<String>
```

## list() behaviour

1. Read `.sdlc/spikes/` directory; collect all subdirectory names as slugs
2. For each slug: read state.yaml (if present) + parse findings.md headers
3. Merge into `SpikeEntry`
4. If verdict == Reject and `knowledge_slug` is None → call `store_in_knowledge()` (side effect, errors are logged but do not fail the list call)
5. Sort by date descending (most recent first); undated entries sort last
6. Return Vec<SpikeEntry>

## promote_to_ponder() behaviour

1. Call `load()` to get entry + findings content
2. Derive ponder_slug: `ponder_slug_override.unwrap_or(&entry.slug)`
3. Create ponder entry via `ponder::create(root, ponder_slug, title)` where title = `entry.the_question.unwrap_or(entry.title)`
4. Capture full findings as `spike-findings.md` via `ponder::capture_artifact(root, ponder_slug, "spike-findings.md", findings_content)`
5. If `extract_open_questions(findings)` returns Some(text) → capture as `open-questions.md`
6. Write ponder_slug back to state.yaml
7. Return ponder_slug

## store_in_knowledge() behaviour

1. Load entry
2. If `knowledge_slug` already set in state.yaml → return it (idempotent)
3. Derive knowledge slug: `format!("spike-{}", slug)` (prefix avoids collisions)
4. Call `knowledge::create(root, knowledge_slug, title, "900")` — code "900" = investigations/decisions category
5. Read findings.md → call `knowledge::append_content(root, knowledge_slug, findings)`
6. Call `knowledge::update(root, knowledge_slug, None, None, &["spike", "rejected"], &[])` to add tags
7. Write knowledge_slug back to state.yaml
8. Return knowledge_slug

## Error handling

- No `unwrap()` — all fallible operations use `?`
- Malformed findings.md (no verdict, no title) → returns `SpikeEntry` with `None` fields, not an error
- Ponder already exists on promote → surface `SdlcError::PonderExists` (caller handles)
- Knowledge already exists → should not happen due to idempotency check, but if so return the existing slug

## Tests

- Parse findings.md with all four verdict states (ADOPT, ADAPT, REJECT, absent)
- Parse findings.md with missing sections (no `## The Question`, no `## Risks and Open Questions`)
- `list()` returns entries sorted by date
- `promote_to_ponder()` creates ponder with correct artifacts
- `store_in_knowledge()` is idempotent (second call returns same slug)
- `extract_open_questions()` handles missing section gracefully
