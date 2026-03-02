# Plan: New User Experience Milestone

## Source Ponder
`new-user-mental-model` — crystallized from 2 sessions with Maya Goldberg (Onboarding Designer), Xist (First-Time User Persona), and Priya Chakraborty (Product Strategist).

## Core Diagnosis
This is a **product design problem**, not a UI or docs problem. The current UI teaches the wrong mental model ("task management tool") at the product level — affordances, entry points, and empty states all reinforce sequential task execution rather than parallel autonomous waves.

**Root signal:** Xist created 20+ features individually and ran them one by one because the UI afforded it. Run Wave was buried 4+ navigation levels deep. The aha moment was never designed — it happened by accident or by asking Jordan.

## Milestone

**Slug:** `v14-new-user-experience`
**Title:** New User Experience
**Vision:** A developer who installs SDLC for the first time understands within 5 minutes that it runs autonomous parallel agents — not a to-do list — and reaches their first Run Wave within 15 minutes, not 30+ minutes by accident.

## Features

### 1. `dashboard-empty-state` — Dashboard Empty State Redesign
**Interventions covered:** A3 (empty dashboard state) + C1 (rename "setup" to "context")

Replace the amber warning banner and empty zero-stats dashboard with an identity-forward welcome state.

**What to change:**
- Remove the amber "Project setup is incomplete" warning banner from Dashboard
- Add the one-sentence identity: "Describe what you're building. Agents build it in parallel waves — you check in on results."
- Add a "New Ponder" CTA as the primary first action button
- Rename all instances of "setup incomplete" → "agents need more context" throughout the UI
- When there are no milestones and no features: show identity sentence + "New Ponder" button, not zero-stats

### 2. `dashboard-milestone-wave` — Active Milestones and Run Wave on Dashboard
**Interventions covered:** B2 (milestone entry point on dashboard)

Surface active milestones with their current wave state and a direct "Run Wave" button on the main Dashboard, eliminating the 4-level navigation currently required.

**What to change:**
- Add an "Active Milestones" section to Dashboard (below or replacing zero-stats when present)
- Each milestone card shows: title, current wave number, feature count, wave status
- "Run Wave" button visible directly on the card when a wave is ready
- Navigate to milestone detail on card click (existing behavior)

### 3. `ponder-first-onboarding` — Ponder-First Entry Path for New Users
**Interventions covered:** A1 (replace setup wall) + C2 (Vision/Architecture subtitles)

Replace the Setup wall as the first-run experience. New users start with a Ponder, not a 4-step wizard.

**What to change:**
- Detect "new project" state (no milestones, no features, setup incomplete)
- On new project state: redirect to Ponder page with a "What are you building?" prompt instead of Setup wall
- Vision/Architecture pages get explanatory subtitles:
  - Vision: "What you're building and why — agents use this to make the right tradeoffs."
  - Architecture: "How it's built — agents use this to write code that fits the system."
- Vision/Architecture become editable post-hoc, not prerequisites

### 4. `pipeline-visibility` — Pipeline Visibility Indicator
**Interventions covered:** A2 (persistent pipeline indicator)

Add a persistent horizontal pipeline indicator visible on Dashboard and Milestones pages showing: `Ponder → Plan → Commit → Run Wave → Ship`.

**What to change:**
- Add a `PipelineIndicator` component to the Dashboard header area
- Five stages: Ponder, Plan, Commit, Run Wave, Ship — displayed as horizontal pills
- Current stage derived from project state (has ponders? has milestones? has wave runs?)
- Stages are clickable — navigate to the relevant page
- Lightweight: a status bar, not a tutorial

### 5. `wave-running-context` — Wave Running Context and Recovery Path
**Interventions covered:** C3 (fire-and-forget framing) + B3 (recovery for manual-feature users)

Two behavior changes: (1) Add "you don't need to watch" framing during wave runs. (2) Detect and surface a recovery path when users have many features but no milestone.

**What to change:**
- During an active wave run, add contextual copy near the live log: "Agents don't need you here. Results appear when they're done."
- Add a "Come back later" affordance (not a button with action — permission copy that normalizes not watching)
- Detect pattern: 5+ features with no milestone → show inline prompt: "You have N features without a milestone. Want to organize them into a wave plan? [ Organize into Milestone ]"
- Recovery path triggers `/sdlc-plan` or routes to milestone creation with features pre-selected

## Priority Order (Implementation Sequence)

1. `dashboard-empty-state` — Low effort, highest visibility, immediate fix for first impression
2. `dashboard-milestone-wave` — Low effort, surfaces Run Wave without architecture change
3. `ponder-first-onboarding` — Medium effort, changes the first-run path entirely
4. `pipeline-visibility` — Medium effort, teaches the flow persistently
5. `wave-running-context` — Medium effort, serves both modes and adds recovery

## What This Is Not

This is not a documentation improvement or an onboarding wizard. These features change what the product does on first run, what affordances the UI exposes, and how the tool communicates its own identity. The goal is a designed aha moment at minute 12–15, not an accidental one at minute 30+ (or never).
