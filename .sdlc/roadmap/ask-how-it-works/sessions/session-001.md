---
session: 1
timestamp: 2026-03-05T00:00:00Z
orientation:
  current: "Design shaped — reuse AMA infra, new AskPonderModal, sidebar button with ⌘/"
  next: "Implement AskPonderModal + wire into AppShell + Sidebar"
  commit: "Agreed pattern is clear; ready to build"
---

## Session

### Context read

Read the following to understand the current state:
- `frontend/src/components/layout/Sidebar.tsx` — bottom-left section with Fix Right Away + Search buttons
- `frontend/src/components/layout/AppShell.tsx` — modal state management, keyboard shortcuts
- `frontend/src/components/shared/FixRightAwayModal.tsx` — existing modal pattern to follow
- `frontend/src/components/tools/AmaAnswerPanel.tsx` — streaming answer component (reusable)
- `frontend/src/api/client.ts` — `answerAma`, `createAmaThread`, `addAmaThreadTurn` all exist

### Brief

> "In the bottom left we have 'fix right away' and 'search'. Lets add something to ask how something works in the app, this is going to be used to ask how features work, about the code, etc."

### Thought partners recruited

**Priya Kapoor · Principal UX Engineer** (Figma, Linear, Vercel)
Strong opinion: quick-access tools must return a skimmable answer fast. If the user sees a spinner for 10 seconds, the button stops getting used. Answer should appear in the modal — don't navigate away. Give an escape hatch to open a full thread for deep exploration.

**Marcus Chen · Backend/DX Engineer** (Stripe, Supabase)
Strong opinion: don't build new backend infrastructure for this. `api.answerAma()` already does semantic code search + synthesis. The AmaAnswerPanel already streams via SSE. This is a pure frontend composition task.

### Key findings

The existing AMA system (`/api/tools/ama/answer`) is exactly what this feature needs:
- Accepts a `question` + optional `threadContext`
- Semantically searches the codebase
- Returns sources (file + line) and a synthesized answer
- Streams via `AmaAnswerPanel` using EventSource on `/api/run/:key/events`
- Can be persisted into a thread via `api.createAmaThread` + `api.addAmaThreadTurn`

No new backend endpoint needed.

### Design decisions

⚑ Decided: Reuse `api.answerAma()` — no new backend needed.
⚑ Decided: Modal pattern, matching FixRightAwayModal structure (fixed overlay, centered card).
⚑ Decided: Icon = `HelpCircle` (lucide-react). Label = "Ask Ponder". Shortcut = ⌘/ (natural for "ask a question").
⚑ Decided: Three modal states: `input` → `answering` → `answered`.
⚑ Decided: Answering state shows source file chips as they arrive, then streams answer text.
⚑ Decided: Answered state has two actions: "Ask another" (reset to input) and "Open as Thread" (persist + navigate).
? Open: Should "Ask follow-up" keep thread context across turns within the modal, or is single-turn enough for v1? (Lean toward single-turn for v1 — threads handle multi-turn.)

### Implementation plan

**New file:** `frontend/src/components/shared/AskPonderModal.tsx`
- Props: `open: boolean`, `onClose: () => void`
- States: `input | answering | answered`
- On submit: `api.answerAma(question, undefined, { turnIndex: 0 })`
- Streaming: mount `AmaAnswerPanel` with `runKey`, call `onDone` to transition to answered
- Sources: extracted from `AmaResultPanel` pattern or inline via `api.getAmaResult(runId)`
- "Open as Thread": `api.createAmaThread({ title: question }) → api.addAmaThreadTurn(id, { question, sources, run_id })` then navigate to `/threads/:id`

**Edit:** `frontend/src/components/layout/Sidebar.tsx`
- Add `onAskPonder?: () => void` to SidebarProps
- Add HelpCircle button after Search in the bottom utility section
- Keyboard shortcut hint: ⌘/

**Edit:** `frontend/src/components/layout/AppShell.tsx`
- Add `askOpen` state
- Wire `onAskPonder={() => setAskOpen(true)}` into Sidebar
- Add ⌘/ keyboard shortcut handler
- Render `<AskPonderModal open={askOpen} onClose={() => setAskOpen(false)} />`

### Mockup captured

`ask-ponder-mockup.html` — three states shown: input, answering (streaming), answered.

## Product Summary

### What we explored
A third sidebar shortcut — "Ask Ponder" — that lets users ask how features, code, and systems work, accessible from the same bottom-left utility strip as Fix Right Away and Search.

### Key shifts
The key realization: no new backend work is needed. The existing AMA infrastructure (semantic code search + synthesis + streaming) already handles this use case completely. This is a pure UI composition task — a new modal wired into existing APIs.

### Implications
This is a small, self-contained feature: one new modal file and edits to two existing files. The "Open as Thread" escape hatch means it naturally integrates with the existing Threads system for follow-up without adding new infrastructure.

### Still open
Should the modal support multi-turn follow-ups within itself (passing threadContext across questions), or is single-turn enough for v1? Decision deferred to implementation — single-turn is simpler and threads already handle multi-turn.
