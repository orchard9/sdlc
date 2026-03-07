# Acceptance Test: v46-realtime-activity-feed

## Prerequisites
- sdlc-server running at http://localhost:7777
- At least one completed agent run exists (for completed-run verification)

## Checklist

### 1. Dashboard loads and agent panel is visible
- Navigate to http://localhost:7777
- Verify the page loads without errors
- Verify the agent panel (run list) is visible in the UI

### 2. Completed run expands with rich activity feed
- Click on a completed run card to expand it
- Verify the expanded card shows an ActivityTimeSeries chart
- Verify the expanded card shows paired-event cards (tool calls, assistant text, init card, or result card)
- Verify NO plain monospace log is rendered (AgentLog is removed)

### 3. Navigation link present on entity-targeted run
- On an expanded run card that targets a feature, milestone, ponder, or investigation, verify a navigation link is visible
- The link text shows the entity path (e.g. "features/some-slug" or "milestones/some-slug")
- The link includes an ExternalLink icon

### 4. Navigation link routes correctly
- Click the navigation link on a run card
- Verify the browser navigates to the correct entity detail page (URL matches the expected route)
- Navigate back to the dashboard

### 5. Dead code removed — no AgentLog or AgentEventLine
- Verify that `AgentLog.tsx` and `AgentEventLine.tsx` do not exist in the codebase
- Verify that no imports reference `AgentLog`, `AgentEventLine`, or the `AgentEvent` type

### 6. Frontend builds without errors
- Run `npm run build` in the frontend directory
- Verify zero errors
