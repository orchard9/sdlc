# QA Plan: Dashboard Horizon Zone

## Scope

Verify that Zone 3 (Horizon) renders correctly on the Dashboard, with correct
filtering, empty-state handling, and link navigation.

## Test Cases

### TC-1: Zone hidden when empty

**Setup**: Project state with no all-draft milestones and no exploring/converging
ponders.

**Steps**:
1. Load the Dashboard page.
2. Inspect the DOM / visual layout.

**Expected**: The Horizon section (with "Horizon" label and Telescope icon) is
absent from the page. No empty card or placeholder is shown.

---

### TC-2: Upcoming milestones shown correctly

**Setup**: Project has at least one active milestone where all assigned features are
in `draft` phase (or the milestone has no features).

**Steps**:
1. Load the Dashboard.
2. Locate the Horizon section.
3. Check the "Upcoming Milestones" sub-list.

**Expected**:
- Each horizon milestone appears as a row with its title (linked to
  `/milestones/<slug>`), a `StatusBadge`, and a feature count (e.g., "4 features").
- Milestones where at least one feature is past `draft` phase do NOT appear in
  this list.

---

### TC-3: Active ponders shown correctly

**Setup**: Roadmap has at least one ponder with `status === 'exploring'` or
`status === 'converging'`.

**Steps**:
1. Load the Dashboard.
2. Locate the "Active Ponders" sub-list in the Horizon section.

**Expected**:
- Each active ponder appears with its title linked to `/ponder/<slug>`, a
  `StatusBadge`, up to 2 tag chips, and a "copy" button.
- Ponders with `committed` or `parked` status are not shown.

---

### TC-4: Copy button behavior

**Setup**: At least one active ponder visible in the Horizon zone.

**Steps**:
1. Click the "copy" button next to a ponder row.

**Expected**:
- Button text changes to "✓" for approximately 1.5 seconds.
- Clipboard contains `/sdlc-ponder <slug>` for that ponder.

---

### TC-5: Navigation links

**Steps**:
1. Click a horizon milestone title link.
2. Verify route changes to `/milestones/<milestone-slug>`.
3. Go back.
4. Click a ponder title link.
5. Verify route changes to `/ponder/<ponder-slug>`.

**Expected**: Both links navigate within the SPA without full-page reload.

---

### TC-6: Mixed content (both sections present)

**Setup**: At least one horizon milestone and at least one active ponder.

**Steps**:
1. Load the Dashboard.
2. Inspect the Horizon card.

**Expected**: Both "Upcoming Milestones" and "Active Ponders" sub-sections are
visible within the same card. Section header labels are present.

---

### TC-7: Only one sub-section

**Setup A**: Horizon milestones present, but no active ponders.  
**Setup B**: Active ponders present, but all milestones have in-progress features.

**Steps**: Load the Dashboard for each setup.

**Expected**: Only the relevant sub-section is shown. The empty sub-section is
absent from the DOM (no empty header or blank area).

---

### TC-8: TypeScript build clean

**Steps**: Run `cd frontend && npx tsc --noEmit`.

**Expected**: Zero TypeScript errors.

---

### TC-9: Long title truncation

**Setup**: Ponder or milestone with a very long title (>60 characters).

**Expected**: Title is truncated with ellipsis (`truncate` CSS class) — no layout
overflow.

---

### TC-10: Tag chips clipped at 2

**Setup**: Ponder with 4+ tags.

**Expected**: At most 2 tag chips are shown per row. Extra tags are silently omitted
(no overflow, no "+N more" indicator needed).

## Pass Criteria

- All 10 test cases pass with no regressions to Zone 1, Zone 2, or Zone 4.
- TypeScript build clean (TC-8).
- No console errors during normal operation.
