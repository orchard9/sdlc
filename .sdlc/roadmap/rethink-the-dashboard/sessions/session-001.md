---
session: 1
timestamp: 2026-03-02T09:00:00Z
orientation:
  current: "The dashboard's build-queue model is obsolete when the orchestrator runs everything. Three zones proposed: Shipped (momentum), Active (milestone-level progress), and Attention+Horizon (escalations + ponder intake). Core mental model: digest, not control panel."
  next: "Validate the three-zone layout with a prototype. Decide whether ponder/ideation belongs ON the dashboard or linked from it."
  commit: "When the zone structure is confirmed and the visual hierarchy of milestone-level (not feature-level) progress is agreed."
---

**jordan · Owner**
rethink the dashboard

we are soon going to be using the orchestrator for constant parallelized building. we do not need the dashboard to be focused on building

what should it be focused on?

whats been done, whats upcoming, what feedback to address, ideation about what to do next, things like that?

---

**Facilitator**

Let me pull up the current dashboard to anchor us. Right now it shows:

- Stats bar (feature counts by status)
- Escalations ("Needs Your Attention")
- PreparePanel (wave planning)
- HITL blocked features
- Active directives (in-flight `/sdlc-run` commands)
- Feature cards grouped by milestone in a 3-column grid
- CommandBlocks with `/sdlc-run <slug>` and `/sdlc-milestone-verify` calls
- Released archive (collapsed)

The whole surface is designed around one question: *what should I run next?*

But when the orchestrator is running features in parallel continuously, that question is already answered — the orchestrator is running everything. So what does a human need when they look at the dashboard?

Let me bring in the team.

---

**Ben Hartley · UX**

The issue isn't content — it's zoom level. The current dashboard is at zoom level 3: individual feature cards in a grid. When you have a dozen features running in parallel, zoom level 3 is visual noise. You can't see the shape of the project.

What you actually need when the orchestrator is doing the work is zoom level 1: milestones. "Where is this milestone?" not "what is this feature doing?"

The feature cards — the whole 3-column grid — should disappear from the default view. You drill into a milestone to see features. The dashboard shows milestones as a timeline or queue.

And the temporal dimension matters more now. When everything is parallelized, *how long has this been running* and *what completed recently* are the orienting signals, not *what action to take*. The dashboard should feel like a morning briefing, not a backlog manager.

**?  Open:** Should milestones be shown as a list, a timeline, or a kanban-style pipeline (active → releasing → shipped)?

---

**Dana Cho · Product Skeptic**

Before we redesign the whole thing — let's be precise about what Jordan actually needs.

He said: what's been done, what's upcoming, feedback to address, ideation about what to do next.

That's four things. The current dashboard already handles one of them decently (active directives / what's running). It partially handles another (escalations = feedback to address). The real gaps are:

1. **Shipped** — the "Archive" exists but it's collapsed and hidden. It should be *front and center*. Shipped milestones are momentum. When you're running an autonomous system, the shipped list IS the progress report.

2. **Upcoming** — the roadmap queue. What milestone is ready to start next? What's the planned horizon? This is completely absent from the dashboard right now.

3. **Ideation intake** — I'd push back on this one. The Ponder pages already exist for this. Do we really want to cram ideation into the dashboard, or do we want a link to "5 active ponders, 2 ready to commit"? The dashboard should surface the *status* of ideation, not be the ideation surface itself.

⚑  Decided: Don't replicate the Ponder workspace on the dashboard. Surface ponder status (count, readiness) and link to /ponder. The dashboard is a digest surface, not the thinking surface.

---

**Felix Wagner · Tooling Architect**

The spine of the new dashboard should be a **milestone pipeline** — a queue of states:

```
[Pondering] → [Planned] → [Active] → [Releasing] → [Shipped]
```

This is the fundamental workflow state machine. The dashboard should make it visible. Today the whole thing is collapsed into a flat feature grid — you can't see the flow at all.

The minimum viable pipeline view:
- **Shipped (last 30 days)** — released milestones with date and feature count
- **Active** — milestones currently being built (with a progress indicator: N/M features done)
- **Queued** — planned milestones ready to start (orchestrator will pick these up)
- **Pondering** — active ponders that could become milestones

That's the core mental model. The escalation panel stays exactly as it is — it's already the right design.

What goes away:
- CommandBlocks (orchestrator handles these, not humans)
- Feature cards in the default view (drill into milestone → see features)
- PreparePanel (orchestrator handles wave planning — this is operational detail)

**?  Open:** The active milestones panel needs a "progress" representation. Is that a progress bar (N/M features), a phase distribution bar (how many features are in spec vs implementation vs review), or something else?

---

**Nadia Osei · Information Design**

The hierarchy Felix proposed is right. But the visual weight matters.

When I look at the current dashboard, the eye goes to: escalation warning (amber) → feature cards (big grid). That's the wrong priority order for a digest surface. For a morning briefing, your eye should land on:

1. **What just shipped** (celebration / momentum)
2. **What needs your attention** (escalations, blocked)
3. **What's in flight** (progress, health)
4. **What's coming next** (horizon)

That's the reading order for a digest. Currently it's inverted — blocked/active is at the top, shipped is buried at the bottom in a collapsed accordion.

For milestone progress, I'd argue for a **phase distribution bar** rather than a simple N/M counter. A bar that shows: `░░▓▓▓▓░░` where each segment is a feature colored by phase — this tells you *where* the work is jammed, not just how much is left. It's the same pattern Linear uses for cycle progress.

For shipped milestones, don't just show a count. Show: milestone title, date shipped, number of features. This is the "what's been done" answer — it should feel like a release log, not an archive.

**?  Open:** Does the shipped section need a velocity signal (e.g. "3 milestones shipped in the last 30 days") or is the list itself sufficient?

---

**Ben Hartley · UX**

On the active milestone progress representation: the phase distribution bar is correct, but it needs to answer a slightly different question than "how much is left."

The question the orchestrator-driven user is asking is: *is this milestone healthy?*

Healthy = features are moving through phases. Unhealthy = features are piling up in one phase (especially HITL or blocked).

So the bar should call out blocked/stuck specially — make blocked features amber or red. A milestone where 8/10 features are in implementation is fine. A milestone where 3 features have been HITL-blocked for 2 days is a problem.

Also: the CommandBlocks should go away entirely from the dashboard. If a human needs to run `/sdlc-run`, they can navigate to the feature. The dashboard is not a CLI launcher anymore.

---

**Dana Cho · Product Skeptic**

Let's settle the ponder question definitively because it's the hardest scope boundary.

Jordan said "ideation about what to do next." The question is: what does that mean on the dashboard?

Option A: Full ponder panel — shows active ponder entries, lets you start new ones
Option B: Ponder status strip — shows "N active pondering, M ready to commit" as a link row
Option C: Nothing — ponder is its own page, dashboard links to it in the sidebar

I'd argue Option B. A small strip at the bottom of the dashboard: "Ideation: 3 active · 2 converging — View workspace →". This surfaces the signal (things are being thought about, some are ready) without making the dashboard into the thinking surface. It's a 1-line component.

⚑  Decided: Ponder status strip (Option B) — not a full panel, not absent.

---

**Felix Wagner · Tooling Architect**

Here's my proposed zone layout for the redesigned dashboard, in visual order:

```
┌─────────────────────────────────────────────────────┐
│  Project header (name, version, description)         │
│  Stats strip: N milestones active · M shipped total  │
├─────────────────────────────────────────────────────┤
│  NEEDS ATTENTION (amber)                             │
│  Escalations + HITL blocked                         │
├─────────────────────────────────────────────────────┤
│  RECENTLY SHIPPED                                    │
│  Last 3-5 milestones with date + feature count      │
│  "View all →" collapses to archive                  │
├─────────────────────────────────────────────────────┤
│  IN FLIGHT                                           │
│  Active milestones with phase-distribution bar       │
│  Click → milestone detail                            │
├─────────────────────────────────────────────────────┤
│  HORIZON                                             │
│  Queued milestones (planned but not started)         │
├─────────────────────────────────────────────────────┤
│  IDEATION                                            │
│  "N pondering · M converging — View workspace →"    │
└─────────────────────────────────────────────────────┘
```

That's 5 zones. No feature cards in the default view. No CommandBlocks. No PreparePanel.

**?  Open:** Does the "Queued" (Horizon) section show just milestone titles + feature counts, or does it also show which milestone is "next" to start (i.e., which the orchestrator will pick up next)?

---

**Ben Hartley · UX**

One more thing: the active directives panel. Currently it shows individual `/sdlc-run <feature>` directives with timestamps. When the orchestrator is running 20 features in parallel, this list becomes noise — it's 20 lines of "implement_task · started 2 minutes ago."

Replace it with an **orchestrator health indicator**. Not a list — a signal:

- Green: orchestrator active, N features in parallel
- Amber: orchestrator active, M features stuck/HITL
- Gray: orchestrator idle

This is a chip in the header, not a panel. The detailed run list lives on the dedicated Runs or Network page.

⚑  Decided: Remove individual active-directive list from dashboard. Replace with orchestrator health chip in header/stats strip.

---

**Nadia Osei · Information Design**

The "Recently Shipped" section should be the emotional anchor of the page. It's the momentum signal. When you're running an autonomous build system, "we shipped 3 milestones this week" is the most important thing a product owner sees.

Make it feel like a changelog, not a list. Each shipped milestone: bold title, date, 1-line description from the milestone vision, feature count. Small green checkmark icon. No toggle, just show the last 3.

The whole page is answering: "Where is the project? Are we making progress? What do I need to handle? What's coming? What am I thinking about?" — in that order.

---

**Facilitator — Synthesis**

The session has converged on a clear direction.

**⚑  Decided: Core mental model shift**
The dashboard moves from **build control panel** → **project digest**. It answers: "Give me the state of the project in 30 seconds." The orchestrator handles all the building; the human uses the dashboard to orient, not to operate.

**⚑  Decided: Five zones, in reading order**
1. **Needs Attention** (escalations + HITL) — highest urgency first
2. **Recently Shipped** (last 3–5 milestones, changelog style) — momentum signal
3. **In Flight** (active milestones with phase-distribution health bars)
4. **Horizon** (queued/planned milestones)
5. **Ideation Strip** (ponder count + readiness chip → link to /ponder)

**⚑  Decided: Remove from dashboard**
- Feature cards grid (too granular — access via milestone drill-down)
- CommandBlocks (orchestrator issues commands, not humans)
- PreparePanel / wave planning (orchestrator manages this)
- Individual active-directive list (replace with orchestrator health chip)
- Archive section (fold into "Recently Shipped" with "View all →")

**⚑  Decided: New components needed**
- `MilestoneProgressBar` — phase-distribution bar, blocked features highlighted amber/red
- `ShippedMilestoneCard` — changelog-style card (title, date, vision excerpt, feature count)
- `OrchestratorHealthChip` — single status chip (active N parallel / idle / stuck)
- `PonderStatusStrip` — "N pondering · M converging → View workspace"
- `HorizonList` — queued milestones, simple list

**?  Open: Still unresolved**
- Should In Flight milestones show estimated completion (based on remaining features and velocity)?
- Does the Horizon section include just planned milestones, or also "ready to ponder → commit" pondering entries?
- What triggers the dashboard to feel "empty" post-orchestrator era? (When orchestrator has nothing to run and ponder queue is empty — the current DashboardEmptyState should change)
