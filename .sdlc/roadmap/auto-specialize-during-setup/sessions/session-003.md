---
session: 3
timestamp: 2026-03-06T23:58:00Z
orientation:
  current: "All three work items fully specified — template replacement, AgentsPage two-tier display, and the not-shared warning. Ready to commit and implement."
  next: "Commit this ponder into milestones/features via /sdlc-ponder-commit"
  commit: "Owner directive received on UI design. Integration spec validated across two sessions. All three deliverables have concrete specs. Ship it."
---

**Xist · Owner**
when showing agents in the ui, show project-level agents as the primary. show user-workstation-level agents as a secondary list and make it really obvious to the developer that the user-workstation-level agents **are not** shared with other developers.

---

## Context restore

Three sessions in. Status: `converging`. Previous sessions established:

1. **Session 1:** Replace init Phase 6 with specialize handoff. Integration spec written.
2. **Session 2:** Diagnosed why Phase 6 is broken (agent exhaustion + UI reads wrong dir). Decided to split scope: template change = this ponder, UI fix = separate task. Chose Option B for UI: project agents primary, user agents secondary.

Xist's message today refines the Session 2 UI decision with a specific directive: make it **really obvious** that workstation agents are not shared.

## Codebase analysis

Investigated the current state to validate feasibility:

**AgentsPage.tsx (164 lines):** Currently fetches only `api.getAgents()` (user-level, `~/.claude/agents/`). Shows a flat list. No awareness of project agents at all. The `api.getProjectAgents()` endpoint exists and is already used by `SetupPage.tsx` — so the backend is ready.

**Server endpoints:** Both `/api/agents` (user) and `/api/project/agents` (project) exist in `crates/sdlc-server/src/routes/agents.rs`. Both return `AgentDefinition[]` with the same shape. No backend changes needed.

**SetupPage.tsx already does it right:** Lines 51-56 call `getProjectAgents()` to check if team setup is complete. This confirms the API works and returns data correctly.

---

## Ben Hartley on the visual hierarchy

**Ben:** "Xist said 'really obvious'. That means don't rely on a subtle label difference. The workstation section needs a visual break — a different container treatment, an explicit callout banner, something that interrupts the scanning pattern. A developer scrolling past should feel the boundary."

Concrete proposal:

**Project Team section** — normal card grid, full visual weight. Header says "Project Team" with a `Users` icon. Subtext references `.claude/agents/` and notes these are shared via git.

**Workstation Agents section** — visually demoted:
- Wrapped in a `border-dashed` container with reduced opacity
- Starts with a persistent info banner: amber/yellow accent, `Lock` or `UserX` icon
- Banner text: "These agents are on **your machine only** — not shared with other developers on this project."
- Cards render with the same `AgentCard` component but the container signals "different zone"

**Ben:** "The dashed border + info banner combo works because it's a pattern developers already know from 'local only' sections in dashboards. It says 'this is yours, not the project's' without being a modal or a warning."

> Decided: Dashed border container + persistent amber info banner for the workstation section. Project section uses solid borders and full visual weight.

---

## Maya Goldberg on the empty states

**Maya:** "What if someone has zero project agents but 5 workstation agents? The page shouldn't feel broken. The project section should have a clear call-to-action, and the workstation section should still show their agents."

Empty state matrix:

| Project agents | User agents | What to show |
|---|---|---|
| 0 | 0 | Single empty state: "Run `/sdlc-specialize` to create your project team" |
| 0 | N | Project: empty CTA ("No project team yet — run `/sdlc-specialize`"). User: show agents in secondary section |
| N | 0 | Project: show agents. User: small muted text "No workstation agents" (no big illustration) |
| N | M | Both sections populated |

**Maya:** "The project empty state should be the loudest element on the page when it's empty. The workstation empty state should be almost invisible — it's not a problem that needs solving."

> Decided: Asymmetric empty states. Project gets the big CTA with icon. Workstation gets a single line of muted text.

---

## Dan Reeves on naming

**Dan:** "Don't call it 'User Agents'. That's database-speak. Call it 'Your Workstation' — it maps to the mental model of 'this machine' vs 'this project'. And the project section should be 'Project Team' not 'Project Agents' — these are team members, not objects."

| Section | Label | Subtext |
|---|---|---|
| Primary | **Project Team** | "Shared with all developers via `.claude/agents/`" |
| Secondary | **Your Workstation** | "Local to this machine — not committed to git" |

> Decided: "Project Team" and "Your Workstation" as section headers.

---

## Tobias on scope adjustment

**Tobias:** "Session 2 said the UI fix is a *separate task*, not part of this ponder's commit. Xist's directive makes the UI work first-class. Include it."

Correct. The ponder now has two features, cleanly scoped:

1. **Feature: Init Phase 6 -> Specialize** (integration-spec.md): Replace init Phase 6 with specialize handoff — 4 platform variants in `sdlc_init.rs`
2. **Feature: AgentsPage Two-Tier Display** (agents-page-ui-spec.md): Project Team primary + Your Workstation secondary with not-shared warning

Both features belong in the same milestone. Both are fully specified.

---

## Final spec captured

Wrote `agents-page-ui-spec.md` to the scrapbook with:
- State management (two separate arrays, parallel fetch)
- Layout spec (two sections with visual hierarchy)
- Warning banner spec (amber, persistent, explicit text about not sharing)
- Empty state matrix (4 scenarios)
- Naming conventions ("Project Team" / "Your Workstation")

Combined with the existing `integration-spec.md` from Session 1, both features are fully specified.

---

## Summary

| Item | Status |
|------|--------|
| Init Phase 6 replacement | Spec complete (Session 1), validated (Session 2) |
| AgentsPage two-tier display | Spec complete (this session) |
| Not-shared warning UX | Spec complete (this session) |
| Backend changes | None needed — both endpoints exist |
| Commit signal | Met — all specs written, owner directive incorporated |
