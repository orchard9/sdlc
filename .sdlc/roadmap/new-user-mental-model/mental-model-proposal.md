# Mental Model Proposal: Teaching Ponder → Run Wave to New Users

## The Core Problem

SDLC's mental model is: **ideas → ponders → committed milestones → waves of parallel agents → shipped software**.

New users arrive with a different mental model: **plans → tasks → I execute each task (maybe with AI help)**.

These are not slightly different. They are fundamentally incompatible. A user operating from the second model will:
- Create features one by one (not in waves)
- Try to manually execute each feature (not fire and forget)
- Feel like the tool is a complicated to-do list (because that's what it looks like with that mental model)

No amount of better copy fixes this if the UI continues to afford the wrong mental model.

---

## Priya's Diagnosis: This Is a Product Design Problem, Not a UI Problem

Before proposing interventions, we must be honest about what kind of problem this is.

**UI problem:** User can't find the button. Fix: move the button.

**Conceptual problem:** User doesn't understand a term. Fix: better label or tooltip.

**Product design problem:** The product's initial experience teaches the wrong mental model, and the UI affords incorrect behavior. Fix: change the first-action path.

Xist's confusion is a product design problem. The dashboard's empty state, the feature creation flow, and the individual "Run" buttons all afford the wrong mental model — "manage tasks one by one." Run Wave is buried behind: setup → milestone creation → prepare → wave plan. That's four screens deep.

**Proposed interventions are organized by category. Implement category A before B before C.**

---

## Category A: Change the First Action (Product Design)

### A1: Replace the Setup Wall with a Ponder-First Entry

**Current:** New user sees "setup incomplete" → must write Vision + Architecture before anything works.

**Proposed:** New user sees an empty Ponder page with one prompt: "What are you building? Describe it in a sentence or two."

When they submit, the system:
1. Creates a Ponder from their text
2. Generates a draft Vision and Architecture in the background (two sentences each)
3. Shows them: "Here's what I understood. This gives agents enough context to start working."

**Why this order:** Ponder is creative and low-stakes. Vision/Architecture are formal and intimidating. Starting with Ponder lets the user *be in the tool* before setup feels like setup. The tool does the setup for them as a consequence of their first creative act.

**Implementation note:** The existing `/sdlc-init` command already does this conversationally. The web UI needs a first-run path that mirrors it. The Ponder page's "New idea" form should be the entry point for first-time users, not a Setup wizard.

---

### A2: Make the Pipeline Visible at All Times

**Current:** No persistent indication of where the user is in the ponder→plan→run-wave flow. Dashboard shows project stats. No flow indicator.

**Proposed:** A persistent horizontal pipeline indicator, visible on Dashboard and Milestones pages:

```
Ponder  →  Plan  →  Commit  →  Run Wave  →  Ship
  ●            ○         ○           ○          ○
```

The filled circle shows current stage. Stages are clickable — clicking "Ponder" goes to the Ponder page, clicking "Run Wave" goes to the Milestone Wave Plan.

**This single UI element would have told Xist everything he needed to know.** He would have seen "I'm in Ponder stage. There are four more stages. Run Wave is what I'm moving toward." He would have known the tool has a sequence.

**Constraint:** Keep it lightweight. A five-stage horizontal pill in the dashboard header. Not a full onboarding wizard. It should feel like a status bar, not a tutorial.

---

### A3: The Empty Dashboard State Must Say What the Tool Does

**Current empty state:** Dashboard shows a warning banner ("setup incomplete"), a stats bar with all zeros, and nothing else.

**Proposed empty state (for a project with no milestones):**

> "SDLC builds software through waves of parallel agents.
> You think in ideas — they think in work.
>
> Start by pondering an idea. When you're ready, commit it to a milestone and run the first wave."
>
> [ Start a Ponder ] [ See how it works ]

This is twelve seconds to read. It explains what the tool does, what the user should do next, and shows that there's a path forward. No warning. No obligation. No pressure.

---

## Category B: Reduce Time to Run Wave Discovery (Onboarding)

### B1: "Your First Wave" Celebration Screen

After a user runs their first wave (or after the first wave completes with results), show a one-time overlay:

> "Your first wave just ran. N features are being built in parallel.
>
> This is how SDLC works: you ponder, you commit, you run — then you check in on results.
>
> While this runs: ponder your next idea. Agents don't need you watching."

**Why:** The aha moment happens at first-wave-run. This overlay crystallizes that moment into explicit understanding. "You don't need to watch" directly addresses Xist's habit (and Jordan's point that he no longer needs to).

**Implementation:** One-time localStorage flag. Shows once, dismissible. Not a modal in the blocking sense — a slide-in panel at the bottom.

---

### B2: Milestone Entry Point Surfaced on Dashboard

**Current:** To find Run Wave, you need to: go to Milestones → click a milestone → scroll to Wave Plan.

**Proposed:** Active milestones appear on the Dashboard with their current wave state and a direct "Run Wave" button. No need to navigate.

Dashboard entry:

```
v0.1 — MVP                        Wave 1: 8 features ready
━━━━━━━━━━━━━━                     [ Run Wave ]
Prepare done
```

**Why:** The current dashboard already shows active features. It should also show active milestones with their wave state. This surfaces Run Wave at the top level without changing the underlying data model.

---

### B3: The "Jump In" Path for Existing Project Users

For Xist specifically: he already had 20+ features created manually. The tool should detect this and say:

> "You have 20+ features without a milestone. Want to organize them into a wave plan?
> Run `/sdlc-plan` or [ Organize into Milestone ]."

**Why:** Users who arrive at SDLC via word-of-mouth may start manually before they understand the flow. The tool should detect this pattern and offer a recovery path, not ignore it.

---

## Category C: Communication Layer (Conceptual Problem)

### C1: Rename "Setup" to "Context"

**Current label:** "Project setup is incomplete"

**Proposed label:** "Agents need more context to start"

The word "setup" implies the tool doesn't work yet. The word "context" implies the tool wants to do a good job and needs to understand the project. Small word change, meaningful reframing.

---

### C2: Vision/Architecture Pages Should Explain Why

**Current:** Vision page shows a text editor. Architecture page shows a text editor.

**Proposed:** Add a single-sentence subtitle beneath each heading:

- Vision: "What you're building and why — agents use this to make the right tradeoffs."
- Architecture: "How it's built — agents use this to write code that fits the system."

**Not a tooltip. Not documentation.** Just a subtitle that answers "why does this page exist?"

---

### C3: "Fire and Check In" Mode Indicator

The tool should make the fire-and-forget pattern explicit. When a wave is running:

```
Wave 1 running — 8 agents working
You don't need to stay. Results appear here when they're done.
[View Live Log]  [Come back later]
```

"Come back later" doesn't do anything — it's not a button with an action. It's permission. It tells Xist: "Jordan's behavior is correct. You can close this tab."

---

## The One Sentence

If we could put one sentence on the first screen a new user sees, it should be:

> **"Describe what you're building. Agents will build it in parallel waves — you check in on results."**

This sentence:
- Establishes the mental model (parallel waves, not sequential tasks)
- Sets the user's role (describe + check in, not manage + execute)
- Communicates the tool's capability ("agents will build it")
- Requires zero prior knowledge

Everything else in this proposal is elaboration on that sentence.

---

## Priority Order

| Priority | Intervention | Category | Effort | Impact |
|---|---|---|---|---|
| 1 | Replace setup wall with Ponder-first entry | A1 | Medium | Critical |
| 2 | Empty dashboard state redesign | A3 | Low | High |
| 3 | Pipeline visibility indicator | A2 | Medium | High |
| 4 | Active milestones + Run Wave on Dashboard | B2 | Low | High |
| 5 | "Your first wave" celebration screen | B1 | Low | Medium |
| 6 | Rename "setup" to "context" | C1 | Trivial | Medium |
| 7 | Vision/Architecture subtitles | C2 | Trivial | Medium |
| 8 | "Fire and check in" mode indicator | C3 | Low | Medium |
| 9 | "Jump In" recovery for manual-feature users | B3 | Medium | Medium |

**Start with A3 and C1/C2** — these are low effort and fix the communication failures immediately. Then implement A1, which requires rethinking the first-run path. A2 and B2 are the architectural changes that make the flow visible.

---

## What This Is Not

This proposal is not a documentation improvement. It does not add tooltips, walkthroughs, or onboarding wizards. It does not add a "getting started" page.

The interventions change *what the product does on first run*, *what affordances the UI exposes*, and *how the tool communicates its own identity*. These are product design changes, not UX decoration.

The goal is: a user who has never heard of SDLC, who installs it and opens the browser, understands the core loop (ponder → run wave → check in) within 10 minutes — not because they read documentation, but because the product's first-run experience teaches it.
