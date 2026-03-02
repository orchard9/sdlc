# QA Plan: Ponder-First Entry Path for New Users

## Scope

Manual verification of four UI changes across three React pages. No backend changes, no new API endpoints. All test cases can be verified in a running dev server (`sdlc ui`).

## Test Cases

### TC1 — Dashboard: Setup-incomplete banner shows "New Ponder" link

**Precondition:** Project has no vision/architecture documents and no agents configured (setup incomplete).

**Steps:**
1. Navigate to `/` (Dashboard).
2. Observe the amber setup-incomplete banner.

**Expected:**
- Banner displays both "New Ponder" and "Go to Setup →" links.
- "New Ponder" link href is `/ponder?new=1`.
- Clicking "New Ponder" navigates to the Ponder page with the NewIdeaForm open.

---

### TC2 — Dashboard: No automatic redirect on load

**Steps:**
1. Navigate to `/` with setup incomplete.
2. Observe page render.

**Expected:**
- Dashboard renders normally — no automatic redirect to `/setup`.
- User remains on Dashboard.

---

### TC3 — Vision page subtitle

**Steps:**
1. Navigate to `/vision`.
2. Observe the page heading area.

**Expected:**
- Heading "Vision" is visible.
- Subtitle "What you're building and why — agents use this to make the right tradeoffs." appears below the heading in muted text.

---

### TC4 — Architecture page subtitle

**Steps:**
1. Navigate to `/architecture`.
2. Observe the page heading area.

**Expected:**
- Heading "Architecture" is visible.
- Subtitle "How it's built — agents use this to write code that fits the system." appears below the heading in muted text.

---

### TC5 — Ponder page: `?new=1` auto-opens form

**Steps:**
1. Navigate to `/ponder?new=1`.
2. Observe page load.

**Expected:**
- NewIdeaForm is displayed immediately without any user interaction.
- URL changes to `/ponder` (query param cleared) after form opens.

---

### TC6 — Ponder page: no form auto-open without param

**Steps:**
1. Navigate to `/ponder` (no query param).
2. Observe page load.

**Expected:**
- NewIdeaForm is NOT displayed on load.
- Normal ponder list is shown.
- "+" button still manually opens the form.

---

### TC7 — Ponder page: refresh does not re-open form

**Steps:**
1. Navigate to `/ponder?new=1` — form opens, URL becomes `/ponder`.
2. Manually refresh the page.

**Expected:**
- Form does NOT auto-open on refresh.
- Page loads in normal state.

---

## Regression Checks

- [ ] Dashboard renders without errors when setup IS complete (banner hidden, no regression).
- [ ] Vision page "Align" button still functions.
- [ ] Architecture page "Align" button still functions.
- [ ] Ponder page "+" button still opens form manually.
- [ ] Ponder page entry navigation works normally (no param interference).

## Pass Criteria

All 7 test cases pass. All 5 regression checks pass. No TypeScript compile errors (`npm run build` succeeds).
