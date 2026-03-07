# QA Plan: Rich Live Activity Renderer

## Manual Test Scenarios

### S1: Active run renders rich feed
1. Trigger an agent run (e.g. `/sdlc-next` on a feature)
2. Expand the running card in the activity sidebar
3. Verify: tool call cards, assistant text blocks render (not plain monospace log lines)
4. Verify: `ActivityTimeSeries` chart renders above the feed

### S2: Spawning state
1. Expand an active run immediately after triggering it (before first SSE event)
2. Verify: "Spawning agent..." spinner displays
3. Once first event arrives, spinner is replaced by the event card

### S3: Auto-scroll
1. Expand an active run that is producing events
2. Verify: feed auto-scrolls to show the latest event as new ones arrive
3. Scroll up manually; verify scroll continues to follow new events (or stops following if user scrolled — either behavior is acceptable)

### S4: Completed run unchanged
1. View a completed/failed/stopped run card expanded
2. Verify: renders identically to before (chart + rich feed, no regressions)

### S5: Build verification
1. `cd frontend && npm run build` passes with no errors
2. No TypeScript type errors
