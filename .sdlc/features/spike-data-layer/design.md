# Design: spike-data-layer

## Module location

```
crates/sdlc-core/src/spikes.rs     ← new module
crates/sdlc-core/src/lib.rs        ← pub mod spikes;
crates/sdlc-core/src/paths.rs      ← spike path helpers
```

## File layout (runtime)

```
.sdlc/spikes/
  <slug>/
    findings.md        ← written by /sdlc-spike; read-only from this layer
    state.yaml         ← written by this layer; verdict + ponder_slug + knowledge_slug
```

## Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpikeVerdict { Adopt, Adapt, Reject }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SpikeState {
    #[serde(skip_serializing_if = "Option::is_none")]
    verdict: Option<SpikeVerdict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ponder_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    knowledge_slug: Option<String>,
}

pub struct SpikeEntry {
    pub slug: String,
    pub title: String,              // from findings.md H1
    pub verdict: Option<SpikeVerdict>, // parsed from findings.md
    pub date: Option<String>,       // from findings.md **Date:** line
    pub the_question: Option<String>, // ## The Question section
    pub ponder_slug: Option<String>,  // from state.yaml
    pub knowledge_slug: Option<String>, // from state.yaml
}
```

## paths.rs additions

```rust
pub const SPIKES_DIR: &str = ".sdlc/spikes";

pub fn spike_dir(root: &Path, slug: &str) -> PathBuf {
    root.join(SPIKES_DIR).join(slug)
}
pub fn spike_findings_path(root: &Path, slug: &str) -> PathBuf {
    spike_dir(root, slug).join("findings.md")
}
pub fn spike_state_path(root: &Path, slug: &str) -> PathBuf {
    spike_dir(root, slug).join("state.yaml")
}
```

## Parsing strategy

`parse_findings(content: &str) -> ParsedFindings` — pure function, no I/O:

```
Pass 1: line-by-line scan for header fields
  "# Spike: "          → title
  "**Verdict:** "      → verdict (trimmed, case-insensitive match)
  "**Date:** "         → date

Pass 2: section extraction (single pass, state machine)
  On "## The Question"           → start collecting into the_question buffer
  On "## Risks and Open Questions" → start collecting into open_questions buffer
  On any other "## " heading     → stop collecting
```

Section text is trimmed of leading/trailing blank lines. No regex — plain `str::starts_with` and `str::trim`.

## state.yaml I/O

Read: `serde_yaml::from_str::<SpikeState>(...)` with `Default` fallback if file absent.
Write: `crate::io::atomic_write(path, serde_yaml::to_string(&state)?.as_bytes())`

## list() call graph

```
list(root)
  ├── read_dir(.sdlc/spikes/)
  │   └── for each slug dir:
  │       ├── read state.yaml (optional, default empty)
  │       ├── read findings.md (optional)
  │       └── parse_findings(content) → merge with state → SpikeEntry
  ├── sort by date desc
  └── for each REJECT entry where knowledge_slug is None:
      └── store_in_knowledge(root, slug)   ← side effect, errors non-fatal
```

## promote_to_ponder() call graph

```
promote_to_ponder(root, slug, override)
  ├── load(root, slug) → (entry, findings_raw)
  ├── ponder_slug = override ?? entry.slug
  ├── title = entry.the_question ?? entry.title
  ├── ponder::create(root, ponder_slug, title)
  ├── ponder::capture_artifact(root, ponder_slug, "spike-findings.md", findings_raw)
  ├── if let Some(oq) = extract_open_questions(findings_raw):
  │   └── ponder::capture_artifact(root, ponder_slug, "open-questions.md", oq)
  ├── state.ponder_slug = ponder_slug
  └── write_state(root, slug, state)
```

## store_in_knowledge() call graph

```
store_in_knowledge(root, slug)
  ├── load state.yaml
  ├── if state.knowledge_slug is Some → return it (idempotent)
  ├── load findings.md
  ├── parse title
  ├── knowledge_slug = format!("spike-{slug}")
  ├── knowledge::create(root, knowledge_slug, title, "900")
  │   → error if exists: derive unique slug with "-2" suffix
  ├── knowledge::append_content(root, knowledge_slug, findings_raw)
  ├── knowledge::update(..., tags_add=["spike","rejected"])
  ├── state.knowledge_slug = knowledge_slug
  └── write_state(root, slug, state)
```

## Dependencies (Cargo.toml additions — sdlc-core)

No new dependencies needed. Parsing uses `std::str`, state uses `serde_yaml` (already present), I/O uses `crate::io::atomic_write`.

## Integration points

| Caller | Function | What it needs |
|---|---|---|
| sdlc-cli spike commands | `list()`, `load()`, `promote_to_ponder()` | All public |
| sdlc-server REST routes | `list()`, `load()`, `promote_to_ponder()` | All public |
| `list()` itself | `store_in_knowledge()` | Internal call, non-fatal |

## Error cases

| Situation | Behaviour |
|---|---|
| `.sdlc/spikes/` doesn't exist | `list()` returns empty Vec (not an error) |
| findings.md absent for a slug | Entry surfaced with None fields |
| Ponder already exists on promote | Propagate `SdlcError::PonderExists` |
| Knowledge create fails in store_in_knowledge | Log warning, do not fail `list()` |
| state.yaml malformed | Treat as missing (default SpikeState) |
