# Spike UI Page — Task Breakdown

## Implementation Tasks

### T1 — SpikePage list view skeleton
Add `frontend/src/pages/SpikePage.tsx` with the two-pane layout (left list, right detail), verdict filter tabs (ALL / ADOPT / ADAPT / REJECT), spike rows with verdict badge color coding (ADOPT=green, ADAPT=yellow, REJECT=red).

### T2 — Per-verdict row affordances
- ADOPT rows: show "Next: /sdlc-hypothetical-planning" hint chip.
- ADAPT rows: show "Promote to Ponder →" inline button. If `ponder_slug` is already set, show "Ponder: `<slug>`" link to `/ponder/<ponder_slug>` instead.
- REJECT rows: show "Stored in Knowledge" badge; link to `/knowledge/<knowledge_slug>` if set.

### T3 — Detail pane
Right pane showing: breadcrumb ("Spikes / `<title>`"), verdict badge, `the_question` prominently, date, and verdict-specific sections.
- ADOPT: "What's Next" card with explanation + CLI hint with copy button.
- ADAPT: "Promote to Ponder" button (calls POST, then navigates). If already promoted, show link to existing ponder entry.
- REJECT: "Stored in Knowledge" card with badge + link.

### T4 — Promote action
`POST /api/spikes/:slug/promote` call — on success, navigate to `/ponder/<ponder_slug>`. Show inline error on failure. Loading state on button during request.

### T5 — Sidebar nav entry
Add to `plan` group in `Sidebar.tsx`: `{ path: '/spikes', label: 'Spikes', icon: FlaskConical, exact: false }`. Import `FlaskConical` from `lucide-react`.

### T6 — Router + BottomTabBar
Add routes in `App.tsx`: `/spikes` and `/spikes/:slug` both render `<SpikePage />`. Add `/spikes` to `Plan` tab roots in `BottomTabBar.tsx`.

### T7 — ADOPT detail "What's Next" section
In detail pane for ADOPT spikes: dedicated card explaining that ADOPT = proven approach, not yet implemented. Show `/sdlc-hypothetical-planning <slug>` command with copy button.

### T8 — Ponder lineage in list view
After promote (or when `ponder_slug` pre-populated), ADAPT spike rows show "Ponder: `<slug>`" as a link to `/ponder/<ponder_slug>`.

### T9 — Empty state
When spike list is empty: `FlaskConical` icon, "No spikes yet" heading, explanation of what spikes are, example CLI format `/sdlc-spike <slug> — <the question>`.

## Types + API Additions
- Add `SpikeSummary`, `SpikeDetail`, `SpikeVerdict` to `frontend/src/lib/types.ts`.
- Add `getSpikes`, `getSpike`, `promoteSpike` to `frontend/src/api/client.ts`.
