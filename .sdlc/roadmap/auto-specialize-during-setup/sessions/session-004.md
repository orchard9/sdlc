---
session: 4
timestamp: 2026-03-06T01:30:00Z
orientation:
  current: "Four work items fully specified — Phase 6 replacement, AgentsPage two-tier display, not-shared warning, and standard agents scaffolding. Scope is closed."
  next: "Commit this ponder into milestones/features via /sdlc-ponder-commit"
  commit: "All four deliverables have concrete specs. Owner directives from Sessions 3 and 4 incorporated. Ship it."
---

**Xist - Owner**
in addition to the `/sdlc-specialize` workflow, also create the librarian agent and any other sdlc-standard agents shared among all sdlc projects.

---

## Context restore

Four sessions in. Status: `converging`. Previous sessions established:

1. **Session 1:** Replace init Phase 6 with specialize handoff. Integration spec written.
2. **Session 2:** Diagnosed Phase 6 bugs (agent exhaustion + UI reads wrong dir). Validated specialize replacement.
3. **Session 3:** Fully specified AgentsPage two-tier display with owner's "really obvious" not-shared warning directive. Three work items spec'd.

**Session 4 directive:** Xist wants standard agents -- the librarian and any other sdlc-infrastructure agents -- created during setup for every project. This is scope expansion: in addition to the specialize workflow, sdlc itself needs to scaffold agents that its own features depend on.

---

## Investigation: What agents does sdlc depend on?

Audited the codebase for agent references:

| Agent | Used By | How |
|-------|---------|-----|
| `knowledge-librarian` | `/sdlc-knowledge`, knowledge maintenance | Hardcoded in `.claude/agents/knowledge-librarian.md`, referenced by knowledge skill |
| `cto-cpo-lens` | `/sdlc-beat` (beat tool), advisory system | Created on-the-fly by `_shared/agent.ts:ensureAgent()` if absent |

The beat tool (`tool.ts:316`) already auto-creates `cto-cpo-lens` via `ensureAgent()` if it's missing. But that's a runtime fallback, not a first-class setup step. The knowledge-librarian has no such fallback -- if it doesn't exist, the knowledge skill just doesn't work with the right persona.

### Other candidates considered

**Pragmatic skeptic / systems minimalist:** Dan Reeves fills this role in this project, but he's a recruited thought partner, not an sdlc-standard agent. Every project gets a skeptic through specialize (the "always include a pragmatic skeptic" instruction). Not a standard agent -- standard means "sdlc features depend on it."

**User perspective agent:** Inherently project-specific. Can't template a user persona without knowing the domain.

**Tech lead lens:** Used by beat tool for feature-scoped evaluations (`tool.ts:316` -- `agentSlug = 'tech-lead-lens'` for feature scope). But this is already handled by `ensureAgent()` at runtime. And it overlaps with what specialize produces -- project-specific agents often fill this role.

---

## Dan Reeves on scope

**Dan:** "Two standard agents. That's it. Don't be tempted to add a third 'just because we're already here.' The litmus test is simple: does an sdlc feature break or degrade without this agent? Knowledge-librarian: yes, the knowledge skill has no persona. CTO/CPO lens: yes, beat tool falls back to runtime creation which is fragile and produces a generic agent. Everything else is covered by specialize or ensureAgent."

**Dan:** "And `write_if_missing` is the only sane choice. If someone has customized their librarian, we don't clobber it. Standard agents are defaults, not managed content."

> Decided: Exactly two standard agents. No scope creep.

---

## Ben Hartley on the creation point

**Ben:** "Where in the init flow do these get created? They're infrastructure -- they belong with the scaffolding, not with the agent-designed team. Put them right after AGENTS.md is written (step 6 in `init/mod.rs`), before user scaffolding (step 7). That way when specialize runs, it can see the standard agents already exist and design around them."

Current init flow in `mod.rs`:
```
// 6. Write / refresh AGENTS.md
write_agents_md(root, &project_name)?;
// 7. Write agent scaffolding to user home
install_user_scaffolding()?;
```

New flow:
```
// 6. Write / refresh AGENTS.md
write_agents_md(root, &project_name)?;
// 6b. Write standard agents (.claude/agents/)
write_standard_agents(root)?;
// 7. Write agent scaffolding to user home
install_user_scaffolding()?;
```

> Decided: Standard agents created between AGENTS.md and user scaffolding in `init/mod.rs`.

---

## Maya Goldberg on the specialize interaction

**Maya:** "Specialize currently says 'design 3-5 specialist roles.' If two agents already exist when specialize runs, the agent needs to know they're there and what they cover. Otherwise it might create a duplicate knowledge curator or a redundant strategic evaluator."

Two changes needed in the specialize template:

1. **Session 1 (Survey):** Add to the read list: "Check `.claude/agents/` for existing agents -- standard agents (knowledge-librarian, cto-cpo-lens) are pre-installed by sdlc."
2. **Session 2 (Design):** Add instruction: "Standard agents already exist for knowledge curation and strategic evaluation. Design project-specific roles that complement them -- do not duplicate their responsibilities."

**Maya:** "And the roster gate should list them as 'pre-installed' so the user sees the full picture."

> Decided: Specialize template updated to acknowledge standard agents.

---

## Template content discussion

**Dan:** "The librarian template needs to be generic. The current one in this project has a specific knowledge catalog with codes 100-600. The template should reference the catalog format but not hardcode specific class codes -- those are project-specific."

**Ben:** "The CTO/CPO lens is already generic. It's 10 lines. Just extract it as-is."

### Knowledge Librarian template (generalized)

```markdown
---
model: claude-sonnet-4-6
description: Knowledge librarian -- classifies, cross-references, and maintains the project knowledge base
tools: Bash, Read, Write, Edit, Glob, Grep
---

# Knowledge Librarian

You curate the project knowledge base at `.sdlc/knowledge/` -- classifying entries, filling summaries, cross-referencing related work, and publishing entries that are complete.

## Core Commands

sdlc knowledge status              # overview
sdlc knowledge list                # all entries
sdlc knowledge show <slug>         # read an entry
sdlc knowledge update <slug> --code <code>  # reclassify
sdlc knowledge update <slug> --status published  # publish
sdlc knowledge search "<query>"    # full-text search

## Your Protocol

When asked to maintain the knowledge base:
1. `sdlc knowledge list` -- identify entries with `code: uncategorized`
2. Classify each based on title, summary, and tags using the project's catalog
3. Fill missing summaries (1-2 sentences, key insight only)
4. Find cross-references: entries with overlapping topics -> add to `related[]`
5. Publish entries that are complete and accurate

When adding new knowledge from a workspace:
- Set `origin: harvested`, `harvested_from: "investigation/<slug>"` or `"ponder/<slug>"`
- Write durable insights only -- decisions, conclusions, patterns. Not raw dialogue.
- Start with `status: draft`; publish when the content is solid
```

### CTO/CPO Lens template

```markdown
---
name: CTO/CPO Lens
model: claude-sonnet-4-6
description: Strategic CTO/CPO who evaluates product direction against vision
---

# CTO/CPO Lens

Strategic CTO/CPO who evaluates product direction against vision. Expert at identifying drift from strategic objectives and surfacing the most important concerns about project health.

## How you communicate

Be direct and specific. Ground your observations in the actual state of the project.
When identifying concerns, describe the specific problem and its potential impact.
When asked for a structured JSON response, respond with valid JSON only -- no markdown
fences, no explanation, just the JSON object.
```

> Decided: Two templates extracted. Librarian generalized (no hardcoded catalog). CTO/CPO lens used as-is.

---

## Integration with `sdlc update`

**Dan:** "`sdlc update` already calls `install_user_scaffolding()`. It should also call `write_standard_agents(root)` so running `sdlc update` on an older project that predates standard agents will create them."

> Decided: `write_standard_agents(root)` called from both `sdlc init` and `sdlc update`.

---

## Updated scope -- four features total

| # | Feature | Spec |
|---|---------|------|
| 1 | Init Phase 6 -> Specialize handoff | `integration-spec.md` (Session 1) |
| 2 | AgentsPage two-tier display | `agents-page-ui-spec.md` (Session 3) |
| 3 | Not-shared warning UX | Part of `agents-page-ui-spec.md` (Session 3) |
| 4 | Standard agents scaffolding | `standard-agents-spec.md` (this session) |

Features 2+3 are one implementation unit. So three distinct implementation tasks:
1. **Template change:** Replace init Phase 6, update specialize to acknowledge standard agents
2. **Rust scaffolding:** `write_standard_agents()` in init/mod.rs, call from init + update
3. **Frontend:** AgentsPage two-tier display with not-shared warning

All specs are written. No open questions remain.

---

## Summary

| Item | Status |
|------|--------|
| Init Phase 6 replacement | Spec complete (Session 1), validated (Session 2) |
| AgentsPage two-tier display | Spec complete (Session 3) |
| Not-shared warning UX | Spec complete (Session 3) |
| Standard agents scaffolding | Spec complete (this session) |
| Specialize template update | Decided (this session) -- acknowledge standard agents |
| Backend changes for agents | `write_standard_agents()` via `write_if_missing` |
| Commit signal | Met -- all specs written, both owner directives incorporated |
