# Code Review: ask-ponder-modal

## Files changed

| File | Type | Change |
|---|---|---|
| `frontend/src/components/shared/AskPonderModal.tsx` | New | 205 lines — modal with 3 states |
| `frontend/src/components/layout/Sidebar.tsx` | Edit | +HelpCircle import, +onAskPonder prop, +button |
| `frontend/src/components/layout/AppShell.tsx` | Edit | +import, +state, +shortcut, +prop, +modal render |

## Findings

### Correctness

**AskPonderModal.tsx**
- Three states (`input | answering | answered`) implemented correctly and match the spec.
- `api.answerAma(q, [], { turnIndex: 0 })` — sources are `[]` intentionally; the synthesis agent reads files via its own tools. This is a pragmatic simplification that avoids needing the semantic pre-search step used by the full AmaThreadPanel. Acceptable for v1.
- EventSource cleanup via `return () => es.close()` is correct.
- Reset on `open` change is correct (useEffect on `open`, clears all state).
- Escape handler registered only when `open` — correct.
- `toThreadId` generates a slug-like string from the question for thread ID. Potential collision if two near-identical questions are asked, but acceptable given low usage frequency.

**AppShell.tsx**
- `⌘/` shortcut guard `!e.shiftKey` added correctly to avoid conflict with other shortcuts.
- Toggle behavior (`prev => !prev`) matches the pattern used by Search and Fix Right Away.

**Sidebar.tsx**
- Ask Ponder button follows the exact same markup/class pattern as Fix Right Away and Search. Consistent.
- `title={collapsed ? 'Ask Ponder' : undefined}` matches collapsed tooltip pattern correctly.

### Style / UX
- Modal header shrinks question text with `truncate` — correct for long questions.
- Spinner while `answerText` is empty in answering state, then progressive markdown render when text starts streaming — good progressive disclosure.
- Answered state: "Ask another" (ghost) left, "Open as Thread" (outlined) right — matches mockup.
- `max-h-72` / `max-h-80` overflow-y-auto on body panels — prevents modal from growing unbounded for long answers.

### Issues found

**None blocking.** One tracked improvement:

- [ ] `toThreadId` can collide if the same question is asked twice within a session (same slug = 409 from server). Could append a timestamp or random suffix. Low priority — the server will return an error caught in `handleOpenAsThread` and shown to the user.

## Verdict

Implementation is complete and matches the spec. Build passes (`tsc -b && vite build`). Ready for QA.
