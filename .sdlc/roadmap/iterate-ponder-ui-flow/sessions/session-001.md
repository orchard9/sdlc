---
session: 1
timestamp: 2026-03-07T15:15:00Z
orientation:
  current: "Analysis complete — committed ponder UI dead-ends identified, implementation plan shaped"
  next: "Implement Phase 1: milestone links + prepare button for committed ponders"
  commit: "Phase 1 implemented and visually verified — committed ponders have forward motion"
---

**Xist · Owner**
iterate ponder ui flow

after successfully committing a feature the user may be on a url like http://localhost:7777/ponder/frontend-port-8881

at the top is a status saying committed. add a new run action button to committed ponder sessions that are in a state where they can be run.

identify other ponder states that can be represented by the ui and suggest other possible improvements to the status, action buttons or other ui in general.

---

**Facilitator**

Studied `PonderPage.tsx`, `DialoguePanel.tsx`, `StatusBadge.tsx`, `AgentRunContext.tsx`, the Rust ponder model, and the roadmap API routes. Here's what I found:

## The Dead-End Problem

When a ponder reaches `committed` status, the UI currently:
- Hides the Commit button (correct — already committed)
- Hides the "Start from title & brief" zero-state button
- Shows a green "committed" badge
- ...and nothing else. No forward action.

The `committed_to: string[]` field on `PonderDetail` contains the milestone slugs that were created. This is the bridge to execution, but the UI doesn't surface it.

## Status Audit — All Four States

| Status | Current UI | What's Missing |
|--------|-----------|----------------|
| **exploring** | Commit button (muted), chat enabled, start session enabled | Could show session count encouragement |
| **converging** | Commit button (primary/prominent), chat enabled | Could highlight orientation commit signal more |
| **committed** | Green badge, no actions | **Milestone links, Prepare/Run button** |
| **parked** | Grey badge, no actions | **Resume button to re-explore** |

## Decisions

`⚑ Decided:` Committed ponders with `committed_to.length > 0` get:
1. A **milestone links section** below the header showing each milestone as a clickable link to `/milestones/{slug}`
2. A **"Prepare" action button** in the header that triggers `/api/milestone/{slug}/prepare` via `AgentRunContext.startRun`

`⚑ Decided:` Parked ponders get a "Resume" button that sets status back to `exploring`.

`⚑ Decided:` DialoguePanel empty state for committed entries should show milestone links instead of just hiding everything.

`? Open:` Multiple milestones — should the Prepare button prepare the first one, or show a dropdown? Leaning toward first milestone with a visual list for the rest.

`? Open:` Status progress indicator (exploring → converging → committed as horizontal steps) — nice to have, not blocking.

## Implementation Shape

Three files to touch:
1. `PonderPage.tsx` — `EntryDetailPane` header: add milestone links section + prepare button
2. `DialoguePanel.tsx` — update empty state for committed entries
3. Import `Play` from lucide-react, `Link` from react-router-dom

Uses `AgentRunContext.startRun` with `runType: 'milestone_prepare'` — same pattern as `WavePlan.tsx`.

See `implementation-plan.md` artifact for full code sketches.
