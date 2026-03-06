# Design: chat-bar-pinning

## Overview

This is a minimal surgical CSS fix. No new components, no new abstractions, no logic changes.
The only change is adding `shrink-0` to the root element of the `InputBar` component in two
files.

## Root Cause

Both dialogue panels use this flex column layout:

```
┌─────────────────────────────────────┐
│  Header strips (shrink-0)           │
├─────────────────────────────────────┤
│                                     │
│  Session stream (flex-1 overflow-y) │
│                                     │
│  ... scrollable content ...         │
│                                     │
├─────────────────────────────────────┤
│  InputBar  ← MISSING shrink-0       │
└─────────────────────────────────────┘
```

In a `flex flex-col` container, all children can be shrunk by default (`flex-shrink: 1`).
When the container is height-constrained (e.g., panel inside a workspace layout), the
flex algorithm may shrink the `InputBar` in order to fit the stream area, resulting in
a compressed or invisible chat bar.

The fix: add `shrink-0` (`flex-shrink: 0`) to the `InputBar`'s root element so the bar
always renders at its natural height and is never compressed.

## After Fix

```
┌─────────────────────────────────────┐
│  Header strips (shrink-0)           │
├─────────────────────────────────────┤
│                                     │
│  Session stream (flex-1 overflow-y) │
│  ... scrollable content ...         │
│                                     │
├─────────────────────────────────────┤
│  InputBar  ← shrink-0 ADDED         │
└─────────────────────────────────────┘
```

## Files Changed

### `frontend/src/components/ponder/DialoguePanel.tsx`

`InputBar` — two render paths:

1. Running state (returns `<div>`):
   ```tsx
   // Before
   <div className="flex items-center gap-2 px-4 py-3 border-t border-border bg-card">
   // After
   <div className="shrink-0 flex items-center gap-2 px-4 py-3 border-t border-border bg-card">
   ```

2. Idle state (returns `<form>`):
   ```tsx
   // Before
   <form onSubmit={handleSubmit} className="flex items-end gap-2 px-4 py-3 border-t border-border bg-card">
   // After
   <form onSubmit={handleSubmit} className="shrink-0 flex items-end gap-2 px-4 py-3 border-t border-border bg-card">
   ```

### `frontend/src/components/investigation/InvestigationDialoguePanel.tsx`

`InputBar` — same two render paths, same change.

## No Mockup Required

This feature introduces no new visual states and no layout changes visible to the user under
normal conditions. The chat bar already looks correct when the panel has adequate height —
this fix prevents a degenerate layout collapse that was only observable under constrained
height conditions.
