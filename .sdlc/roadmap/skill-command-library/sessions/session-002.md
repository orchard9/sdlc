---
session: 2
timestamp: 2026-03-02T08:00:00Z
orientation:
  current: "Architecture and content decisions made — commands page has a clear spec ready to build"
  next: "Create feature `commands-docs-page` and implement: CommandMeta in server, GET /api/commands, CommandsPage in frontend with category grouping"
  commit: "Concrete implementation spec written — feature can be created and driven to completion"
---

**jordan · Owner**
we want to add ALL commands to our /docs/commands page, follow recommendations based on convo

---

## Session 2: Implementation Decisions

### Context restored from Session 1

Session 1 surfaced the core signal: Xist (early user) couldn't figure out _when_ to use each command. The problem wasn't that commands were hidden — they're autocomplete-discoverable in Claude Code. The problem is that "what it does" descriptions don't answer "should I use this right now?"

Key recommendations carried forward:
- Frame commands by **intent** (when/why), not just function (what)
- A **browseable catalog** is the right surface — not a README table
- "When to use" examples are the bridge between command discovery and actual use

### What we're building

The `/docs/commands` section in the web UI currently shows a placeholder. Jordan wants ALL 34 commands listed there, following the prior session recommendations.

Full inventory:
- **34 commands** across 8 functional categories
- Each has: `slug`, `gemini_description` (1-liner), `opencode_hint` (argument shape)
- **5 commands missing gemini_description**: sdlc-convo-mine, sdlc-enterprise-readiness, sdlc-guideline, sdlc-init, sdlc-prepare

### Voices in this session

---

**Ben Hartley · Developer Productivity UX**

The previous session already nailed the diagnosis. What matters now is what goes _on each card_. My concern with command reference pages: they always end up as a wall of text nobody reads.

The cognitive load question: what does a developer actually need at the moment they're choosing a command? Three things — name (so they can type it), the trigger (when do I reach for this?), and the argument shape (what do I pass it?). That's it. The card should be three lines, scannable, copy-on-click.

Don't try to embed the full prompt content in the UI. Link to it if anything. The descriptions we have are already good — "Autonomously drive a feature to completion" tells you exactly when to use `/sdlc-run`. The problem is you can't _browse_ them right now. Fixing that is 80% of the value.

**⚑ Decided: Card anatomy** — name + 1-line description + argument hint. Copyable command. No full prompt preview in V1.

---

**Felix Wagner · Developer Tooling Architect**

The data model question: where does the API get this data from?

Looking at the codebase: `ALL_COMMANDS` lives in `sdlc-cli/src/cmd/init/commands/`, which is a binary crate. The server can't import it. We have three choices:

1. Move `CommandDef` + `ALL_COMMANDS` to `sdlc-core` — correct long-term, non-trivial refactor
2. Static `CommandMeta` array in `sdlc-server` — minimal duplication, no restructuring needed
3. Build-time generation — too clever, creates a circular dep

For a docs feature, option 2 wins. 34 entries × 5 fields is two screens of code. We're not building a dynamic catalog with runtime registration — these commands change only with releases. A `COMMANDS_CATALOG: &[CommandMeta]` array in the server is honest about that. If it ever diverges from reality, a test catches it.

**⚑ Decided: Data source** — static `CommandMeta` array in sdlc-server, exposed as `GET /api/commands`. No crate restructuring for V1. Follow-up task: consolidate to sdlc-core in a later cleanup.

---

**Ben Hartley**

On categorization — this is where it gets interesting. The current command list has an implicit taxonomy. Explicit categories:

| Category | Commands |
|---|---|
| **Onboarding** | sdlc-init, sdlc-specialize |
| **Ideation** | sdlc-ponder, sdlc-ponder-commit, sdlc-suggest, sdlc-empathy, sdlc-recruit, sdlc-convo-mine, sdlc-guideline |
| **Execution** | sdlc-run, sdlc-next, sdlc-approve, sdlc-status |
| **Planning** | sdlc-plan, sdlc-prepare, sdlc-run-wave, sdlc-pressure-test |
| **Quality** | sdlc-milestone-uat, sdlc-cookbook, sdlc-cookbook-run, sdlc-quality-fix, sdlc-setup-quality-gates, sdlc-enterprise-readiness, sdlc-tool-audit, sdlc-tool-uat |
| **Tooling** | sdlc-tool-build, sdlc-tool-run, sdlc-skill-build |
| **Adjustment** | sdlc-vision-adjustment, sdlc-architecture-adjustment |
| **Research** | sdlc-spike, sdlc-hypothetical-planning, sdlc-hypothetical-do |

That's 8 categories. The UI renders them as tabs or accordion sections.

**? Open: Category for `sdlc-convo-mine`** — it's a signal extraction tool (discovery → ponder). It fits Ideation but could also be Onboarding. Lean toward Ideation since it's a recurring workflow, not a setup step.

**⚑ Decided: 8 categories** as above. `sdlc-convo-mine` → Ideation.

---

**Felix Wagner**

One thing I want to flag: 5 commands have empty `gemini_description`. We need to fill these before the page goes live, otherwise we're shipping blank cards. Proposed values:

- `sdlc-convo-mine` → "Mine conversations for actionable signal and launch ponder sessions per theme group"
- `sdlc-enterprise-readiness` → "Analyze project for enterprise-grade production readiness and distribute findings into milestones"
- `sdlc-guideline` → "Build an evidence-backed guideline from five research lenses and real codebase evidence"
- `sdlc-init` → "Bootstrap vision, architecture, config, and team through a conversational interview"
- `sdlc-prepare` → "Pre-flight a milestone — align features with vision, fix gaps, write wave plan, mark ready"

These should be set in the source files too, not just patched in the server catalog. Part of the implementation task.

**⚑ Decided: Fix missing descriptions** — fill in gemini_description in source for all 5 commands as part of the implementation task.

---

**Ben Hartley**

On "when to use" blurbs from Session 1 recommendation — I want to push back on scope creep here.

The `gemini_description` field already functions as a trigger condition for most commands. "Autonomously drive a feature to completion" tells you exactly when to pick `/sdlc-run`. "Suggest what to ponder next based on current project state" — that's already "when to use" framing.

The gap isn't _missing trigger conditions_ — it's that you can't browse them. Adding explicit "when to use" blurbs would help for the few commands where the description is ambiguous (sdlc-approve, sdlc-hypothetical-planning, sdlc-hypothetical-do). But making that a blocker for V1 is wrong.

Ship the catalog with existing descriptions. Add a follow-up task: "improve 'when to use' framing for power-user commands". Don't block.

**⚑ Decided: Use existing descriptions for V1** — no new "when to use" blurbs required before ship. Follow-up task to improve ambiguous entries.

---

**Felix Wagner**

Frontend implementation plan:

1. `GET /api/commands` — returns `Vec<CommandMeta>` where `CommandMeta` is:
   ```rust
   pub struct CommandMeta {
       pub slug: &'static str,
       pub description: &'static str,
       pub hint: &'static str,
       pub category: &'static str,
   }
   ```
   All 34 commands. Static array, no DB.

2. `useCommands()` hook — fetch + cache

3. `CommandsPage` component:
   - Tab bar: All | Onboarding | Ideation | Execution | Planning | Quality | Tooling | Adjustment | Research
   - Each command renders as a card: `/slug` (copy button) + description + hint chip
   - Optional: search/filter by name (V1 nice-to-have if small)

4. Wire up in DocsPage where `section === 'commands'` renders `<CommandsPage />`

The DocsPage currently renders a placeholder div — swap it for the real component.

**⚑ Decided: Implementation sequence** — CommandMeta struct → static array (with categories) → server route → frontend hook → CommandsPage component → DocsPage wire-up.

---

### Summary of decisions

| Decision | Outcome |
|---|---|
| Data source | Static `CommandMeta` array in sdlc-server, `GET /api/commands` |
| Card anatomy | Command name (copy) + 1-line description + argument hint |
| Categories | 8 groups: Onboarding, Ideation, Execution, Planning, Quality, Tooling, Adjustment, Research |
| "When to use" blurbs | Deferred — existing descriptions cover most cases; improve ambiguous ones as follow-up |
| Missing descriptions | Fill in 5 empty `gemini_description` fields as part of this task |
| Frontend | CommandsPage with tab-based category filter, wired into DocsPage |

### Implementation scope

Small, clean feature: one new server struct + static array (~100 lines), one new route (~20 lines), one new frontend component (~150 lines), DocsPage wire-up. No state machine changes. No database. No migration.

⚑ **Commit signal met.** The spec is clear enough to create a feature and drive it to completion.
