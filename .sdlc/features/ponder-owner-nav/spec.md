# Spec: ponder-owner-nav

## Overview

Two independent UX fixes in the ponder UI:

1. **Owner message visibility fix** — Owner messages (those with role "Owner") in completed
   sessions are not visually distinguished from regular partner messages. The current logic
   requires a runtime `ownerName` prop that is only set while a ponder run is actively in
   progress. For all previously completed sessions the prop is undefined, so `isOwner` is
   always false and the owner's messages are rendered without the highlighted card style.

2. **Floating nav in ponder detail** — The back-to-list navigation button in the ponder
   entry detail pane is only visible on mobile and is located inside the header row. On a
   long dialogue session, once the user scrolls down the header is out of view and there is
   no way to return to the list without scrolling back to the top. A floating nav element
   (e.g. a floating back button or breadcrumb) should remain accessible without requiring
   the user to scroll back up.

## Bug 1: Owner message visibility

### Root cause

In `SessionBlock.tsx` the `isOwner` flag is derived as:

```ts
const isOwner = ownerName
  ? event.name.toLowerCase() === ownerName.split(' ')[0].toLowerCase() &&
    event.role.toLowerCase().includes('owner')
  : false
```

The guard `ownerName ?` means `isOwner` is always `false` when `ownerName` is `undefined`
(i.e., for every completed session). The `ownerName` prop is only passed from
`DialoguePanel` while a run is actively `running`:

```ts
ownerName={isRunning && runState.status === 'running' ? runState.ownerName : undefined}
```

The session format written by the agent always marks the owner with role "Owner":
`**NAME · Owner**`. This role information is self-contained in each parsed `partner` event
and does not require an external `ownerName` reference.

### Fix

Remove the dependency on the `ownerName` prop for rendering completed sessions. Detect
owner messages by checking if `event.role.toLowerCase().includes('owner')` directly. The
`ownerName` prop can be dropped from `SessionBlock`'s interface (or kept as an optional
no-op for backward compatibility), but the `isOwner` logic must not require it.

The `PartnerMessage` component already accepts `isOwner` and applies the highlighted card
style when true — no changes needed there.

## Bug 2: Floating nav in ponder entry detail

### Problem

The `EntryDetailPane` header contains a back button (`ArrowLeft`) that is:
- Only rendered on mobile (`md:hidden`)
- Located in the static header row

When a session has many messages, the user scrolls the dialogue content but the header
stays put (it is `shrink-0`). However, on **desktop** there is no back navigation at all
since the left pane is always visible — the issue is mobile-only.

Wait — on mobile the header IS visible and doesn't scroll. The more precise problem is
a floating back button for returning to the ponder list on mobile without the header
needing to be visible. On mobile, when a slug is active, the full-screen detail view
covers the list. The existing back button is in the header which is always at the top
(not obscured by scroll). On desktop the left pane is always present.

Re-examining: the floating nav is about providing quick entry-to-entry navigation within
the ponder list without going back to the list — a floating "jump to top" or a persistent
mini breadcrumb that lets the user navigate between entries.

### Fix

Add a floating back-to-list FAB (Floating Action Button) that is visible on mobile at the
bottom of the detail pane, allowing the user to return to the ponder list without
scrolling to the top of a long session. The button appears fixed/sticky above the mobile
tab bar in the detail view.

Additionally, on the desktop view, make the left-pane entry list highlight the currently
selected entry so the nav context is always clear without any floating element needed
(the selected entry is already highlighted via `selected` prop — confirm it is working
correctly and the list scrolls to keep it in view).

## Acceptance criteria

1. In a completed ponder session where the agent wrote an `**OWNER_NAME · Owner**` block,
   that message is rendered with the `isOwner=true` highlighted card style (bordered card,
   primary-colored name).

2. On mobile (`< md` breakpoint), when viewing an entry with a long dialogue, a floating
   back-to-list button is accessible without scrolling to the top of the page.

3. No regression: existing `pendingMessage` owner block in `DialoguePanel` (shown during
   an active run) continues to render correctly.

4. TypeScript compiles without errors; existing Vite build passes.

## Scope

- `frontend/src/components/ponder/SessionBlock.tsx` — fix `isOwner` logic
- `frontend/src/pages/PonderPage.tsx` — add floating back button on mobile detail view
- No Rust/server changes required
