# Spike UI Page — QA Plan

## Scope

Manual + visual verification of the SpikePage component: list view, detail view, promote flow, sidebar nav, and empty state.

## Test Scenarios

### 1. Sidebar navigation
- [ ] "Spikes" entry appears in the sidebar Plan group with FlaskConical icon.
- [ ] Clicking "Spikes" navigates to `/spikes`.
- [ ] Sidebar entry is highlighted when on `/spikes` or `/spikes/:slug`.
- [ ] Collapsed sidebar shows icon only (no label).

### 2. List view — with data
- [ ] Spikes list loads from `GET /api/spikes`.
- [ ] Each row shows title, verdict badge, date, and a snippet of `the_question`.
- [ ] ADOPT badge is green.
- [ ] ADAPT badge is yellow.
- [ ] REJECT badge is red.
- [ ] ADOPT rows show "Next: /sdlc-hypothetical-planning" chip.
- [ ] ADAPT rows show "Promote to Ponder →" button when `ponder_slug` is not set.
- [ ] ADAPT rows show "Ponder: `<slug>`" link when `ponder_slug` is set.
- [ ] REJECT rows show "Stored in Knowledge" badge.
- [ ] REJECT rows where `knowledge_slug` is set have the badge link to `/knowledge/<slug>`.
- [ ] Clicking a row navigates to `/spikes/:slug` and shows detail pane.
- [ ] Verdict filter tabs (ALL / ADOPT / ADAPT / REJECT) filter the list correctly.

### 3. List view — empty state
- [ ] When `GET /api/spikes` returns an empty array, empty state renders.
- [ ] Empty state shows FlaskConical icon.
- [ ] Empty state heading: "No spikes yet".
- [ ] Empty state body explains what spikes are.
- [ ] Empty state shows example CLI command `/sdlc-spike <slug> — <the question>`.

### 4. Detail view — ADOPT
- [ ] Breadcrumb shows "Spikes / `<title>`".
- [ ] Verdict badge shows "ADOPT" in green.
- [ ] `the_question` is displayed prominently.
- [ ] Date is displayed.
- [ ] "What's Next" section is present.
- [ ] Section explains ADOPT = proven approach, not yet implemented.
- [ ] CLI hint shows `/sdlc-hypothetical-planning <slug>`.
- [ ] Copy button copies the CLI hint to clipboard.

### 5. Detail view — ADAPT (not yet promoted)
- [ ] Verdict badge shows "ADAPT" in yellow.
- [ ] "Promote to Ponder" button is present.
- [ ] Clicking "Promote to Ponder" calls `POST /api/spikes/:slug/promote`.
- [ ] On success, navigates to `/ponder/<ponder_slug>` from response.
- [ ] On failure, shows inline error message near the button; does not navigate.
- [ ] Button shows loading state during the request.

### 6. Detail view — ADAPT (already promoted)
- [ ] "Promote to Ponder" button is replaced by a link to `/ponder/<ponder_slug>`.
- [ ] Link text is "→ `<ponder_slug>`" or similar.

### 7. Detail view — REJECT
- [ ] Verdict badge shows "REJECT" in red.
- [ ] "Stored in Knowledge" section is present.
- [ ] If `knowledge_slug` is set, shows a link to `/knowledge/<knowledge_slug>`.

### 8. Mobile / BottomTabBar
- [ ] On mobile viewport, BottomTabBar "Plan" tab covers `/spikes`.
- [ ] Tapping a spike row in the list collapses the left pane and shows detail.
- [ ] Back navigation returns to list.

### 9. No-selection state (desktop)
- [ ] When on `/spikes` without a slug, right pane shows "Select a spike to view details" placeholder.

### 10. Error handling
- [ ] If `GET /api/spikes` fails, shows an error message (no crash).
- [ ] If `GET /api/spikes/:slug` fails, right pane shows "Spike not found".

## Definition of Done

All 10 scenario groups pass without errors in the browser console. TypeScript compiles with no errors (`npm run build`). No regressions in existing pages.
