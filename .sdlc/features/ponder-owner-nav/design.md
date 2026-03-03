# Design: ponder-owner-nav

## Summary

Two targeted UI bug fixes in the ponder entry detail view. No new components, no backend
changes, no routing changes. Both fixes are surgical edits to existing React components.

---

## Fix 1: Owner message visibility

### Current behavior

`SessionBlock` receives an optional `ownerName: string | undefined` prop. The `isOwner`
detection is:

```ts
const isOwner = ownerName
  ? event.name.toLowerCase() === ownerName.split(' ')[0].toLowerCase() &&
    event.role.toLowerCase().includes('owner')
  : false
```

`ownerName` is only passed while a run is actively `running`. For completed sessions it is
`undefined`, so the ternary short-circuits to `false` and owner messages are never
highlighted.

### Fix

Remove the `ownerName` dependency entirely. The session format always uses role "Owner" for
the project owner. Check only the role:

```ts
const isOwner = event.role.toLowerCase().includes('owner')
```

The `ownerName` prop can be removed from `SessionBlock`'s `Props` interface — it is not
needed by any logic path. The `ownerName` is still used in `DialoguePanel`'s
`pendingMessage` block (the optimistic owner message shown during an active run), which is
separate from `SessionBlock` and does not need to change.

The `PartnerMessage` component's `isOwner` prop and rendering are already correct and
require no changes.

### Component changes

**`frontend/src/components/ponder/SessionBlock.tsx`**
- Remove `ownerName?: string | null` from `Props`
- Change `isOwner` computation to `event.role.toLowerCase().includes('owner')`
- Remove the prop passing at the call site in `DialoguePanel.tsx`

---

## Fix 2: Floating back button for mobile ponder nav

### Current behavior

On mobile, when a ponder entry is selected (`/ponder/:slug`), the full-screen detail view
replaces the list. The back button (ArrowLeft) is in the header row at the top. If the
session is long and the user scrolls down, the header is still visible (it's `shrink-0`
and does not scroll). So this is actually not a scroll-accessibility problem.

Re-examining the "floating nav" requirement more carefully: the intent is to let the user
navigate between ponder entries without going back to the list. The current UX requires
the user to:
1. Tap back to go to the list
2. Tap a different entry

A floating entry-switcher or a "floating breadcrumb/nav" lets users move between entries
more fluidly. The simplest approach: add a floating previous/next arrow FAB pair at the
bottom-right of the mobile detail view, allowing navigation through the sorted+filtered
ponder entry list without leaving the detail view.

### Design

Add a `FloatingEntryNav` component within `EntryDetailPane` that:

- Is only rendered on mobile (`md:hidden`)
- Positions `fixed bottom-20 right-3` (above the mobile tab bar)
- Shows `ChevronLeft` (previous entry) and `ChevronRight` (next entry) buttons
- Receives `prevSlug` and `nextSlug` from the parent `PonderPage`
- Calls `navigate('/ponder/:slug')` on press
- Is hidden when there is only one entry in the list

```
[  ‹  ]  [  ›  ]    <- floating bottom-right, two pill buttons
```

The parent `PonderPage` computes `prevSlug` and `nextSlug` from the current `filtered`
sorted list based on the active `slug`.

### Component changes

**`frontend/src/pages/PonderPage.tsx`**
- Compute `currentIndex`, `prevSlug`, `nextSlug` from `filtered` list and current `slug`
- Pass them to `EntryDetailPane`
- `EntryDetailPane` renders a `FloatingEntryNav` (inline sub-component) on mobile

**`EntryDetailPane` props addition:**
```ts
prevSlug: string | null
nextSlug: string | null
```

---

## ASCII Wireframe

### Mobile detail view (long session)

```
┌─────────────────────────────┐
│  ← Back   Title     [status]│  ← fixed header
├─────────────────────────────┤
│                             │
│   Session 1  ·  2h ago      │
│   ─────────────────────     │
│   KAI · Systems Architect   │
│   Lorem ipsum...            │
│                             │
│   JORDAN · Owner           │  ← highlighted card (after fix)
│   My seed message...        │
│                             │
│   ...                       │
│   (long session content)    │
│                             │
│   ...                       │
│                   [ ‹ ] [›] │  ← FloatingEntryNav (above tab bar)
├─────────────────────────────┤
│  Chat    Files    Team      │  ← mobile tab bar
└─────────────────────────────┘
```

---

## No backend changes

Both fixes are purely frontend. No Rust code, no API changes, no YAML state mutations.
