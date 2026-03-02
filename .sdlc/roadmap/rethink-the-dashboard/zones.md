# Dashboard Zones — Proposed Layout

## Mental model shift
**Build control panel → Project digest**

The dashboard answers: "Give me the state of the project in 30 seconds." The orchestrator handles all building; the human uses the dashboard to orient, not operate.

## Five zones (reading order)

### 1. Needs Attention (amber)
Escalations + HITL blocked features. Highest urgency first. Already exists — keep it.

### 2. Recently Shipped
Last 3–5 released milestones, changelog style:
- Bold milestone title
- Date shipped
- 1-line vision excerpt
- Feature count with green checkmark

This is the **momentum signal**. Front and center, not buried in a collapsed archive.

### 3. In Flight
Active milestones with:
- Phase-distribution health bar (each feature colored by phase)
- Blocked features highlighted amber/red
- Click → drill into milestone detail for feature cards

No feature cards on the dashboard itself.

### 4. Horizon
Queued/planned milestones (not yet started). Simple list. Gives orchestrator its queue to look at.

### 5. Ideation Strip
"N pondering · M converging — View workspace →"
Single-line chip. Not a full panel.

## What gets removed
- Feature cards grid (access via milestone drill-down)
- CommandBlocks (orchestrator drives these, not humans)
- PreparePanel / wave planning (orchestrator manages this)
- Individual active-directive list (replace with OrchestratorHealthChip)
- Archive accordion (fold into Recently Shipped)

## New components needed
-  — phase-distribution bar, blocked features highlighted
-  — changelog-style card
-  — status chip (active N parallel / idle / stuck)
-  — link row to /ponder
-  — queued milestones list

## Open questions
- Should In Flight milestones show estimated completion?
- Does Horizon include ponder entries that are ready to commit?
- What does the empty state look like in the orchestrator era?