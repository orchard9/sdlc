# Tasks: chat-bar-pinning

## T1 — Add shrink-0 to InputBar in DialoguePanel

File: `frontend/src/components/ponder/DialoguePanel.tsx`

Add `shrink-0` to the root element of `InputBar` in both render paths:
- Running state: `<div className="flex items-center ...">` → `<div className="shrink-0 flex items-center ...">`
- Idle state: `<form ... className="flex items-end ...">` → `<form ... className="shrink-0 flex items-end ...">`

## T2 — Add shrink-0 to InputBar in InvestigationDialoguePanel

File: `frontend/src/components/investigation/InvestigationDialoguePanel.tsx`

Same change as T1 — add `shrink-0` to the root element of `InputBar` in both render paths:
- Running state: `<div className="flex items-center ...">` → `<div className="shrink-0 flex items-center ...">`
- Idle state: `<form ... className="flex items-end ...">` → `<form ... className="shrink-0 flex items-end ...">`
