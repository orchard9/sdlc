# Tasks: Commands Catalog — /docs/commands

## T1 — Create static commands data module

**File:** `frontend/src/components/docs/commands-data.ts`

Create the `CommandCategory` type, `CommandEntry` interface, `CATEGORY_LABELS` record, and the `COMMANDS` array with all 34 entries as specified in the design. Entries must cover all commands in `ALL_COMMANDS` from `crates/sdlc-cli/src/cmd/init/commands/mod.rs`.

**Done when:** The file exports `COMMANDS`, `CATEGORY_LABELS`, `CommandEntry`, and `CommandCategory` with no TypeScript errors.

---

## T2 — Build CommandsCatalog component

**File:** `frontend/src/components/docs/CommandsCatalog.tsx`

Implement the component per design:
- Search input (controlled, `useState('')`) that filters by slug + description
- Group filtered commands by category in fixed order: lifecycle → planning → workspace → analysis → tooling → project
- Render a category header with label and count badge for each non-empty group
- Render a `CommandRow` per command: monospace invocation code, muted description, `CopyButton`
- Empty state: "No commands match" when filter produces zero results
- Use `CopyButton` from `@/components/shared/CopyButton`

**Done when:** Component renders all 34 commands ungrouped in a passing browser smoke-test; search filters work correctly.

---

## T3 — Wire CommandsCatalog into DocsPage

**File:** `frontend/src/pages/DocsPage.tsx`

Modify `DocsPage` to import `CommandsCatalog` and render it when `section === 'commands'`, replacing the existing placeholder `<div>`. All other sections continue showing their placeholder divs unchanged.

**Done when:** Navigating to `/docs/commands` shows the full commands catalog; other docs routes are unaffected.

---

## T4 — TypeScript build verification

Run `npm run build` (or `npx tsc --noEmit`) inside `frontend/` and confirm zero TypeScript errors and zero ESLint errors related to the new files.

**Done when:** Build exits 0.
