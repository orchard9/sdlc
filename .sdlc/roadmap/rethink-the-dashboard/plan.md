# Dashboard Rethink — Commit Plan

## Milestone: `dashboard-rethink`
**Title:** Dashboard redesign — project digest, not control panel

**Vision:** When a developer opens the dashboard, they see the state of their project in 30 seconds. Shipped milestones show momentum. In-flight milestones show health at a glance. The Horizon zone surfaces everything coming — queued milestones, ideas ready to commit, ideas still being explored. No feature cards, no CommandBlocks, no wave-planning panels. The orchestrator builds; the dashboard tells the story.

**Acceptance test:** Open the dashboard. In under 30 seconds you can tell: (1) what shipped recently, (2) what's actively building and whether it's healthy, (3) what's queued to run next. Feature cards are gone. Wave controls are gone. The ponder workspace is visible in Horizon without navigating away.

---

## Three features (ordered by dependency)

### Feature 1: `dashboard-zone-layout`
**Title:** Dashboard four-zone layout — milestone-level digest replaces feature card grid

**What it does:**
- Replaces the current Dashboard with four zones (reading order): Needs Attention, Recently Shipped, In Flight, Horizon (placeholder)
- Removes: feature card grid, CommandBlocks, PreparePanel/wave panel, archive accordion, individual directive list
- Adds:
  - `MilestoneHealthCard` — milestone title, phase-distribution bar (features colored by phase), blocked count badge
  - `RecentlyShippedCard` — changelog style: milestone title, date, 1-line vision excerpt, green feature count
  - `OrchestratorStatusChip` — "N parallel / idle / stuck" status indicator (replaces active directive list)
- In-flight milestones link to milestone detail for feature-level drill-down

**Tasks:**
- Remove feature card grid and CommandBlocks from Dashboard.tsx
- Remove PreparePanel and archive accordion
- Create MilestoneHealthCard component
- Create RecentlyShippedCard component (last 3-5 released milestones)
- Create OrchestratorStatusChip component
- Wire four zones into new Dashboard layout
- Migrate Needs Attention (escalations/HITL) as-is — no changes there

---

### Feature 2: `dashboard-horizon-zone`
**Title:** Horizon zone — unified forward-looking surface for milestones and ponders

**What it does:**
- Implements the Horizon zone with three subgroups:
  1. **Queued Milestones** — planned milestones not yet started; solid card, feature count badge
  2. **Ready to Commit** — ponders with `status: converging`; amber dashed border, [Commit →] CTA
  3. **Exploring** — ponders with `status: exploring`; muted/ghost, [Continue →] CTA, capped at 3 items + "View all →" link
- Section header: "HORIZON" + "[+ New idea]" button (routes to /ponder new entry) + "N planned · M pondering" count
- Parked ponders excluded
- Eliminates the standalone Ideation Strip ("N pondering · M converging — View workspace →")
- Requires: `/api/roadmap` (ponder list endpoint) to supply ponder entries

**Tasks:**
- Create HorizonZone component with three-subgroup layout
- Create QueuedMilestoneRow (solid, feature count badge)
- Create ConvergingPonderRow (amber dashed, Commit CTA)
- Create ExploringPonderRow (muted, Continue CTA, cap 3 + truncation)
- Add "[+ New idea]" button and count label to section header
- Wire into Dashboard after In Flight zone
- Remove IdeationStrip component from Dashboard

---

### Feature 3: `dashboard-empty-states`
**Title:** Orchestrator-aware empty states — suggestion chips replace generic empty state

**What it does:**
- **State A** (orchestrator idle, Horizon not empty): renders amber status banner "Orchestrator idle — N milestones ready" with "Go to milestone board →" link. No wave execution button on dashboard.
- **State B** (true empty — no in-flight milestones AND Horizon is empty): renders three-block empty state:
  1. Accomplishment signal: "The project is caught up. N milestones shipped."
  2. Suggestion chips: calls `/api/suggest` (or equivalent) on load, renders top 3 suggestions with CTAs to /ponder or /sdlc-plan
  3. Look-back links: "View shipped milestones" + "Review filed tasks without a milestone"
- Replaces: existing `DashboardEmptyState` component
- Requires: a `/api/suggest` endpoint (or equivalent) that surfaces project-context suggestions

**Tasks:**
- Create OrchestratorIdleBanner component (State A)
- Create TrueEmptyState component with three blocks (State B)
- Wire state detection logic: check in-flight count + Horizon item count
- Implement suggestion chip fetch from /api/suggest
- Remove old DashboardEmptyState component

---

## Scoping notes (from session decisions)
- Wave execution UI is NOT on the dashboard — it lives on the milestone board page
- State A banner is status-only (no execution button)
- One milestone, atomic replacement — no partial rollouts
- Feature card drill-down access via milestone board, not dashboard

