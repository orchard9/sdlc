# Code Review: chat-bar-pinning

## Summary

Added `shrink-0` to the root element of the `InputBar` component in both dialogue panel
files to ensure the chat bar always anchors to the bottom of its flex container.

## Changes Reviewed

### `frontend/src/components/ponder/DialoguePanel.tsx`

**Running state (line 129):**
```tsx
// Before
<div className="flex items-center gap-2 px-4 py-3 border-t border-border bg-card">
// After
<div className="shrink-0 flex items-center gap-2 px-4 py-3 border-t border-border bg-card">
```

**Idle state (line 146):**
```tsx
// Before
<form onSubmit={handleSubmit} className="flex items-end gap-2 px-4 py-3 border-t border-border bg-card">
// After
<form onSubmit={handleSubmit} className="shrink-0 flex items-end gap-2 px-4 py-3 border-t border-border bg-card">
```

### `frontend/src/components/investigation/InvestigationDialoguePanel.tsx`

**Running state (line 112):**
```tsx
// Before
<div className="flex items-center gap-2 px-4 py-3 border-t border-border bg-card">
// After
<div className="shrink-0 flex items-center gap-2 px-4 py-3 border-t border-border bg-card">
```

**Idle state (line 129):**
```tsx
// Before
<form onSubmit={handleSubmit} className="flex items-end gap-2 px-4 py-3 border-t border-border bg-card">
// After
<form onSubmit={handleSubmit} className="shrink-0 flex items-end gap-2 px-4 py-3 border-t border-border bg-card">
```

## Findings

**No issues found.** The changes are:
- Minimal and surgical — only `shrink-0` was added, no other classes were changed
- Correctly applied to all 4 render paths (2 files × 2 states each)
- Consistent with the existing pattern — the header strips in both panels already use `shrink-0`
- No logic changes, no new components, no new abstractions

## Verdict

APPROVED. The implementation exactly matches the design spec.
