# QA Plan: Knowledge Research Modal and Research Button on List View

## Scope

Frontend-only change: `NewResearchModal` component + Research button in `EntryListPane`. No server or data-layer changes.

## Test Areas

### 1. Build and Type Safety
- `npm run build` completes with zero TypeScript errors
- No missing imports or undefined references
- `cargo clippy --all -- -D warnings` passes (Rust codebase unchanged, but verify no regressions)

### 2. NewResearchModal — Unit Behaviour
- Modal renders when `open={true}` and does not render when `open={false}`
- Header shows "Research: {entryTitle}" with the correct title
- Close button (X) calls `onClose`
- Escape key calls `onClose`
- Clicking backdrop calls `onClose`
- "Cancel" button calls `onClose`
- "Start Research" button is disabled while `submitting` is true
- Submitting with empty topic calls `api.researchKnowledge(slug)` with no topic argument (or `undefined`)
- Submitting with non-empty topic calls `api.researchKnowledge(slug, topic)`
- Successful API call triggers `onStarted()` and closes modal
- API failure shows inline error message; modal stays open

### 3. Research Button in Entry List
- Each entry row shows a Research button
- Research button is hidden by default and appears on row hover (`group-hover`)
- Clicking the Research button does NOT select/navigate to the entry (stopPropagation)
- Clicking the Research button opens `NewResearchModal` with correct `entrySlug` and `entryTitle`

### 4. KnowledgePage Integration
- Only one `NewResearchModal` is rendered at a time
- After modal closes (`onClose` or `onStarted`), the modal is unmounted
- Selected entry in the list is unchanged after modal interaction

### 5. Existing Behaviour Unchanged
- `EntryDetailPane` "Research More" button still works as before
- Catalog pane and entry list navigation unaffected
- SSE reload on `KnowledgeResearchCompleted` still triggers entry list refresh

## Manual Verification Checklist

- [ ] Open Knowledge page in browser
- [ ] Hover over an entry row — Research button appears
- [ ] Click Research button — modal opens with entry title in header
- [ ] Type a topic hint — "Start Research" becomes active
- [ ] Submit — modal closes, run appears in Runs page
- [ ] Open modal again, leave topic blank, submit — run starts with entry slug as default topic
- [ ] Press Escape while modal open — modal closes, no run started
- [ ] Click entry row (not the Research button) — entry opens in detail pane normally
