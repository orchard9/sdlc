# Knowledge Base + Librarian — Complete Spec

## Problem

Projects accumulate institutional memory in the worst possible ways: Slack DMs,
people's heads, deprecated wikis, and investigation sessions nobody reads again.
The cost compounds: bad decisions repeat, onboarding takes weeks, the same
root cause gets rediscovered three times.

## Solution

A first-class knowledge system with an autonomous librarian agent that:
1. Indexes everything the project learns into a structured, queryable base
2. Harvests insights from workspaces automatically — maintenance is structurally forced
3. Answers natural-language questions across all project knowledge
4. Keeps knowledge fresh — detects stale sources, evolving catalog

---

## The Flywheel

The value is not linear — it compounds. Every investigation feeds the knowledge base.
Every knowledge entry makes the next investigation faster. Every query that finds a gap
initiates a research run that fills it. Over 6 months, the knowledge base becomes the
project's most valuable asset.

---

## Core Concepts

### Entries

Each entry is one classified piece of knowledge.

```
.sdlc/knowledge/<code>/<slug>/
├── entry.yaml
└── content.md
```

`entry.yaml`:
```yaml
code: "100.20"
title: "Hexagonal Architecture"
slug: "hexagonal-architecture"
status: draft | published
tags: [architecture, patterns]
summary: "Ports & adapters — decouple business logic from infrastructure."
sources:
  - type: web | local | manual | harvested
    url: "https://..."             # web sources
    path: "docs/arch.md"          # local sources
    workspace: "root-cause/auth"  # harvested sources
    captured_at: "2026-03-01T..."
related: ["100.20.2", "200.10.3"]
last_verified_at: "2026-03-01T..."
staleness_flags: []               # url_404 | code_ref_gone | superseded_by | aged_out
origin: manual | web | research | harvested
harvested_from: "investigation/auth-bug"  # workspace slug if harvested
created_at: "2026-03-01T..."
updated_at: "2026-03-01T..."
```

`content.md` is rich markdown — prose, code, diagrams. Agents append; never overwrite.

### The Catalog

Stored at `.sdlc/knowledge/catalog.yaml`. A three-level hierarchy:

```
<class>          e.g. 100
<class>.<div>    e.g. 100.20
<class>.<div>.<sec>  e.g. 100.20.3
```

**The catalog is emergent, not designed.** The default is sparse (5–7 project-derived
categories). The librarian proposes new categories when clusters form and merges thin ones.
Codes are stable references — renaming a category doesn't break existing entries.

---

## The Librarian

A project-generated Claude agent (`.claude/agents/knowledge-librarian.md`) with the
project's catalog and domain context baked in. Not generic — tailored to this project
by `sdlc knowledge librarian init`.

### Mode 1: Maintain (scheduled + hooks)

**Triggered by:**
- Workspace completion hooks (post-investigate-complete, post-ponder-commit)
- Weekly cron job
- Manual: `sdlc knowledge librarian run --mode maintain`

**What it does:**
1. URL health — fetch all web sources, flag 404s and permanent redirects
2. Code ref health — grep for referenced functions/types, flag if gone
3. Duplication detection — flag entries with same tags + code prefix + similar titles
4. Catalog fitness — suggest subdivisions for categories >10 entries; flag empty categories
5. Cross-ref suggestions — entries with similar tags that don't reference each other
6. Harvest pending — check for completed investigations not yet harvested, run harvest

**Write authority:** Autonomous. All writes committed to git with "librarian:" prefix.
Audit trail in `.sdlc/knowledge/maintenance-log.yaml`.

### Mode 2: Query (on-demand)

**Triggered by:**
- CLI: `sdlc knowledge ask "<question>"`
- UI: chat panel in KnowledgePage (Phase 3)

**What it does:**
1. Searches knowledge base (taxonomy + full-text)
2. Synthesizes answer from relevant entries
3. Cites which entries + codes it drew from
4. If question reveals a gap → proposes `sdlc knowledge research <topic>`

---

## Librarian Init Flow

`sdlc knowledge librarian init` on an existing project:

1. **Project scan** — Read VISION.md, ARCHITECTURE.md, CLAUDE.md, .sdlc/config.yaml
2. **Workspace harvest** — For each completed investigation and committed ponder:
   extract durable insights, create draft knowledge entries
3. **Guideline linking** — For each published guideline:
   create a knowledge entry referencing it (origin: guideline)
4. **Domain extraction** — From harvested entries, identify 5–7 knowledge domains
5. **Catalog generation** — Write `.sdlc/knowledge/catalog.yaml` with derived categories
6. **Librarian agent generation** — Write `.claude/agents/knowledge-librarian.md` with
   project name, full catalog, key architectural decisions baked in as context
7. **Hook registration:**
   - post-investigate-complete → `sdlc knowledge librarian harvest investigation <slug>`
   - post-ponder-commit → `sdlc knowledge librarian harvest ponder <slug>`
   - (Scheduled maintenance is handled by the orchestrator-tick; no cron entry needed here)
8. **First maintenance pass** — Cross-ref suggestions on newly seeded entries
9. **Report** — N entries across M categories created

---

## Harvest Protocol

`sdlc knowledge librarian harvest <type> <slug>`

The librarian reads the workspace asking: "What here is durable? What would still be
true in 6 months? What helps the next developer facing a similar situation?"

**From root-cause:** root cause hypothesis + fix approach → Failure Patterns / relevant domain
**From evolve:** lens scores + rationale, chosen evolution paths → System Health / Architecture
**From ponder commit:** ⚑ Decided entries, surfaced constraints → Decision Records

For each durable insight:
1. Check if matching entry exists (semantic: similar title + tags + code prefix)
2. If exists: append to content.md, update updated_at
3. If not: create new entry with librarian-derived classification code
4. Record harvested_from in entry metadata
5. Log to maintenance-log.yaml: what was harvested and why

**NOT harvested:** transient symptom details, specific file paths, exploratory dialogue,
raw survey output — only the durable knowledge.

---

## CLI Reference

```bash
# Librarian management
sdlc knowledge librarian init               # generate project-specific librarian + seed
sdlc knowledge librarian run                # run maintenance pass
sdlc knowledge librarian harvest <type> <slug>  # harvest from workspace (type: investigation|ponder)

# Query mode
sdlc knowledge ask "<question>"            # query the knowledge base

# Ingest
sdlc knowledge add --title "..." [--code "100.20"] \
  [--from-url <url>] [--from-file <path>] [--content "..."]
sdlc knowledge research "<topic>" [--code "100.20"]  # agent-driven research + index

# Browse + manage
sdlc knowledge list [--code-prefix "100"] [--tag <t>] [--status draft|published] [--json]
sdlc knowledge show <slug> [--json]
sdlc knowledge search "<query>" [--json]
sdlc knowledge update <slug> [--code "..."] [--status published] [--tag <t>] [--related <code>]

# Catalog
sdlc knowledge catalog show [--json]
sdlc knowledge catalog add --code "100.40" --name "..." [--description "..."]

# Sessions (for research mode)
sdlc knowledge session log <slug> [--file <path>]
sdlc knowledge session list <slug> [--json]
sdlc knowledge session read <slug> <number>
```

---

## REST API

```
GET  /api/knowledge/catalog           → taxonomy tree
GET  /api/knowledge                   → all entries (metadata only)
GET  /api/knowledge?code=100          → entries filtered by code prefix
POST /api/knowledge                   → create entry
GET  /api/knowledge/:slug             → entry detail + content
PUT  /api/knowledge/:slug             → update metadata
POST /api/knowledge/:slug/capture     → append content from source
POST /api/knowledge/:slug/research    → spawn research agent run
POST /api/knowledge/maintain          → spawn librarian maintenance run
POST /api/knowledge/harvest           → harvest from a workspace slug
GET  /api/knowledge/:slug/sessions    → session list
GET  /api/knowledge/:slug/sessions/:n → session content
```

---

## Rust Data Layer

New module: `crates/sdlc-core/src/knowledge.rs`

Reuses `workspace.rs` for sessions and artifact I/O. Adds:
- `KnowledgeEntry` struct
- `Source` struct + `SourceType` enum (web, local, manual, harvested, guideline)
- `Catalog` struct + `CatalogClass`, `CatalogDivision`, `CatalogSection`
- `MaintenanceLog` struct + `MaintenanceAction` enum
- `validate_code(code: &str)` → must match `^\d{3}(\.\d{1,2}(\.\d)?)?$`
- CRUD: `create`, `list`, `get`, `update`, `list_by_code_prefix`, `search`

Storage:
```
.sdlc/knowledge/
  catalog.yaml
  maintenance-log.yaml
  <code-dashed>-<slug>/      # 100-20-hexagonal-architecture/
    entry.yaml
    content.md
    sessions/                # only present for research-mode entries
      session-001.md
```

---

## Frontend: KnowledgePage

`frontend/src/pages/KnowledgePage.tsx` — three-pane layout:
- **Left pane:** catalog tree (expandable, click to filter) + search bar
- **Center pane:** entry list for selected class/division + staleness indicators
- **Right pane:** entry detail (content as rendered markdown, sources, related refs,
  staleness flags, [Research More] / [Harvest] action buttons)

Sidebar: `Library` icon (lucide), below Guidelines in Plan group.

No PhaseStrip — knowledge has no phases. Entries grow indefinitely.
Staleness flags shown as subtle warning badges on entries.

---

## Integration with Existing Systems

| System | Integration |
|---|---|
| `sdlc investigate` complete | Hook → librarian harvest investigation |
| `sdlc ponder commit` | Hook → librarian harvest ponder |
| Guideline publish | Hook → librarian harvest guideline |
| Advisory run | Top-N entries via tag match + keyword overlap scoring injected as context |
| `sdlc-init` | Calls `sdlc knowledge librarian init` automatically during project setup |
| `/sdlc-next` | Can include "check knowledge base" step before implementation |

---

## Slash Command

```
/sdlc-knowledge [topic or subcommand]
```

- No arg → catalog overview + status
- `<topic>` → query mode, answer from knowledge base
- `init` → initialize librarian
- `research <topic>` → active research + index
- `maintain` → run maintenance pass

---

## Phased Build

**Phase 1 — Core data layer + CLI**
Rust data model, catalog, CRUD, `sdlc knowledge add`, `sdlc knowledge list/show/search`.
Librarian init (project scan + catalog generation + agent file). Harvest protocol.
No agent-driven modes yet — just the plumbing.

**Phase 2 — Agent modes**
`sdlc knowledge ask` (query mode), `sdlc knowledge research` (active research with sessions),
`sdlc knowledge librarian run` (maintenance pass), harvest hooks wired to investigations.
`spawn_agent_run` for research and maintenance.

**Phase 3 — Frontend + advisory integration**
KnowledgePage (catalog browser, entry viewer). Advisory integration (knowledge as context).
Chat panel in KnowledgePage for exploratory queries.

---

## Scope: Project-level only (v1)

Knowledge is project-scoped, stored in `.sdlc/knowledge/`, committed to git.
User-level knowledge (patterns that span all projects) is a future phase.
Generic patterns are already on the internet — the value is in project-specific context.

---

## Resolved Decisions (Session 3)

1. **sdlc-init calls knowledge librarian init automatically** — no opt-in. Every project
   bootstraps the knowledge system from day one. Init must be resilient on empty projects
   (minimal catalog, graceful report when nothing to harvest).

2. **Advisory integration uses both approaches** — tag match + keyword overlap scoring (v1
   semantic proxy). Two-pass: (1) entries where tags intersect, (2) keyword overlap against
   entry summaries. Merge, deduplicate, inject top-N as context block. No external embeddings
   in v1; vector search is a future phase.

3. **No cron in the knowledge feature** — scheduled maintenance is delegated to the
   orchestrator-tick-cli. The knowledge feature only owns: post-workspace-complete harvest
   hooks. Orchestrator calls `POST /api/knowledge/maintain` on a tick.
