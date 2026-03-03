# Tasks: ponder-owner-nav

## T1 — Fix isOwner detection in SessionBlock

Remove the `ownerName` prop from `SessionBlock` and update `isOwner` logic to detect owner
messages by role alone (`event.role.toLowerCase().includes('owner')`).

**Files:**
- `frontend/src/components/ponder/SessionBlock.tsx`
- `frontend/src/components/ponder/DialoguePanel.tsx` (remove ownerName prop pass)

## T2 — Add FloatingEntryNav to mobile ponder detail

Add a fixed floating nav component at the bottom-right of the mobile entry detail view
that shows prev/next entry navigation buttons. Compute prevSlug/nextSlug from the filtered
sorted entry list in PonderPage and pass to EntryDetailPane.

**Files:**
- `frontend/src/pages/PonderPage.tsx`

## T3 — Verify TypeScript build

Run `npm run build` (or `npx tsc --noEmit`) in `frontend/` to confirm no type errors.
Fix any compilation issues.

**Files:** frontend build output
