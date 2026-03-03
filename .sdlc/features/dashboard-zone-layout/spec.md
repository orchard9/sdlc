# Spec: Dashboard Four-Zone Layout

## Problem

The current Dashboard is a control panel: feature card grids nested inside milestone
sections, with stats, escalations, wave plans, and active directives all stacked
vertically in a single scrolling list. For a project with several milestones and
dozens of features, this layout requires significant scrolling and offers no mental
model — every section is equally prominent, and the feature card grid dominates the
visual weight.

The dashboard should be a **project digest** — an at-a-glance status page that shows
where the project stands, not a list of every feature card that happens to exist.

## Goal

Restructure the Dashboard into four visually distinct zones, each with a clear
information purpose. Replace the per-milestone feature card grids with compact
**milestone digest rows** that summarise milestone health at a glance.

## The Four Zones

### Zone 1 — Attention

Live items requiring human input or showing active agent work:
- Escalations (human-in-the-loop items from agents)
- HITL-blocked features (wait_for_approval, unblock_dependency)
- Active directives (features currently being worked on by an agent)
- "What Changed" banner (since-last-visit event summary)

This zone is collapsed or hidden when empty. It always appears at the top.

### Zone 2 — Current

The active milestone(s): what the project is working on right now.

Each milestone renders as a **MilestoneDigestRow** — a compact single-row summary:
- Milestone title (linked to `/milestones/<slug>`)
- Status badge
- Progress bar or fraction (X of Y features done)
- Current action label (the next action of the first non-done feature)
- Copy-ready command (`/sdlc-run <next-feature-slug>`)

No feature cards. The digest row expands on click to reveal a compact feature list
(one line per feature: title, phase badge, next action) — not the full FeatureCard.

Ungrouped features (not assigned to any milestone) appear below the milestones in
this zone, also as a compact list rather than a card grid.

### Zone 3 — Horizon

Forward-looking surface: upcoming milestones and active ponders. This zone is
implemented by the companion `dashboard-horizon-zone` feature. Zone 3 is a reserved
layout slot that renders a `<HorizonZone />` placeholder in this feature and becomes
populated once that feature ships.

### Zone 4 — Archive

Released milestones. Collapsed by default, expandable. Same compact list style as
Zone 2 — no feature cards, just milestone title + released badge.

## MilestoneDigestRow Component

```
┌─────────────────────────────────────────────────────────────────┐
│  ● v22 — Project Changelog   [in-progress]  ██████░░░░ 3/5      │
│  Next: implement_task · /sdlc-run changelog-core   [copy]       │
└─────────────────────────────────────────────────────────────────┘
```

- Single card, ~64px tall in collapsed state
- Expand chevron on right edge → reveals per-feature compact list
- Progress fraction and bar calculated from feature phases (done = released/merge phase, or next_action == done)
- "Next" row shows the command for the first actionable feature

## Behaviour

- Zone 1 is omitted from DOM when there is nothing to show (no escalations, no
  active directives, no HITL items, no recent changes)
- Zone 2 milestones: active (non-released) milestones only; released move to Zone 4
- Zone 3 renders `<HorizonZone />` — initially a stub `null` return until that feature
  is implemented; the layout slot is present now
- Zone 4 is collapsed by default, same as the current archive toggle
- All state refreshes via SSE — no refresh buttons

## What Does Not Change

- Project name + description header
- Stats bar (feature count, milestone count, active count, etc.)
- WhatChangedBanner (moved into Zone 1)
- PreparePanel (Wave Plan) — kept in Zone 1 when active, omitted otherwise
- FeatureCard component — stays in codebase for the feature detail page; not rendered on Dashboard
- Routing — `/` still renders `<Dashboard />`
- No new API endpoints — all data comes from the existing `useProjectState` hook

## Out of Scope

- The `dashboard-horizon-zone` feature itself (Zone 3 content)
- Orchestrator-aware empty states (`dashboard-empty-states`)
- Mobile-specific layout changes beyond what's already in place
