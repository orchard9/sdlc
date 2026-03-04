# Tasks: ThreadsPage Mobile Layout Fix

## T1: Add `cn` import and fix left pane visibility classes

**File:** `frontend/src/pages/ThreadsPage.tsx`

1. Add `import { cn } from '@/lib/utils'` to the imports section.
2. Update the left pane `<div>` className from:
   ```
   "w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden md:flex md:w-[280px]"
   ```
   to use `cn()` with responsive classes so the pane is hidden on mobile when a thread is selected.

## T2: Fix right pane visibility classes

**File:** `frontend/src/pages/ThreadsPage.tsx`

Update the right pane `<div>` className so it is hidden on mobile when no thread is selected (list-only view), and shown on mobile when a thread is selected.

## T3: Manual smoke test

Verify in browser:
- Mobile viewport (375px): `/threads` shows list only, `/threads/:slug` shows detail only, back chevron returns to list.
- Desktop viewport (1280px): both panes visible side by side, no regression.
