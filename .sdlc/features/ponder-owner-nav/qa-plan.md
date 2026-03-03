# QA Plan: ponder-owner-nav

## TC-1: Owner message highlighted in completed session

**Precondition:** A ponder session file exists with a partner block using role "Owner"
(e.g., `**JORDAN · Owner**`).

**Steps:**
1. Render `SessionBlock` with a parsed partner event where `role === 'Owner'`.
2. Inspect the rendered `PartnerMessage`.

**Expected:** `isOwner` is `true`; the component renders with the highlighted card style
(border, bg-muted/20, primary-colored name).

**Verification:** Code inspection — `isOwner` is now derived purely from
`event.role.toLowerCase().includes('owner')` without any `ownerName` prop dependency.

---

## TC-2: Non-owner partner message not highlighted

**Steps:**
1. Render `SessionBlock` with a partner event where `role === 'Systems Architect'`.
2. Inspect the rendered `PartnerMessage`.

**Expected:** `isOwner` is `false`; standard rendering without card style.

---

## TC-3: Pending message (active run) still shows owner card

**Steps:**
1. In `DialoguePanel`, trigger a send that produces a `pendingMessage`.
2. Observe the optimistic owner message block (the inline JSX in `DialoguePanel`, not
   `SessionBlock`).

**Expected:** The pending block still shows the owner name and message in the card style.
This code path is unchanged by this feature.

---

## TC-4: FloatingEntryNav renders on mobile

**Steps:**
1. Open the ponder page in a mobile viewport (< 768px).
2. Select a ponder entry that is in the middle of a list with 3+ entries.
3. Observe the bottom-right of the detail view.

**Expected:** Two navigation buttons (ChevronLeft, ChevronRight) are visible above the
mobile tab bar. The previous/next buttons navigate to adjacent entries.

---

## TC-5: FloatingEntryNav hidden when only one entry

**Steps:**
1. Open the ponder page on mobile with only one entry in the current filtered list.
2. Observe the detail view.

**Expected:** FloatingEntryNav is not rendered (no chevron buttons visible).

---

## TC-6: TypeScript compilation clean

**Steps:**
1. Run `npm run build` or `npx tsc --noEmit` in `frontend/`.

**Expected:** Zero TypeScript errors. Vite build succeeds.

---

## TC-7: No regression in desktop ponder view

**Steps:**
1. Open ponder page on desktop viewport (>= 768px).
2. Select a ponder entry with a completed session that has an owner message.
3. Verify owner message is highlighted.
4. Verify FloatingEntryNav is NOT rendered (desktop has sidebar nav).

**Expected:** Owner card visible; no floating nav buttons on desktop.
