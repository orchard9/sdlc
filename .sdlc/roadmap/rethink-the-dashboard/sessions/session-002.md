---
session: 2
timestamp: 2026-03-02T09:30:00Z
orientation:
  current: "Three open questions from session 1 are resolved. Horizon includes ponders. No velocity estimates. Empty state needs suggestions. Design work: how ponders and milestones coexist in Horizon, and what the suggestion engine looks like."
  next: "Capture Horizon zone design (with ponders) and empty state spec as scrapbook artifacts, then assess commit readiness."
  commit: "When Horizon zone layout (with ponder integration) is designed and the empty-state suggestion model is specified — the dashboard spec is complete enough to become a milestone."
---

**jordan · Owner**
"Should In Flight milestones show estimated completion (based on remaining features and velocity)?" - no
"Does the Horizon section include just planned milestones, or also 'ready to ponder → commit' pondering entries?" - include pondering
"What triggers the dashboard to feel 'empty' post-orchestrator era? (When orchestrator has nothing to run and ponder queue is empty — the current DashboardEmptyState should change)" - we need to fill it with suggestions when nothing is left to ponder or build

---

**Facilitator**

Three answers. Let's unpack each one before moving on.

**On velocity estimates — No.**
Good. The phase-distribution bar already tells you health (where things are stuck) without pretending the system can predict when work will complete. This simplifies the In Flight zone considerably.

**On Horizon — include pondering.**
This is the most structurally interesting answer. It means Horizon is no longer just a queue of planned milestones — it's the forward-looking view that spans from "concrete and ready to run" down to "idea being explored." This changes the Ideation Strip from session 1 — if ponders are in Horizon, what's left for the strip?

**On empty state — suggestions.**
This is a design problem we haven't touched yet. When the orchestrator is idle and there's nothing in Horizon, the system should surface suggestions for what to think about or build next. That's a content + intelligence problem.

Let's dig in.

---

**Felix Wagner · Tooling Architect**

On Horizon with ponders: the zone needs a spectrum, not a flat list. Think of it as "what's coming" ordered by concreteness:

```
HORIZON
  --- Planned milestones (slug exists, features defined, orchestrator can pick up)
  --- Converging ponders (status: converging — idea is shaped, ready for /sdlc-ponder-commit)
  --- Exploring ponders (status: exploring — still active ideation)
```

Visual differentiation matters here. Planned milestones are solid — they have a defined spec, feature count, slug. Ponders are softer — they have a title and status. So:

- Planned milestone: solid card, milestone icon (chevrons/layers), feature count badge, "Ready to run" or "Queued" label
- Converging ponder: dashed border, lightbulb icon, "Ready to commit ->" CTA in amber
- Exploring ponder: muted/ghost appearance, lightbulb icon, "Exploring" label, no CTA

The "Ready to commit" ponders should be visually prominent — amber dashed border — because they represent actionable next steps for the human. These are the "almost milestones" that need one decision to become real work.

**?  Open:** Do we show *all* exploring ponders in Horizon, or just the most recent N? If someone has 10 active ponders, showing all of them makes Horizon noisy.

---

**Dana Cho · Product Skeptic**

On the Ideation Strip from session 1: if ponders are now in Horizon, the strip becomes redundant for status. But it had a second job: being the entry point for new ponders.

I'd kill the strip as a standalone component. Instead, put a "New idea" button at the top of the Horizon section header. This collapses two things into one:

```
HORIZON                                          [+ New idea]
  Queued                    v14-new-user-exp  | 8 features
  Queued                    v15-layout-found  | 4 features
  Ready to commit --------- rethink-dashboard  [Commit ->]
  Exploring --------------- sdlc-fleet-deploy [Continue ->]
```

The "New idea" button routes to `/ponder` with a new entry dialog. The strip is gone. The status count (N pondering · M converging) gets folded into the Horizon section header label, e.g. "Horizon — 2 planned · 3 pondering."

**Decided:** Kill the standalone Ideation Strip. Merge into Horizon section with a [+ New idea] header button and a count in the section title. The Horizon zone is the single forward-looking surface.

---

**Nadia Osei · Information Design**

On the empty state — "suggestions when nothing is left to ponder or build." This is the hardest design problem because it requires the system to have an opinion about what matters.

Let me separate two different empty states:

**Empty State A: Nothing in flight, but Horizon has content.**
This is actually not empty — it's "the orchestrator is idle, but there's a queue." The action is "start the orchestrator." This is a different state from true empty.

**Empty State B: Nothing in flight AND Horizon is empty.**
This is true empty. The system has no milestones queued, no ponders active. This is the post-project feeling — "we're done, what's next?"

For State B, the suggestions need to be contextual. The system knows:
- What milestones have shipped (from `.sdlc/milestones/`)
- The project vision (from `VISION.md`)
- Whether there are any stuck features or filed tasks with no milestone

So the suggestions could be:
1. "You've shipped N milestones. Consider running `/sdlc-suggest` to surface what's next."
2. "N features are filed without a milestone — ready to group into a new milestone?"
3. "Your project vision mentions [X] — have you explored [related area]?"
4. A big "Start a new ponder ->" CTA with 2-3 suggested topics from `/sdlc-suggest`

**?  Open:** Is the suggestion content static (just CTAs) or dynamically generated by running `/sdlc-suggest` on dashboard load?

---

**Ben Hartley · UX**

The empty state question is really a question about **emotional design**. When someone looks at their project dashboard and sees nothing in flight and nothing queued, they feel one of two things:

1. **Accomplished**: "We shipped everything. We're done."
2. **Adrift**: "The system has nothing to do. What happens now?"

The design principle is: **the empty dashboard should feel like a clean desk, not a 404.**

For State B (true empty), I'd design three content blocks:

```
+-----------------------------------------------------------+
|  The project is caught up.                               |
|  N milestones shipped. Nothing queued.                   |
+-----------------------------------------------------------+
|  What to think about next:                              |
|  · [Suggestion from /sdlc-suggest, dynamic]             |
|  · [Suggestion from /sdlc-suggest, dynamic]             |
|  · [Suggestion from /sdlc-suggest, dynamic]             |
|                                                          |
|  [+ Start a new ponder]                                 |
+-----------------------------------------------------------+
|  Or look back:                                          |
|  · View shipped milestones                             |
|  · Review filed tasks without a milestone              |
+-----------------------------------------------------------+
```

The key insight: the suggestions don't have to be AI-generated on the fly. The `/sdlc-suggest` command already exists and already produces output. The dashboard can call it on load when Horizon is empty and render its output as suggestion chips. This is a data fetch, not an inference call.

**Decided:** Empty State B uses `/sdlc-suggest` output as suggestion chips. The dashboard calls the suggest endpoint when it detects Horizon is empty and renders the top 3 suggestions with CTAs to `/sdlc-ponder` or `/sdlc-plan`.

---

**Felix Wagner · Tooling Architect**

Before we go further on empty state: there's a product distinction that matters.

**State A (idle, queue not empty)**: This is the most common production state. The orchestrator ran everything it could, finished a wave, and is waiting. There are planned milestones in Horizon. The dashboard should say: "Ready. N milestones queued. Start the orchestrator." With a prominent "Run wave" button or at least instructions for the human to fire `/sdlc-run-wave`.

Wait — in the fully automated orchestrator era, does a human even need to trigger wave execution? Or does the orchestrator self-trigger?

This is actually a product assumption we need to surface. If the orchestrator is truly autonomous, State A doesn't need a CTA — it just shows the queue. If the human still triggers waves, State A needs a "Start next wave" button.

**?  Open:** In the orchestrator era, who triggers wave execution — a human or the orchestrator automatically?

---

**Dana Cho · Product Skeptic**

Felix, I think that's outside scope for this session. We're designing the dashboard UI — the orchestrator trigger model is an orchestrator feature, not a dashboard feature. The dashboard should show state, not decide policy.

What we can decide is: if Horizon has queued milestones and the orchestrator is idle, show a status banner that says "Orchestrator idle — N milestones queued." Whether there's a button in that banner depends on whether the orchestrator has an auto-trigger. Let's say the banner exists; button presence is conditional on the orchestrator API.

**Decided:** State A (Horizon not empty, orchestrator idle) shows an amber status banner: "Orchestrator idle — N milestones ready." Button presence TBD on orchestrator capability.

---

**Nadia Osei · Information Design**

Coming back to the Horizon ponder integration — Felix's spectrum model is right, but I want to nail the visual hierarchy.

The Horizon zone should be scannable in 3 seconds. If there are 8 ponders and 4 planned milestones, that's 12 items. That's too many for a flat list. We need grouping:

```
HORIZON                                          [+ New idea]
  2 planned · 3 pondering

  [QUEUED MILESTONES]
    v14-new-user-experience       8 features planned
    v15-layout-foundation         4 features planned

  [READY TO COMMIT]
    rethink-the-dashboard         [Commit ->]

  [EXPLORING]
    sdlc-fleet-deploy             [Continue ->]
    knowledge-graph               [Continue ->]
```

Three subgroups, each with a header. The visual symbols communicate concreteness level at a glance. "Ready to Commit" is amber — it's an action available to the human right now.

**Decided:** Horizon zone uses three subgroups with visual differentiation: Queued Milestones (concrete), Ready to Commit (amber, action), Exploring (muted).

---

**Ben Hartley · UX**

One last thing on the Horizon ponder question: should *parked* ponders appear in Horizon?

My vote: no. Parked means the human actively shelved the idea. Showing parked ponders in the forward-looking Horizon would be noise. If someone wants to revisit a parked ponder, they go to `/ponder` directly.

**Decided:** Horizon only shows status: `exploring` and `converging`. Parked ponders are excluded from Horizon.

---

**Facilitator — Session 2 Synthesis**

The three open questions are now resolved, and several new decisions have been made.

**Resolved: No velocity estimates on In Flight**
Phase-distribution bars already answer "is this healthy?" — estimated completion adds false precision.

**Resolved: Horizon includes ponders**
Horizon is the single forward-looking surface: Queued Milestones + Converging Ponders + Exploring Ponders. The standalone Ideation Strip from session 1 is eliminated — its count merges into the Horizon section header and its CTA becomes [+ New idea] button in the header.

**Resolved: Empty state surfaces suggestions**
When Horizon is empty and orchestrator is idle, the dashboard calls `/sdlc-suggest` and renders top 3 suggestions as ponder/plan CTAs. The empty state has three blocks: accomplishment signal, suggestions, look-back links.

**New decisions**

- Horizon subgroups: Queued Milestones / Ready to Commit (amber) / Exploring (muted)
- Parked ponders excluded from Horizon
- State A (idle, queued) shows amber banner: "Orchestrator idle — N milestones ready"
- Horizon section header: "[+ New idea]" button + "N planned · M pondering" count label

**Still open (for implementation, not design)**
- Who triggers wave execution in full orchestrator mode — determines whether State A banner has a button
- How many exploring ponders to show before truncating (suggest: cap at 3, "View all ->")

**Commit signal check:**
The zone structure is complete. Horizon design (with ponder integration) is specified. Empty state model is specified. Visual hierarchy and component breakdown are sufficient to write a spec and begin implementation.

Status: **converging**
