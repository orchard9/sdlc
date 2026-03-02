# Plan: Knowledge Base + Librarian

Source: ponder/knowledge-librarian

## Background

Projects accumulate institutional memory in the worst ways — Slack DMs, people's heads,
deprecated wikis. The cost compounds: decisions repeat, onboarding takes weeks, the same
root cause gets rediscovered three times.

The knowledge system solves this with a first-class indexed knowledge base (Dewey-inspired
classification, emergent catalog) and an autonomous librarian agent that harvests insights
from completed workspaces, detects stale entries, and answers natural-language questions.

The value is a flywheel: every investigation feeds the base; every entry makes the next
investigation faster; every gap query triggers research that fills it.

---

## Milestone 1: v10 — Knowledge base: capture and organize

**Vision:** A developer can add knowledge from any source (URL, local file, or written content),
browse it by taxonomy, and find it again. The librarian agent exists and can be initialized
for a new project — seeding entries from completed workspaces and generating a project-tailored
catalog and agent definition. No agent intelligence yet — just the plumbing.

**Acceptance test:** Run `sdlc knowledge librarian init` on this project. Verify it scans
VISION.md + completed investigations, produces a catalog.yaml, creates a librarian agent file,
and creates initial knowledge entries. Run `sdlc knowledge list` and see entries. Run
`sdlc knowledge search "agent"` and get relevant results.

### Features

**knowledge-core-data** — Rust knowledge.rs module
Core data model in `crates/sdlc-core/src/knowledge.rs`. Reuses `workspace.rs` for sessions.
- `KnowledgeEntry` struct with all fields: code, title, slug, status, tags, summary, sources,
  related, last_verified_at, staleness_flags, origin, harvested_from, created_at, updated_at
- `Source` struct + `SourceType` enum (web, local, manual, harvested, guideline)
- `Catalog` struct + `CatalogClass`, `CatalogDivision`, `CatalogSection`
- `MaintenanceLog` struct + `MaintenanceAction` enum
- `validate_code(code: &str)` — must match `^\d{3}(\.\d{1,2}(\.\d)?)?$`
- CRUD: `create`, `list`, `get`, `update`, `list_by_code_prefix`, `full_text_search`
- Storage paths module entries: `.sdlc/knowledge/catalog.yaml`, `.sdlc/knowledge/maintenance-log.yaml`,
  `.sdlc/knowledge/<code-dashed>-<slug>/entry.yaml`, `.sdlc/knowledge/<code-dashed>-<slug>/content.md`

**knowledge-cli-ingest** — CLI commands + server CRUD routes
CLI in `crates/sdlc-cli/src/cmd/knowledge.rs`:
- `sdlc knowledge add --title "..." [--code "100.20"] [--from-url <url>] [--from-file <path>] [--content "..."]`
- `sdlc knowledge list [--code-prefix "100"] [--tag <t>] [--status draft|published] [--json]`
- `sdlc knowledge show <slug> [--json]`
- `sdlc knowledge search "<query>" [--json]`
- `sdlc knowledge update <slug> [--code ...] [--status published] [--tag <t>] [--related <code>]`
- `sdlc knowledge catalog show [--json]`
- `sdlc knowledge catalog add --code "100.40" --name "..." [--description "..."]`
- `sdlc knowledge session log/list/read <slug> ...` (delegates to workspace.rs)
Server routes in `crates/sdlc-server/src/routes/knowledge.rs`:
- `GET /api/knowledge/catalog`, `GET /api/knowledge`, `GET /api/knowledge?code=100`
- `POST /api/knowledge`, `GET /api/knowledge/:slug`, `PUT /api/knowledge/:slug`
- `POST /api/knowledge/:slug/capture`
- `GET /api/knowledge/:slug/sessions`, `GET /api/knowledge/:slug/sessions/:n`

**knowledge-librarian-init** — `sdlc knowledge librarian init` command
Command that bootstraps the knowledge system for a project:
1. Project scan — reads VISION.md, ARCHITECTURE.md, CLAUDE.md, .sdlc/config.yaml
2. Workspace harvest — for each completed investigation and committed ponder: extracts durable
   insights (root cause hypothesis, evolution paths, decided constraints) into draft entries
3. Guideline linking — for each published guideline: creates a knowledge entry referencing it
4. Domain extraction — from harvested entries, identifies 5–7 knowledge domains
5. Catalog generation — writes `.sdlc/knowledge/catalog.yaml` with derived categories
6. Librarian agent generation — writes `.claude/agents/knowledge-librarian.md` with project
   name, full catalog, and key architectural context baked in
7. Hook registration — documents harvest hooks in `.sdlc/config.yaml`
8. First pass — generates cross-ref suggestions on seeded entries
9. Summary report — N entries across M categories created

---

## Milestone 2: v11 — Knowledge base: the active librarian

**Vision:** The librarian runs autonomously. It harvests completed investigations into the
knowledge base without any human action. A developer can ask the knowledge base a natural-
language question and get an answer with citations. Maintenance runs detect stale sources,
near-duplicates, and catalog drift. The knowledge base grows on its own.

**Acceptance test:** Complete a root-cause investigation. Verify the librarian harvest hook
fires automatically and creates/updates knowledge entries without manual invocation.
Run `sdlc knowledge ask "how does the SSE event system work?"` and verify a synthesized
answer with entry citations is returned. Run `sdlc knowledge librarian run` and verify
a maintenance report is produced with staleness flags and cross-ref suggestions.

### Features

**knowledge-query-mode** — `sdlc knowledge ask` + server endpoint
CLI: `sdlc knowledge ask "<question>"` — invokes librarian agent via spawn_agent_run with
question + full knowledge base as context. Returns synthesized answer with cited entry codes.
If a gap is detected, suggests `sdlc knowledge research <topic>`.
Server: `POST /api/knowledge/ask` → spawns agent run, streams via SSE.
SSE events: `KnowledgeQueryStarted`, `KnowledgeQueryCompleted { answer, cited_entries }`.

**knowledge-research-mode** — `sdlc knowledge research` with sessions
CLI: `sdlc knowledge research "<topic>" [--code "100.20"]` — agent-driven research:
multi-source web search + local file scan, synthesizes into a comprehensive entry,
logs research sessions. Uses spawn_agent_run with web search tools.
Server: `POST /api/knowledge/:slug/research` → spawns research agent run.
SSE: `KnowledgeResearchStarted { slug }`, `KnowledgeResearchCompleted { slug }`.

**knowledge-librarian-maintain** — maintenance pass + harvest hooks wired
CLI: `sdlc knowledge librarian run [--mode maintain|harvest]`
Maintenance mode (spawn_agent_run):
- URL health: fetch web sources, flag 404s
- Code ref health: grep for referenced symbols, flag if gone
- Duplication detection: flag same-tags + code-prefix + similar-title entries
- Catalog fitness: suggest subdivisions >10 entries, flag empty categories
- Cross-ref suggestions: entries with similar tags lacking mutual refs
- Harvest pending: find completed workspaces not yet harvested, run harvest
All writes autonomous, logged to `.sdlc/knowledge/maintenance-log.yaml`, committed to git.
Hooks: wire `post-investigate-complete` and `post-ponder-commit` in the investigation and
ponder CLI commands to invoke `sdlc knowledge librarian harvest`.
Server: `POST /api/knowledge/maintain`, `POST /api/knowledge/harvest`.
SSE: `KnowledgeMaintenanceStarted`, `KnowledgeMaintenanceCompleted { actions_taken }`.

---

## Milestone 3: v12 — Knowledge base: UI and integrations

**Vision:** The knowledge base is browsable in the sdlc UI. A developer can navigate the
catalog tree, view entry details with rendered markdown, see staleness warnings, and ask
questions in a chat panel. Advisory runs draw from the knowledge base as context. The
`/sdlc-knowledge` slash command is available across all supported AI CLIs.

**Acceptance test:** Open the sdlc UI. Navigate to the Knowledge section. Browse the catalog
tree. Click an entry and see its content rendered as markdown with source provenance and
related entry links. Run an advisory analysis and verify a "Knowledge context" section
appears in the output citing relevant entries. Run `/sdlc-knowledge "what is the agent
pattern for spawn_agent_run?"` and receive a synthesized answer.

### Features

**knowledge-page-ui** — KnowledgePage three-pane UI
`frontend/src/pages/KnowledgePage.tsx` — three-pane layout:
- Left pane: catalog tree (expandable, click to filter) + search bar
- Center pane: entry list for selected class/division + staleness badges (url_404, aged_out, etc.)
- Right pane: entry detail — content rendered as markdown, source provenance footer,
  related entry links (clickable by code), [Research More] action button
Sidebar: `Library` icon (lucide), below Guidelines in Plan group.
Bottom tab bar: Knowledge in Plan tab roots.
Routes: `/knowledge`, `/knowledge/:slug`.
No PhaseStrip — entries have no phases. SSE listener for KnowledgeResearchCompleted to refresh.

**knowledge-advisory-integration** — inject knowledge context into advisory runs
Before an advisory agent run starts, query the knowledge base for relevant entries
(top-N by tag match + recency). Inject as a "Project Knowledge" context block in the
advisory prompt. This prevents the advisory system from re-discovering known patterns.
Server: modify `routes/advisory.rs` to call knowledge CRUD before building advisory prompt.

**knowledge-slash-command** — `/sdlc-knowledge` in init.rs templates
Add `SDLC_KNOWLEDGE_COMMAND` (Claude Code), `SDLC_KNOWLEDGE_PLAYBOOK` (Gemini/OpenCode),
`SDLC_KNOWLEDGE_SKILL` (Agents) constants to `crates/sdlc-cli/src/cmd/init.rs`.
Command behavior:
- No arg → show catalog overview and entry count by category
- `<topic>` → query mode: answer from knowledge base with citations
- `init` → run knowledge librarian init
- `research <topic>` → active research + index
- `maintain` → run maintenance pass
Register in all four `write_user_*` functions. Add to `migrate_legacy_project_scaffolding`.
Update GUIDANCE_MD_CONTENT CLI reference table with `sdlc knowledge *` commands.
