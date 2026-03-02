# Spec: sdlc knowledge librarian init

## Overview

`sdlc knowledge librarian init` bootstraps the project knowledge base in a single idempotent command. It harvests durable insights from completed workspaces (investigations, ponders, published guidelines), derives a taxonomy catalog from project architecture docs, seeds the catalog file, writes a project-specific librarian agent file, and cross-references related entries.

The command is safe to run multiple times. Each of the 9 steps is independently idempotent — a step that has already completed does nothing.

## Problem

Knowledge generated during root-cause investigations, ponder sessions, and guideline authoring is scattered across `.sdlc/investigations/`, `.sdlc/roadmap/`, and published guideline files. There is no central, searchable, cross-referenced library. Agents starting a new task have no way to query existing decisions and learnings. The knowledge base exists as a CRUD layer but is empty until an agent explicitly seeds it.

`librarian init` is the bootstrap command that turns a populated project workspace into a live knowledge base with zero manual configuration.

## Behavior

### Command

```bash
sdlc knowledge librarian init
sdlc knowledge librarian init --json   # machine-readable report
```

### 9-Step Flow (in order)

| # | Step | Idempotency |
|---|------|-------------|
| 1 | Ensure `.sdlc/knowledge/` directory exists | `create_dir_all` is a no-op if present |
| 2 | Harvest completed investigations | `upsert_knowledge_entry` appends if slug exists |
| 3 | Harvest committed ponders | same upsert logic |
| 4 | Harvest published guidelines (origin: guideline) | same upsert logic |
| 5 | Seed catalog from ARCHITECTURE.md H2 headings (5–7 classes, codes 100–700) | no-op if `catalog.yaml` already exists |
| 6 | Write `.claude/agents/knowledge-librarian.md` (always overwrites to pick up catalog changes) | overwrite is intentional |
| 7 | Hook registration — log maintenance action in `maintenance-log.yaml` | append-only |
| 8 | Cross-reference pass — link entries sharing ≥2 tags via `related[]` | idempotent: skips existing links |
| 9 | Return `LibrarianInitReport` and print summary | read-only |

### Harvest Protocol

For each completed investigation or committed ponder:

1. Derive a `knowledge_slug` (`investigation-<slug>` or `ponder-<slug>`)
2. Check for existing entry at `.sdlc/knowledge/<knowledge_slug>/`
3. If absent: create new entry (`status: draft`, `origin: harvested`, `harvested_from: "<type>/<slug>"`)
4. If present: append content to `content.md` without overwriting
5. Log action to `maintenance-log.yaml`

**Durable insight extraction rules:**
- From root-cause investigations: hypothesis + fix approach from `context` field + session 1 body
- From evolve investigations: same — context + session 1
- From ponder entries: full scrapbook content (titles, artifacts)
- From published guidelines: full published file content

**NOT harvested:**
- Incomplete (`in_progress`) investigations
- Parked ponders
- Transient symptom details, specific file paths, raw exploratory dialogue (left to agent judgment during manual curation)

### Catalog Seeding (Step 5)

If `catalog.yaml` does not yet exist:
- Read ARCHITECTURE.md — extract `## Heading` lines (up to 7)
- Map headings to class codes `100`, `200`, … `700` (one per heading)
- Fallback defaults if fewer than 3 headings found:
  - 100: Architecture & Design
  - 200: Development
  - 300: Process
  - 400: Research
  - 500: Operations

The catalog is a no-op if the file already exists, so changing ARCHITECTURE.md after first init does not auto-update the catalog (requires manual `sdlc knowledge catalog add` or a second init after deleting the catalog).

### Librarian Agent File (Step 6)

Always written (overwrites) to `.claude/agents/knowledge-librarian.md`. Contains:
- Project name (derived from directory name or VISION.md title)
- Full catalog YAML embedded in template
- Core `sdlc knowledge` commands reference
- Maintenance protocol (classify uncategorized entries, fill summaries, publish)

### Cross-Reference Pass (Step 8)

For every pair of entries that share ≥2 tags: add each slug to the other's `related[]` list if not already present. Returns count of new links added.

## Output

### Human-readable (default)

```
Knowledge base initialized
  Investigations harvested: 3 (2 new, 1 updated)
  Ponders harvested:        2 (2 new, 0 updated)
  Guidelines linked:        1
  Catalog:                  5 classes (created: yes)
  Cross-references added:   4
  Librarian agent:          /path/to/.claude/agents/knowledge-librarian.md
```

### JSON (`--json`)

```json
{
  "investigations_new": 2,
  "investigations_updated": 1,
  "ponders_new": 2,
  "ponders_updated": 0,
  "guidelines": 1,
  "catalog_created": true,
  "catalog_class_count": 5,
  "cross_ref_count": 4,
  "agent_file": "/path/to/.claude/agents/knowledge-librarian.md"
}
```

## Failure Handling

Each step runs independently. A failure in harvesting one investigation does not abort the others. The function propagates `Result` — any IO error in catalog seeding or agent file writing returns an error to the caller. Partial runs are recoverable: re-running `librarian init` picks up where it left off because every step is idempotent.

## Data Contracts

### KnowledgeEntry fields set during harvest

| Field | Value |
|-------|-------|
| `slug` | `investigation-<slug>` or `ponder-<slug>` or `guideline-<slug>` |
| `origin` | `harvested` or `guideline` |
| `harvested_from` | `"investigation/<slug>"` or `"ponder/<slug>"` |
| `status` | `draft` |
| `tags` | includes kind string + `"investigation"` or `"ponder"` or `"guideline"` |

### LibrarianInitReport

```rust
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

## Non-Goals

- No LLM calls — all extraction is purely structural (fields + first session body)
- Does not manage or restart `sdlc ui`
- Does not validate or modify existing entries beyond appending content
- Does not delete entries

## Acceptance Criteria

1. Running on an empty project creates `.sdlc/knowledge/` and `catalog.yaml` with ≥1 class
2. Running on a project with completed investigations creates one entry per completed investigation
3. Running on a project with committed ponders creates one entry per committed ponder
4. Running on a project with a published guideline creates one entry with `origin: guideline`
5. Running twice on the same project produces the same result as running once (idempotency)
6. The librarian agent file is always (re)written and contains the project name and catalog YAML
7. Cross-reference pass links entries sharing ≥2 tags; does not add duplicate links on second run
8. `--json` flag outputs valid JSON matching the schema above
9. Any step failure returns a non-zero exit code with an error message; previous steps are not rolled back
