# Spike UI Page — Specification

## Overview

Add a `SpikePage` React component that gives users a read-only view of all spike investigations — their verdict, lineage to ponder/knowledge entries, and next-step guidance. Spikes are time-boxed investigations that conclude with an ADOPT, ADAPT, or REJECT verdict. This page surfaces those outcomes so teams can act on them.

## User Stories

1. **As a developer**, I want to see all spikes and their verdicts at a glance so I can decide what to do next.
2. **As a developer** who ran an ADAPT spike, I want to promote it to a Ponder entry with one click so the idea enters the planning pipeline.
3. **As a developer** reviewing an ADOPT spike, I want to see a clear next-step hint so I know the tool/command to continue with.
4. **As a developer** on a new team, I want an empty-state explainer so I understand what spikes are and how to create one.

## Functional Requirements

### Data Shape

Each spike object returned by `GET /api/spikes`:
```
{
  slug: string,
  title: string,
  verdict: "ADOPT" | "ADAPT" | "REJECT",
  date: string,          // ISO date
  the_question: string,  // the spike's driving question
  ponder_slug?: string,  // set when already promoted to ponder
  knowledge_slug?: string // set for REJECT spikes auto-filed to knowledge
}
```

### List View (`/spikes`)

- Renders a list of spike rows, each showing: title, verdict badge, date, the_question snippet.
- Verdict color coding: ADOPT=green, ADAPT=yellow, REJECT=red.
- For ADAPT rows: show an inline "Promote to Ponder →" affordance (button or link). If `ponder_slug` is already set, show "Ponder: `<ponder_slug>`" as a link to `/ponder/<ponder_slug>` instead.
- For ADOPT rows: show a next-step hint chip "Next: /sdlc-hypothetical-planning".
- For REJECT rows: show an "Stored in Knowledge" badge. If `knowledge_slug` is set, badge links to `/knowledge/<knowledge_slug>`.
- Empty state: explains what spikes are ("time-boxed investigations that answer one question"), shows example command format `/sdlc-spike <slug> — <need>`, encourages running `/sdlc-spike`.

### Detail View (`/spikes/:slug`)

- Breadcrumb: "Spikes / `<title>`"
- Verdict badge (color-coded as above).
- The question (`the_question`) displayed prominently.
- Date.
- For ADOPT: "What's Next" section — explains ADOPT means the approach is proven (not yet implemented), shows `/sdlc-hypothetical-planning <slug>` hint with copy button.
- For ADAPT: "Promote to Ponder" button — calls `POST /api/spikes/:slug/promote`, then navigates to `/ponder/<ponder_slug>` from the response. If already promoted (`ponder_slug` set), show a link to the existing ponder entry instead.
- For REJECT: "Stored in Knowledge" section with badge and link to `/knowledge/<knowledge_slug>` if available.
- Ponder lineage: if `ponder_slug` is set, show "Ponder: `<slug>`" link.

### Sidebar Navigation

- Add "Spikes" to the `plan` nav group in `Sidebar.tsx`.
- Icon: `FlaskConical` from lucide-react.
- Route: `/spikes`, `exact: false`.

### Mobile (BottomTabBar)

- Add `/spikes` to the `Plan` tab roots array in `BottomTabBar.tsx`.

### Routing

- Add routes in `App.tsx`: `/spikes` and `/spikes/:slug` both render `<SpikePage />`.

## Non-Functional Requirements

- No SSE/real-time updates needed — spikes are immutable once created.
- No create/edit form — spikes are created from the CLI (`/sdlc-spike`), not the UI.
- Follows the same layout pattern as `InvestigationPage.tsx`: left pane list + right pane detail, responsive (mobile collapses to single pane).
- Uses `api.client.ts` pattern for fetch calls.
- Types defined in `frontend/src/lib/types.ts`.

## Out of Scope

- Creating spikes from the UI.
- Editing spike content.
- SSE streaming.
- Any backend changes (REST endpoints are provided by existing server code).
