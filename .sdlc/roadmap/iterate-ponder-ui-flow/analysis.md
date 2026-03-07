# Ponder UI Flow Analysis

## Current State (PonderPage.tsx)

### Ponder Statuses (from sdlc-core/ponder.rs)
- **exploring** — actively ideating (violet badge)
- **converging** — idea is shaping up, nearly ready to commit (amber badge)
- **committed** — milestones/features created via `commitPonder` (green badge)
- **parked** — shelved/deferred (grey badge)

### Current UI Behavior by Status

| Status | Commit Button | Chat Input | Start Session | Status Badge |
|--------|--------------|------------|---------------|-------------|
| exploring | shown (muted) | enabled | enabled | violet |
| converging | shown (primary) | enabled | enabled | amber |
| committed | **hidden** | enabled | **hidden** | green |
| parked | **hidden** | enabled | **hidden** | grey |

### Key Data: `committed_to: string[]`
When a ponder is committed, `committed_to` contains the milestone slugs that were created. This is the bridge from ideation to execution.

## Problem Statement
After committing a ponder, the user lands on a page that says 'committed' with no forward action. The natural next step — running the features — requires navigating away to milestones/features manually. The UI dead-ends.

## Proposed Changes

### 1. Run Action Button for Committed Ponders (Primary Ask)
- Show a **"Prepare & Run"** button when `status === 'committed' && committed_to.length > 0`
- Links to the first milestone's prepare endpoint: `/api/milestone/{slug}/prepare`
- Uses the `startRun` from `AgentRunContext` with `runType: 'milestone_prepare'`
- If only one milestone, button says "Prepare {milestone-slug}"
- If multiple, show a dropdown or list of milestone links

### 2. Committed State — Show Milestone Links
- Below the status badge area, render a **"Created milestones"** section
- Each milestone is a clickable link to `/milestones/{slug}`
- Shows milestone status inline if available

### 3. Other Status-Aware UI Improvements

**Exploring state:**
- Could show session count progress (e.g., "2 sessions — keep exploring or commit when ready")
- The commit button is muted but present — good

**Converging state:**
- Could show orientation strip more prominently (commit signal)
- Highlight that the idea is nearly ready

**Parked state:**
- Show "Resume" action to set status back to exploring
- Show reason/context for parking if available

**Committed state (expanded):**
- Show `committed_at` timestamp
- Show milestone progress summary (how many features, their phases)
- "View in Milestones" quick link

### 4. General UI Improvements
- **Status transition breadcrumb**: exploring → converging → committed (visual progress)
- **Empty committed_to handling**: If committed but no milestones (edge case), show "Commit created no milestones — re-commit?" with action
- **Keyboard shortcut for commit**: Ctrl+Enter or similar in the dialogue panel
