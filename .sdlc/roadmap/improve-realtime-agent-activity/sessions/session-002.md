---
session: 2
timestamp: 2026-03-07T07:30:00Z
orientation:
  current: "All three changes scoped and decided — rich live renderer, navigation links, dead code cleanup. Pure frontend, no open blockers."
  next: "Commit to milestones — this is ready to execute."
  commit: "Owner confirmed direction ('yes to both'), added navigation link requirement, all changes scoped with zero open blockers."
---

**Xist · Owner**
yes to both. also add a link to the milestone or feature the agent is working on so users can quickly navigate from an agent activity tile to the detailed information about that agent run in the ui.

---

## Session 2 — Navigation Links + Final Scope

### Owner Feedback

Xist confirmed both prior decisions (swap AgentLog for RunActivityFeed, delete dead code) and added a new requirement: **each agent activity tile should link to the entity being worked on** — milestone, feature, ponder, or investigation detail page.

### Analysis: Navigation Links

Investigated `RunCard.tsx` and the frontend routing. The data is already there:

- `RunRecord` has `run_type` (feature, milestone_uat, ponder, etc.) and `target` (the slug)
- Frontend routes are clean: `/features/:slug`, `/milestones/:slug`, `/ponder/:slug`, `/investigations/:slug`

The mapping is a simple switch on `run_type`:

| run_type | Route |
|---|---|
| feature | `/features/{target}` |
| milestone_uat / milestone_prepare / milestone_run_wave | `/milestones/{target}` |
| ponder | `/ponder/{target}` |
| investigation | `/investigations/{target}` |
| vision_align / architecture_align | no link (project-level) |

**Implementation:** Add a `getTargetLink()` helper in `RunCard.tsx` and render the target as a react-router `<Link>` in the card header. Small, subtle — an arrow icon or just the slug text as a link. Zero backend changes.

### Final Scope (3 changes)

1. **Swap AgentLog for RunActivityFeed** on active runs — fix the type from `AgentEvent[]` to `RawRunEvent[]`, handle spawning state, add auto-scroll
2. **Add navigation links** from agent activity tiles to the target entity detail page
3. **Delete AgentLog.tsx + AgentEventLine.tsx** as dead code

All frontend-only. No backend changes. No new SSE events. No API changes.

### Decisions

- Decided: All three changes confirmed by owner
- Decided: Navigation link uses run_type to route mapping, rendered as subtle link in card header
- Decided: vision_align/architecture_align get no link (project-level, no detail page)
- Open: Unify AgentEvent/RawRunEvent types (cleanup, not blocking — can be done as follow-up)

### Commit Signal

Met. Owner confirmed direction. Problem diagnosed, solution designed with all three changes scoped. No open blockers. Ready to commit to milestones.
