# Spec: AskPonderModal — input, streaming answer, open-as-thread

## Problem

Developers using the Ponder app frequently need to understand how a feature works, what a file does, or how system components connect. Currently there is no quick way to get a synthesized, code-grounded answer from within the app. The closest entry point (the AMA tool in `/tools`) requires navigating away and is buried inside a specific tool context.

## Solution

Add a third utility button to the sidebar bottom-left — **Ask Ponder** — that opens a modal for asking natural-language questions about the app's code and features. The modal reuses the existing AMA infrastructure (`api.answerAma`, `AmaAnswerPanel`) with no new backend work.

## Scope

### In scope
- New `AskPonderModal` component (`frontend/src/components/shared/AskPonderModal.tsx`)
- Sidebar button: HelpCircle icon, label "Ask Ponder", kbd hint `⌘/`
- AppShell wiring: `askOpen` state, `⌘/` keyboard shortcut, modal render
- Three modal states: `input`, `answering`, `answered`
- Source file chips shown when answer is complete
- "Ask another" action (reset to input)
- "Open as Thread" action (persist answer + navigate to `/threads/:id`)

### Out of scope
- Multi-turn follow-ups within the modal (threads handle this)
- Backend changes
- Mobile-specific layout

## User Stories

1. **Developer asks a how-it-works question.** Presses ⌘/, types "How does Fix Right Away diagnose issues?", gets a streamed answer with source file citations, dismisses the modal.
2. **Developer wants to continue exploring.** After getting an answer, clicks "Open as Thread" to persist it and continue the conversation in the Threads page.
3. **Developer resets.** Clicks "Ask another" to clear the answer and ask a different question without reopening the modal.

## Component Design

### `AskPonderModal`

```
Props:
  open: boolean
  onClose: () => void

State:
  step: 'input' | 'answering' | 'answered'
  question: string
  runKey: string | null
  runId: string | null
  sources: AmaSource[]
  answerText: string
  error: string | null
```

**input state:**
- Textarea (autofocused on open, reset on close)
- Placeholder: "How does X work?  What does Y file do?  How are Z runs streamed?"
- ⌘↵ to submit; "Ask" button
- Disabled when question is empty

**answering state:**
- Header subtitle: "Reading codebase…" with pulsing indicator
- Call `api.answerAma(question, undefined, { turnIndex: 0 })` → get `{ run_id, run_key }`
- Mount `AmaAnswerPanel` with `runKey` — streams answer text via SSE
- `onDone(finalText)` callback: call `api.getAmaResult(runId)` to get sources, transition to `answered`

**answered state:**
- Source chips: file path + line range (font-mono, muted)
- Rendered answer (markdown via ReactMarkdown)
- Footer: "Ask another" (left, text button) | "Open as Thread" (right, outlined)
- "Open as Thread": `api.createAmaThread({ title: question })` → `api.addAmaThreadTurn(id, { question, sources, run_id })` → navigate to `/threads/:id` → `onClose()`

### Keyboard behavior
- Escape → `onClose()`
- ⌘↵ in input → submit
- ⌘/ toggles modal (handled in AppShell)

### Reset behavior
- On `open` change to true: reset all state to `input`, clear question, autofocus textarea
- "Ask another" resets to `input` state, clears answer/sources, keeps question text for editing

## Sidebar changes (`Sidebar.tsx`)

Add `onAskPonder?: () => void` to `SidebarProps`.

New button after Search in the bottom utility section:
```tsx
<button onClick={onAskPonder} ...>
  <HelpCircle className="w-4 h-4 shrink-0" />
  {!collapsed && (
    <>
      <span className="flex-1 text-left">Ask Ponder</span>
      <kbd ...>⌘/</kbd>
    </>
  )}
</button>
```

## AppShell changes (`AppShell.tsx`)

```tsx
const [askOpen, setAskOpen] = useState(false)
```

Keyboard handler addition:
```tsx
if (e.key === '/') {
  e.preventDefault()
  setAskOpen(prev => !prev)
}
```
(condition: `e.metaKey || e.ctrlKey`)

Pass `onAskPonder={() => setAskOpen(true)}` to `<Sidebar>`.

Render `<AskPonderModal open={askOpen} onClose={() => setAskOpen(false)} />`.

## API calls used (all existing)

| Call | Purpose |
|---|---|
| `api.answerAma(question, undefined, { turnIndex: 0 })` | Start AMA run |
| `api.getAmaResult(runId)` | Get sources after streaming completes |
| `api.createAmaThread({ title: question })` | Create thread for Open as Thread |
| `api.addAmaThreadTurn(id, { question, sources, run_id })` | Persist turn into thread |

## Files changed

| File | Change |
|---|---|
| `frontend/src/components/shared/AskPonderModal.tsx` | **New** |
| `frontend/src/components/layout/Sidebar.tsx` | Add prop + button |
| `frontend/src/components/layout/AppShell.tsx` | Add state + shortcut + modal render |

## Acceptance criteria

- [ ] Ask Ponder button visible in sidebar bottom-left (below Search)
- [ ] ⌘/ opens the modal from any page
- [ ] Escape closes the modal
- [ ] Submitting a question transitions to answering state with streaming indicator
- [ ] Source file chips appear when answer completes
- [ ] "Ask another" resets to input without closing modal
- [ ] "Open as Thread" creates a thread and navigates to `/threads/:id`
