# QA Plan: UnifiedDialoguePanel

## Strategy

This is a pure refactor — no new behavior is introduced. QA focuses on regression
verification: the two affected pages must behave identically before and after the change.
No new Playwright tests are required; the existing behavior is validated by build success
and manual inspection of the two key screens.

## Automated Checks

### 1. TypeScript build

```bash
cd frontend && npm run build
```
Pass criteria: zero TypeScript errors, zero build warnings about missing exports.

### 2. Backend test suite

```bash
SDLC_NO_NPM=1 cargo test --all
```
Pass criteria: all tests pass (no regressions in server or core logic).

### 3. Clippy

```bash
cargo clippy --all -- -D warnings
```
Pass criteria: zero warnings.

## Manual Verification Checklist

### PonderPage — dialogue tab

- [ ] OrientationStrip renders above the session stream
- [ ] TeamRow renders when team members exist
- [ ] Sessions load and render (`SessionBlock` list visible)
- [ ] "No sessions yet" empty state shows when no sessions
- [ ] "Start from title & brief" button appears in empty state for non-committed entries
- [ ] "Commit anyway" button (ZeroStateCommitButton) appears for non-committed entries
- [ ] Sending a message shows the pending-message overlay (owner bubble + McpCallCard labeled `sdlc_ponder_chat` + working spinner)
- [ ] On run completion, pending overlay disappears and sessions reload
- [ ] Stop button appears while running; clicking it stops the run
- [ ] Auto-scroll fires on new sessions when user is at bottom
- [ ] Scroll lock holds position when user scrolls up during a run

### InvestigationPage — dialogue tab

- [ ] PhaseStrip renders above the session stream with correct phase highlighted
- [ ] Sessions load and render
- [ ] "No sessions yet" simple empty state shows (no commit button)
- [ ] Sending a message shows the pending-message overlay (owner bubble + McpCallCard labeled `sdlc_investigation_chat` + working spinner)
- [ ] On run completion, pending overlay disappears and sessions reload
- [ ] Stop button appears while running; clicking it stops the run
- [ ] Auto-scroll and scroll-lock behave correctly

## Risk Areas

| Risk | Mitigation |
|---|---|
| SSE handler for the wrong event type gets wired | Verified by checking `adapter.sseEventType` branch in code review |
| Ponder-specific empty state leaks to investigation | Verified by `emptyState` prop being independently provided per wrapper |
| `InputBar` placeholder text regresses | Verified manually in both pages |
| Session loading API calls swapped | Verified by checking adapter constants reference correct `api.*` methods |
