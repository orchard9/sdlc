---
session: 2
timestamp: 2026-03-07T19:05:00Z
orientation:
  current: "All workspace pages now have rich onboarding empty states — verified on desktop and mobile"
  next: "Owner review — check each page visually, confirm messaging and layout"
  commit: "Owner confirms the messaging across all workspace pages"
---

**Xist - Owner**
Do the other workspace pages as well. Try to make the mobile experience work.

---

## Session 2: All workspace pages — rich empty states + mobile

### Context

Session 1 implemented a rich onboarding empty state for the Ponder page, replacing a near-blank detail pane. The owner asked to apply the same treatment to all other workspace pages and ensure mobile works.

### Analysis

Surveyed all workspace pages for empty state quality:

| Page | Before | Layout type |
|------|--------|-------------|
| Root Cause | Icon + "Select an investigation to continue" | WorkspaceShell |
| Guidelines | Icon + "Select a guideline to continue" | WorkspaceShell |
| Spikes | Icon + "Select a spike to view details" | Custom flex |
| Knowledge | Icon + "Select an entry to read" | 3-pane custom |
| Evolve | Already had full onboarding | WorkspaceShell |
| Ponder | Session 1 (done) | WorkspaceShell |

### Implementation

Applied the same design pattern from Ponder to all four pages needing work:

**Root Cause** (`InvestigationPage.tsx`): "Find the root cause." — 4-step flow (Symptoms, Areas, Synthesis, Action plan) + New Root Cause CTA button.

**Guidelines** (`GuidelinePage.tsx`): "Codify what works." — 4-step flow (Evidence, Principles, Draft, Publish) + New Guideline CTA button.

**Spikes** (`SpikePage.tsx`): "Answer one question fast." — 3-step flow (Question, Investigate, Verdict) + verdict strip (ADOPT/ADAPT/REJECT badges) + CLI command CTA. Spikes don't have a create modal so the CTA shows the slash command.

**Knowledge** (`KnowledgePage.tsx`): "What the team knows." — 3-step flow (Catalog, Research, Staleness) + CLI command CTA. Knowledge entries are added via CLI or accumulate from other workspaces.

### Design consistency

Every empty state follows the same structure:
1. Icon badge in colored circle (`bg-primary/10`)
2. Tagline heading (`text-xl font-semibold`)
3. Description paragraph (`text-sm text-muted-foreground`)
4. "HOW IT WORKS" section with step cards
5. Optional extra section (verdict badges for Spikes)
6. CTA (button where modal exists, CLI command where it doesn't)
7. Contextual "or select from the list" hint when entries exist

### Mobile responsiveness

Verified at 390x844 (iPhone 14 Pro) viewport:

- **WorkspaceShell pages** (Ponder, Root Cause, Evolve, Guidelines): The `WorkspaceShell` component uses `lg:` breakpoint. On mobile, only the list pane is visible — the detail pane (containing the onboarding) is `hidden lg:flex`. This is correct: on mobile users see the list and tap an item to navigate. The onboarding fills the detail pane on tablet/desktop where both panes are visible simultaneously.

- **Spikes**: Custom layout with `hidden lg:flex` on the detail pane. Same behavior — list shows on mobile, onboarding shows on desktop.

- **Knowledge**: 3-pane layout. On mobile, catalog + entry list show side-by-side (using `md:` breakpoint). Detail pane hidden until item selected. Onboarding visible at `md:` and above.

All pages render correctly at both viewports. No overflow, no clipping, no layout breaks.

### Verification

- TypeScript: `npx tsc --noEmit` — clean
- Frontend build: `npm run build` — success
- Rust binary rebuild with embedded assets — success
- Playwright screenshots at 1440x900 (desktop): all 4 pages verified
- Playwright screenshots at 390x844 (mobile): all pages verified

### Artifacts

- `all-workspace-empty-states.md` — detailed implementation summary with page-by-page breakdown
