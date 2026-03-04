# Code Review: ThreadsPage Mobile Layout Fix

## Summary

Single-file change: `frontend/src/pages/ThreadsPage.tsx`
- Added `cn` import from `@/lib/utils`
- Changed left pane `<div>` to use responsive `cn()` classes
- Changed right pane `<div>` to use responsive `cn()` classes

## Diff Review

### Import addition

```diff
+import { cn } from '@/lib/utils'
```

Correct. `cn` is the standard Tailwind class merge utility already used throughout the codebase.

### Left pane

```diff
-<div className="w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden md:flex md:w-[280px]">
+<div className={cn(
+  'w-full shrink-0 border-r border-border flex-col overflow-hidden',
+  'md:flex md:w-[280px]',
+  slug ? 'hidden' : 'flex',
+)}>
```

Analysis:
- `w-full` on mobile ensures the list pane uses the full viewport width when shown. Correct.
- `flex-col` is always applied (was already present). Correct.
- `md:flex md:w-[280px]` restores desktop behavior identically to the original. Correct.
- `slug ? 'hidden' : 'flex'` — on mobile, hides the pane when a thread is selected and shows it on the list page. Correct.
- Tailwind precedence: `hidden` from the conditional and `md:flex` from the responsive override — Tailwind's `md:` prefix correctly overrides the base `hidden` at ≥ 768px. No conflict.

### Right pane

```diff
-<div className="flex-1 flex flex-col overflow-hidden">
+<div className={cn(
+  'flex-1 flex-col overflow-hidden',
+  'md:flex',
+  slug ? 'flex' : 'hidden',
+)}>
```

Analysis:
- `flex-1` is always applied. Correct.
- `flex-col` is always applied. Correct.
- `md:flex` restores desktop always-visible behavior. Correct.
- `slug ? 'flex' : 'hidden'` — on mobile, shows the detail pane only when a thread is selected. Correct.
- On desktop the `EmptyDetailState` is still rendered when no thread is selected (pane visible, content shows "Select a thread"). This is correct — desktop behavior unchanged.

## Correctness

- Tailwind `hidden` = `display: none`. At `md:` breakpoint, `md:flex` overrides to `display: flex`. This is standard Tailwind responsive override and works correctly.
- `slug` is derived from `useParams` at component mount and updates reactively via React Router. No stale value risk.
- The `AppShell` already renders a back chevron (`<`) for paths matching `/threads/:slug` via `DETAIL_BASES`. This back chevron navigates to `-1` in history, which brings the user back to `/threads`. Combined with the left pane showing again at `/threads`, navigation is complete.

## TypeScript

`npx tsc --noEmit` passes with zero errors.

## Findings

None. The change is correct, minimal, and complete.
