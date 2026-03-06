# Plan: Ask Ponder — sidebar explain query button

## Summary

Add a third utility button to the sidebar bottom-left — "Ask Ponder" — for asking how features, code, and systems work. Sits beside "Fix Right Away" and "Search". Reuses existing AMA infrastructure entirely.

## Size assessment: Small — single feature

One modal component + two file edits. No new backend, no new API endpoints, no new data model.

## Milestone

**slug:** v44-ask-ponder
**title:** Ask Ponder — sidebar shortcut for how-it-works questions
**vision:** A developer can press ⌘/ from anywhere in the app, ask "how does X work?", and get a synthesized answer with source file citations in under 30 seconds — without leaving their current page.

## Features

### ask-ponder-modal

**title:** AskPonderModal — input, streaming answer, open-as-thread

**description:**
New modal component at `frontend/src/components/shared/AskPonderModal.tsx`. Three states:

1. `input` — textarea for the question, ⌘↵ to submit, "Ask" button
2. `answering` — source file chips appear as context is found, answer streams via AmaAnswerPanel (EventSource on `/api/run/:key/events`)
3. `answered` — full answer shown with sources; footer has "Ask another" (reset) and "Open as Thread" (calls `api.createAmaThread` + `api.addAmaThreadTurn` then navigates to `/threads/:id`)

Implementation:
- On submit: `api.answerAma(question, undefined, { turnIndex: 0 })` → returns `{ run_id, run_key }`
- Mount `AmaAnswerPanel` with `runKey` to stream; `onDone` callback transitions to `answered`
- Sources: call `api.getAmaResult(runId)` on done to get source chips
- Escape closes; ⌘/ toggles (same as AppShell keyboard handler pattern)
- Wire into `AppShell.tsx`: `askOpen` state, `⌘/` handler, `<AskPonderModal open={askOpen} onClose=...>`
- Wire into `Sidebar.tsx`: `onAskPonder?: () => void` prop, `HelpCircle` icon button after Search, kbd hint `⌘/`

## Tasks (for ask-ponder-modal)

- [ ] Create `AskPonderModal.tsx` with input/answering/answered states
- [ ] Wire streaming via AmaAnswerPanel + source chips on completion
- [ ] Add "Open as Thread" action
- [ ] Add `onAskPonder` prop to Sidebar + HelpCircle button + ⌘/ kbd hint
- [ ] Add `askOpen` state + ⌘/ keyboard handler to AppShell
- [ ] Render `<AskPonderModal>` in AppShell
