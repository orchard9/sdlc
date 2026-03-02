# QA Plan: Fullscreen View with Sticky TOC Navigation

## Scope

Verify that the sticky TOC rail appears correctly in the fullscreen artifact modal, TOC links work, the mobile fallback renders, and existing non-fullscreen behavior is unaffected.

## Test Cases

### TC1 — TOC rail appears on desktop fullscreen
Open fullscreen on an artifact with headings at ≥1024px viewport.
**Expected:** Left-rail nav panel with "CONTENTS" label and one entry per heading.

### TC2 — TOC entries scroll to correct section
Click a TOC entry for a heading below the fold.
**Expected:** Page smooth-scrolls to the corresponding heading anchor.

### TC3 — Heading indentation reflects level
Inspect the TOC rail on an artifact with H1, H2, H3 headings.
**Expected:** H1 = no indent, H2 = pl-3, H3 = pl-6.

### TC4 — Mobile "Jump to..." dropdown appears below `lg:`
Resize viewport below 1024px, open fullscreen on artifact with headings.
**Expected:** Left rail is hidden. A "Jump to..." `<select>` dropdown appears above the content.

### TC5 — No TOC for artifact with no headings
Open fullscreen on an artifact with no `#` headings.
**Expected:** No left rail, no dropdown. Single-column layout, modal width `max-w-4xl`.

### TC6 — Modal widens to `max-w-5xl` when TOC is present
Open fullscreen on an artifact with headings.
**Expected:** Modal container uses `max-w-5xl`.

### TC7 — In-panel (non-fullscreen) card view is unchanged
View an artifact card in normal panel mode.
**Expected:** No TOC rail or dropdown. Layout identical to pre-feature behavior.

### TC8 — `slugify` produces stable IDs
Open fullscreen on an artifact with headings containing special characters.
**Expected:** Heading `id` attributes contain only lowercase alphanumeric and hyphens.

### TC9 — TypeScript compilation passes
Run `cd frontend && npx tsc --noEmit`.
**Expected:** No type errors.

### TC10 — No console errors in fullscreen TOC mode
Open fullscreen on an artifact with headings, check browser console.
**Expected:** No React key warnings, no undefined errors.
