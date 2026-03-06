# Acceptance Test: Ask Ponder

## Setup
- sdlc-server running with at least one feature in the project

## Steps

1. Open the app sidebar
2. Verify "Ask Ponder" button is visible in the bottom-left utility strip, below "Search"
3. Click "Ask Ponder" — modal opens in `input` state
4. Type: "How does Fix Right Away diagnose issues?"
5. Press ⌘↵ (or click Ask)
6. Verify modal transitions to `answering` state — at least one source file chip appears
7. Verify answer text streams in (markdown rendered)
8. Verify modal transitions to `answered` state when streaming completes
9. Verify "Ask another" button resets modal to `input` state
10. Ask a second question, wait for answer
11. Click "Open as Thread" — verify navigation to `/threads/:id` with the answer persisted
12. Close modal, press ⌘/ — verify modal opens from keyboard shortcut
13. Press Escape — verify modal closes

## Pass criteria
- Modal opens via button click and ⌘/ shortcut
- Answer streams with source file citations
- "Open as Thread" navigates and persists the answer
- Escape closes the modal
