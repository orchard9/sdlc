# QA Plan: Commands Catalog — /docs/commands

## Test Approach

Manual smoke tests against the running dev server (`sdlc ui`) + TypeScript build check. No backend to test.

---

## QC-1 — All 34 commands are rendered

**Steps:**
1. Navigate to `/docs/commands`.
2. Count all command rows visible on screen (scroll to bottom if needed).

**Pass:** 34 rows rendered.
**Fail:** Any command missing or count differs.

---

## QC-2 — Commands are grouped under correct categories

**Steps:**
1. Navigate to `/docs/commands`.
2. Verify category headers appear: Lifecycle, Planning, Workspace, Analysis & Quality, Tooling, Project Setup.
3. Spot-check: `/sdlc-run` is under Lifecycle; `/sdlc-ponder` is under Workspace; `/sdlc-quality-fix` is under Analysis & Quality.

**Pass:** All 6 headers present; spot-checked commands in correct categories.
**Fail:** Missing headers or misplaced commands.

---

## QC-3 — Search filters by command name

**Steps:**
1. Type "run" in the search box.
2. Verify only commands whose slug or description contains "run" are shown (e.g. `/sdlc-run`, `/sdlc-run-wave`, `/sdlc-cookbook-run`, `/sdlc-tool-run`, `/sdlc-tool-uat` description includes "run").
3. Clear search — all 34 commands reappear.

**Pass:** Filtered results match; clear restores full list.
**Fail:** Non-matching commands shown or matching commands hidden.

---

## QC-4 — Search filters by description

**Steps:**
1. Type "milestone" in the search box.
2. Commands whose description mentions "milestone" should appear (e.g. `/sdlc-prepare`, `/sdlc-run-wave`, `/sdlc-milestone-uat`, `/sdlc-pressure-test`).

**Pass:** At least 3 milestone-related commands shown.
**Fail:** Fewer or zero results.

---

## QC-5 — Empty state on no match

**Steps:**
1. Type a nonsense string: "zzznomatch".
2. Verify an empty-state message appears ("No commands match" or equivalent).
3. No category headers or command rows should be visible.

**Pass:** Empty state shown; no stale rows.
**Fail:** Old commands still visible or no empty-state message.

---

## QC-6 — Copy button copies invocation string

**Steps:**
1. Click the copy button on `/sdlc-next <feature-slug>`.
2. Paste into a text editor.

**Pass:** Clipboard contains exactly `/sdlc-next <feature-slug>`.
**Fail:** Wrong text, empty clipboard, or error.

---

## QC-7 — Other docs sections unaffected

**Steps:**
1. Navigate to `/docs/quickstart` — verify placeholder still renders.
2. Navigate to `/docs/planning-flow` — verify placeholder still renders.

**Pass:** Placeholder divs appear for non-commands sections.
**Fail:** Blank page or catalog renders where placeholder expected.

---

## QC-8 — TypeScript build passes

**Steps:**
```bash
cd frontend && npm run build
```

**Pass:** Build exits 0, no TypeScript errors.
**Fail:** Any compile error in new files.
