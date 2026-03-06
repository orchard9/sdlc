# Tasks: AskPonderModal

## T1: Create AskPonderModal component

Create `frontend/src/components/shared/AskPonderModal.tsx`.

Implement three states: `input | answering | answered`.

- `input`: textarea (autofocused on open), ⌘↵ submits, Ask button disabled when empty
- On submit: call `api.answerAma(question, undefined, { turnIndex: 0 })` → get `{ run_id, run_key }`; transition to `answering`
- `answering`: show pulsing "Reading codebase…" indicator in header; mount `AmaAnswerPanel` with `runKey`; `onDone(finalText)` → call `api.getAmaResult(runId)` to get sources, transition to `answered`
- `answered`: render source chips (file path + line range), render answer markdown via ReactMarkdown + remark-gfm
- Reset on `open` → true: clear all state, autofocus textarea
- Escape closes (via `useEffect` keydown listener on `open`)

## T2: Add "Ask another" and "Open as Thread" footer actions

In the `answered` state footer:

- **Ask another** (ghost button, left): reset to `input` state, preserve question text for editing, clear sources/answer
- **Open as Thread** (outlined button, right):
  1. Call `api.createAmaThread({ title: question })`
  2. Call `api.addAmaThreadTurn(thread.id, { question, sources, run_id: runId })`
  3. Navigate to `/threads/${thread.id}` via `useNavigate()`
  4. Call `onClose()`

## T3: Add Ask Ponder button to Sidebar

In `frontend/src/components/layout/Sidebar.tsx`:

- Add `onAskPonder?: () => void` to `SidebarProps` interface
- Add `onAskPonder` to destructured props
- Import `HelpCircle` from lucide-react
- Add button after the Search button in the bottom utility section, following the exact same markup/class pattern as Fix Right Away and Search
- Label: "Ask Ponder", kbd hint: `⌘/`

## T4: Wire AskPonderModal into AppShell

In `frontend/src/components/layout/AppShell.tsx`:

- Import `AskPonderModal`
- Add `const [askOpen, setAskOpen] = useState(false)`
- In the existing keyboard handler `useEffect`, add: if `(e.metaKey || e.ctrlKey) && e.key === '/'` → `e.preventDefault(); setAskOpen(prev => !prev)`
- Pass `onAskPonder={() => setAskOpen(true)}` to `<Sidebar>`
- Render `<AskPonderModal open={askOpen} onClose={() => setAskOpen(false)} />` alongside the other modals
