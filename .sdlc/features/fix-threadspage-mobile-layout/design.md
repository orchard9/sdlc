# Design: ThreadsPage Mobile Layout Fix

## Overview

The change is a single-file CSS/conditional-render fix in `ThreadsPage.tsx`. No new components, no new state beyond what already exists (`slug` from `useParams`), no backend changes.

## Layout Strategy

Use Tailwind responsive classes to toggle pane visibility based on the current URL:

- **List pane**: `hidden md:flex` when a thread is selected on mobile; always `flex` on desktop.
- **Detail pane**: `hidden md:flex` when no thread is selected on mobile; always `flex` on desktop.

The `slug` from `useParams` already tells us which pane is "active" on mobile.

## Wireframes

### Mobile — List View (`/threads`)

```
┌─────────────────────┐
│ ← Threads           │  ← AppShell mobile header (existing)
├─────────────────────┤
│  [ + New Thread ]   │
├─────────────────────┤
│  ○ UAT acceptance   │
│    open · 2 cmt     │
├─────────────────────┤
│  ○ QA Test Thread   │
│    open · 1 cmt     │
└─────────────────────┘
  (detail pane hidden)
```

### Mobile — Detail View (`/threads/:slug`)

```
┌─────────────────────┐
│ < Threads           │  ← AppShell back chevron (existing)
├─────────────────────┤
│  UAT acceptance  ●  │
│  opened by jordan   │
├─────────────────────┤
│  CORE ELEMENT       │
│  ...                │
├─────────────────────┤
│  Comments           │
├─────────────────────┤
│  [ Add a comment ]  │
└─────────────────────┘
  (list pane hidden)
```

### Desktop (≥ md) — Unchanged

```
┌───────────┬──────────────────────────────┐
│ List Pane │ Detail Pane                  │
│ 280px     │ flex-1                       │
│           │                              │
│  Thread 1 │  Title ● [Synthesize][Promo] │
│  Thread 2 │  ...                         │
│  Thread 3 │  Comments                    │
│           │  [ compose ]                 │
└───────────┴──────────────────────────────┘
```

## Implementation

### `ThreadsPage.tsx` — left pane div

Current:
```tsx
<div className="w-[280px] shrink-0 border-r border-border flex flex-col overflow-hidden md:flex md:w-[280px]">
```

New:
```tsx
<div className={cn(
  "w-full shrink-0 border-r border-border flex-col overflow-hidden",
  "md:flex md:w-[280px]",
  slug ? "hidden" : "flex"
)}>
```

On mobile: `flex` when no slug (list view), `hidden` when slug is present (detail view).
On desktop (`md:`): always `md:flex` with fixed `md:w-[280px]`.

### `ThreadsPage.tsx` — right pane div

Current:
```tsx
<div className="flex-1 flex flex-col overflow-hidden">
```

New:
```tsx
<div className={cn(
  "flex-1 flex-col overflow-hidden",
  "md:flex",
  slug ? "flex" : "hidden"
)}>
```

On mobile: `hidden` when no slug, `flex` when slug present.
On desktop: always `md:flex`.

## Dependencies

- `cn` utility is already imported in `ThreadsPage.tsx` — actually it is not currently imported. Need to add the import: `import { cn } from '@/lib/utils'`.
- `slug` is already derived from `useParams` at the top of the component.
- The `AppShell` back chevron for detail routes is already implemented and will continue to function correctly with these changes.

## Risk

Low. The change is two `className` modifications and one import addition. No logic, no state, no API calls affected.
