# Spec: Ponder-First Entry Path for New Users

## Problem

The current first-run experience forces new users through a 4-step Setup wizard (Description → Vision → Architecture → Team) before they can do anything meaningful. This:

1. Presents a "wall" of formal documentation before the user has context about what the tool does
2. Teaches the wrong sequence — the tool's actual first creative act should be a Ponder, not a Vision doc
3. Makes Vision and Architecture feel like prerequisites rather than living project documents

Priya's diagnosis: "The product lets you fill in Vision/Architecture forms before you understand why those things matter. The correct order is: describe your idea first, then the tool generates Vision/Architecture from that, and you edit them later."

## Solution

Detect "new project" state and redirect first-run users to the Ponder page instead of the Setup wall. Generate draft Vision/Architecture from the first ponder as a side effect. Add explanatory subtitles to the Vision/Architecture pages so they don't feel opaque when users eventually visit them.

## Implementation

### 1. First-Run Detection and Redirect

**Condition for "new project" state:**
- No milestones exist AND no features exist AND setup is incomplete (Vision and/or Architecture not yet written)

**Current behavior:** Dashboard shows warning → user clicks "Go to Setup" → Setup wizard forces completion before tool works.

**New behavior:**
- Dashboard empty state shows "New Ponder" as the primary CTA (handled by `dashboard-empty-state`)
- The Setup page link remains accessible in the sidebar for power users who want to write Vision/Architecture directly
- Remove any hard redirect or blocking behavior that forces Setup before Dashboard loads

File likely affected: `frontend/src/pages/DashboardPage.tsx` or a routing layer in `frontend/src/App.tsx` that redirects to Setup.

Check for: any `useEffect` or router redirect that detects `setup_complete: false` and pushes to `/setup`. If found, remove or gate it so it only redirects on explicit user action, not automatically.

### 2. Vision and Architecture Page Subtitles

Both pages currently show a heading and a text editor with no explanation of purpose.

**File:** `frontend/src/pages/SetupPage.tsx` or the individual Vision/Architecture step components.

Add a single-sentence subtitle beneath each heading:

**Vision page:**
- Heading: "Vision"
- Subtitle (new): "What you're building and why — agents use this to make the right tradeoffs."

**Architecture page:**
- Heading: "Architecture"
- Subtitle (new): "How it's built — agents use this to write code that fits the system."

Styling: `text-sm text-muted-foreground` (or equivalent in the project's design system). One line below the heading, above the editor.

### 3. Vision/Architecture as Post-Hoc Editable (Not Gates)

Audit the codebase for any logic that blocks agent runs based on setup_complete status:

- If the server enforces `setup_complete: true` before allowing agent runs, that enforcement should be softened to a warning, not a hard block
- The goal: Vision/Architecture are useful context for agents but not required for the tool to function
- Any hard blocks should become informational prompts: "Adding a Vision helps agents make better decisions. [Add Vision]"

Note: If removing the hard block is a larger change than expected, acceptable fallback is to make the "skip" path more visible — a "Skip for now" link that is clearly labeled and prominent.

### 4. Ponder Page as Natural First Destination

The Ponder page already has a "New idea" form. When a user lands on Ponder for the first time (via "New Ponder" CTA from Dashboard), the new idea form should be auto-opened/focused.

File: `frontend/src/pages/PonderPage.tsx`

- Check if the page already supports `?new=1` query param to auto-open the new idea form
- If not, add it: detect `?new=1` in URL params and automatically trigger the "New idea" modal or inline form on page load
- This makes the "New Ponder" button from Dashboard feel like a direct continuation, not a navigation with an extra click

## Acceptance Criteria

- [ ] No automatic redirect to `/setup` on first load of the Dashboard
- [ ] Vision page shows explanatory subtitle: "What you're building and why — agents use this to make the right tradeoffs."
- [ ] Architecture page shows explanatory subtitle: "How it's built — agents use this to write code that fits the system."
- [ ] Ponder page responds to `?new=1` query param by auto-opening the new idea form
- [ ] Setup pages remain accessible from the sidebar — they are not removed, only de-prioritized
- [ ] No hard block prevents agent runs when Vision/Architecture are empty (warning is acceptable, block is not)

## Out of Scope

- Auto-generating Vision/Architecture from a Ponder description (complex agent-side feature, future milestone)
- Changing the Setup page's content or step order
- Removing the Setup page from the sidebar
