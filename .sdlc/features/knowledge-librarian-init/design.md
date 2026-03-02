# Design: sdlc knowledge librarian init

## Architecture

This feature is implemented entirely in the Rust data layer. No server routes, no frontend changes, no agent decision logic in Rust. All decision logic (what counts as a "durable insight", when to re-run init) lives in agent skill text.

### Call Graph

```
sdlc knowledge librarian init
  └─ crates/sdlc-cli/src/cmd/knowledge.rs :: run_librarian()
       └─ crates/sdlc-core/src/knowledge.rs :: librarian_init(root)
            ├─ harvest_investigations(root)   → Vec<HarvestResult>
            ├─ harvest_ponders(root)          → Vec<HarvestResult>
            ├─ harvest_guidelines(root)       → Vec<HarvestResult>
            ├─ seed_catalog(root)             → Catalog          [no-op if exists]
            ├─ write_librarian_agent_file(root) → PathBuf        [always overwrites]
            ├─ cross_ref_pass(root)           → usize
            └─ append_maintenance_action(root, ...)
```

### Key Data Structures

```rust
// In crates/sdlc-core/src/knowledge.rs

pub struct HarvestResult {
    pub slug: String,
    pub created: bool,   // true = new entry, false = appended
    pub source: String,  // "investigation/<slug>" or "ponder/<slug>"
}

pub struct LibrarianInitReport {
    pub investigation_results: Vec<HarvestResult>,
    pub ponder_results: Vec<HarvestResult>,
    pub guideline_results: Vec<HarvestResult>,
    pub catalog_created: bool,
    pub catalog_class_count: usize,
    pub agent_file_path: std::path::PathBuf,
    pub cross_ref_count: usize,
}
```

### Filesystem Layout (post-init)

```
.sdlc/
  knowledge/
    catalog.yaml                    ← taxonomy (created by seed_catalog)
    maintenance-log.yaml            ← append-only action log
    investigation-<slug>/
      entry.yaml                    ← KnowledgeEntry (origin: harvested)
      content.md                    ← harvested text
    ponder-<slug>/
      entry.yaml
      content.md
    guideline-<slug>/
      entry.yaml                    ← KnowledgeEntry (origin: guideline)
      content.md

.claude/
  agents/
    knowledge-librarian.md          ← generated agent file (always overwritten)
```

## Module Boundaries

| Module | Responsibility |
|--------|---------------|
| `sdlc-core/knowledge.rs` | All harvest logic, catalog seeding, cross-ref pass, agent file write |
| `sdlc-cli/cmd/knowledge.rs` | CLI dispatch, argument parsing, output formatting |
| `sdlc-core/investigation.rs` | Read-only: `list()`, `list_sessions()`, `read_session()` |
| `sdlc-core/ponder.rs` | Read-only: `PonderEntry::list()`, scrapbook read |
| `sdlc-core/io.rs` | `atomic_write()` for all file writes |

## Idempotency Design

Every write path uses an "upsert" pattern via `upsert_knowledge_entry()`:

```rust
fn upsert_knowledge_entry(
    root: &Path,
    knowledge_slug: &str,
    title: &str,
    summary: &str,
    tags: &[String],
    origin: OriginKind,
    source: &str,
    content: &str,
) -> Result<bool>  // true = created, false = updated
```

- If `.sdlc/knowledge/<slug>/entry.yaml` is absent: call `knowledge::create()` + `knowledge::append_content()`
- If present: call `knowledge::append_content()` only (no title/origin overwrite)

Catalog seeding checks `catalog_path.exists()` before writing — never overwrites.

Agent file always overwrites (intentional: picks up catalog changes on re-run).

Cross-ref pass checks `entries[i].related.contains(&slug_j)` before adding links.

## Content Extraction

### Investigations (root-cause + evolve)

Source fields used (in order):
1. `inv.context` — the investigation's context/description field
2. Session 1 body — `list_sessions()` + `read_session(root, slug, sessions[0].session)`

Combined as:
```markdown
# <title>

<context>

## Session 1

<session body>
```

### Ponders (committed only)

Reads all scrapbook artifacts from the ponder workspace. The `read_ponder_content()` helper aggregates available artifact files into a single Markdown block.

### Guidelines

Reads the file at `inv.publish_path` directly. No transformation — the guideline document becomes the knowledge entry content as-is.

## Catalog Seeding Algorithm

```
1. Read ARCHITECTURE.md → collect "## Heading" lines (up to 7)
2. If >= 3 headings found:
     zip(["100","200","300","400","500","600","700"], headings)
     → call add_class(root, code, heading_name, None) for each pair
3. Else (fewer than 3 headings or no ARCHITECTURE.md):
     use hardcoded defaults:
       100: Architecture & Design
       200: Development
       300: Process
       400: Research
       500: Operations
4. Return load_catalog(root)
```

## Librarian Agent File Template

Embedded as `const LIBRARIAN_AGENT_TEMPLATE: &str` in `knowledge.rs`. Two substitution tokens:

| Token | Replaced with |
|-------|--------------|
| `{PROJECT_NAME}` | directory name of `root` (or title from VISION.md if present) |
| `{CATALOG_YAML}` | `serde_yaml::to_string(&catalog)` |

Output written to `.claude/agents/knowledge-librarian.md` via `atomic_write`.

## Cross-Reference Pass Algorithm

```
For each pair (i, j) where i < j:
  overlap = |entries[i].tags ∩ entries[j].tags|
  if overlap >= 2:
    if entries[j].slug not in entries[i].related:
      entries[i].related.push(entries[j].slug)
      link_count += 1
    if entries[i].slug not in entries[j].related:
      entries[j].related.push(entries[i].slug)
      link_count += 1
For each modified entry: save(root, entry)
Return link_count
```

Time complexity: O(n² × t) where n = entry count, t = avg tag count. Acceptable for typical project sizes (< 200 entries).

## Error Handling

- All IO operations use `?` — no `unwrap()` in library code
- Each harvest function (`harvest_investigations`, `harvest_ponders`, `harvest_guidelines`) propagates errors but does not abort the entire init on a single entry failure
- The CLI handler wraps with `.context("librarian init failed")` for human-readable error messages
- Partial state on failure is safe: already-created entries remain; re-running picks up from the first incomplete step

## Testing Approach

Integration tests in `crates/sdlc-core/src/knowledge.rs` (existing `#[cfg(test)]` block):
- `librarian_init_on_empty_project` — runs on empty tempdir, no crash, zero harvested
- `librarian_init_creates_agent_file` — agent file written to correct path
- `librarian_init_idempotent` — two runs produce identical report

New test to add (T1 from manifest):
- `librarian_init_idempotent_with_entries` — seed one investigation and one ponder, run twice, assert second run has `created: false` for all results and no duplicate related links
